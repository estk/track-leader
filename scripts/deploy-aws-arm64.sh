#!/bin/bash
# AWS EC2 ARM64 (Amazon Linux 2023) Deployment Script for Track Leader
#
# Pulls pre-built images from ghcr.io — no local compilation needed.
#
# Prerequisites:
#   1. EC2 t4g.micro (ARM64) instance with Amazon Linux 2023
#   2. Security group allowing ports 22, 80, 443
#   3. Elastic IP attached (recommended for stable DNS)
#   4. Supabase project with database ready
#   5. GitHub Personal Access Token (PAT) with read:packages scope
#
# Usage:
#   1. SSH into your instance: ssh -i your-key.pem ec2-user@your-ip
#   2. Clone repo or upload this script
#   3. Run: chmod +x deploy-aws-arm64.sh && ./deploy-aws-arm64.sh

set -euo pipefail

echo "=========================================="
echo "Track Leader - AWS ARM64 Deployment"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running on Amazon Linux
if ! grep -q "Amazon Linux" /etc/os-release 2>/dev/null; then
    log_warn "This script is designed for Amazon Linux 2023. Proceeding anyway..."
fi

# Check architecture
ARCH=$(uname -m)
if [ "$ARCH" != "aarch64" ]; then
    log_warn "Expected ARM64 (aarch64), got $ARCH. Script may still work."
fi

NERDCTL_VERSION="2.0.4"

echo ""
log_info "Step 1: Installing containerd + nerdctl..."
echo ""

if ! command -v nerdctl &> /dev/null; then
    sudo dnf update -y
    sudo dnf install -y tar gzip curl postgresql16 --allowerasing

    # Download nerdctl full package (includes containerd, CNI plugins)
    NERDCTL_ARCHIVE="nerdctl-full-${NERDCTL_VERSION}-linux-arm64.tar.gz"
    curl -sSL "https://github.com/containerd/nerdctl/releases/download/v${NERDCTL_VERSION}/${NERDCTL_ARCHIVE}" \
        -o "/tmp/${NERDCTL_ARCHIVE}"

    # Extract to /usr/local
    sudo tar -xzf "/tmp/${NERDCTL_ARCHIVE}" -C /usr/local
    rm -f "/tmp/${NERDCTL_ARCHIVE}"

    # Enable and start containerd
    sudo systemctl daemon-reload
    sudo systemctl enable --now containerd

    log_info "nerdctl ${NERDCTL_VERSION} installed."
else
    log_info "nerdctl already installed: $(nerdctl --version)"
fi

echo ""
log_info "Step 2: Authenticate with GitHub Container Registry..."
echo ""

if ! sudo nerdctl pull ghcr.io/estk/track-leader/backend:latest --quiet 2>/dev/null; then
    GH_USER="${GH_USER:-}"
    GH_TOKEN="${GH_TOKEN:-}"
    if [ -z "$GH_USER" ] || [ -z "$GH_TOKEN" ]; then
        log_info "Log in to ghcr.io to pull private images."
        echo "You need a GitHub Personal Access Token (PAT) with read:packages scope."
        echo "Create one at: https://github.com/settings/tokens"
        echo ""
        read -rp "GitHub username: " GH_USER
        read -rsp "GitHub PAT: " GH_TOKEN
        echo ""
    fi
    echo "$GH_TOKEN" | sudo nerdctl login ghcr.io -u "$GH_USER" --password-stdin
else
    log_info "Already authenticated with ghcr.io."
fi

echo ""
log_info "Step 3: Setting up application directory..."
echo ""

APP_DIR="/home/ec2-user/track-leader"

REPO_URL="${REPO_URL:-https://github.com/estk/track-leader.git}"

if [ ! -d "$APP_DIR" ]; then
    log_info "Cloning repository..."
    sudo dnf install -y git
    git clone "$REPO_URL" "$APP_DIR"
else
    log_info "App directory exists."
fi

cd "$APP_DIR"

echo ""
log_info "Step 4: Environment configuration..."
echo ""

if [ ! -f ".env.production" ]; then
    if [ -f ".env.production.example" ]; then
        cp .env.production.example .env.production
        log_warn "Created .env.production from example. Please edit it with your values:"
        echo ""
        echo "  nano $APP_DIR/.env.production"
        echo ""
        echo "Required values:"
        echo "  - DATABASE_URL: Your Supabase connection string"
        echo "  - PASETO_KEY: Run 'openssl rand -hex 32' to generate"
        echo "  - DOMAIN: Your domain name"
        echo ""
    else
        log_error ".env.production.example not found. Create .env.production manually."
    fi
else
    log_info ".env.production already exists."
fi

echo ""
log_info "Step 5: Pulling images and starting services..."
echo ""

# Check if .env.production has been configured
if [ -f ".env.production" ]; then
    if grep -q "^DATABASE_URL=$" .env.production || ! grep -q "^DATABASE_URL=" .env.production; then
        log_error "DATABASE_URL not set in .env.production"
        log_info "Edit the file and re-run this script, or run manually:"
        echo ""
        echo "  cd $APP_DIR"
        echo "  sudo nerdctl compose -f docker-compose.deploy.yml --env-file .env.production up -d"
        echo ""
        exit 1
    fi
fi

# Pull and start
log_info "Pulling images..."
sudo nerdctl compose -f docker-compose.deploy.yml --env-file .env.production pull

log_info "Starting services..."
sudo nerdctl compose -f docker-compose.deploy.yml --env-file .env.production up -d

echo ""
log_info "Step 6: Verifying deployment..."
echo ""

sleep 10  # Give services time to start

# Check container status
echo "Container status:"
sudo nerdctl compose -f docker-compose.deploy.yml ps

echo ""

# Check health
if curl -sf http://localhost:3001/health > /dev/null 2>&1; then
    log_info "Backend health check: PASSED"
else
    log_warn "Backend health check: PENDING (may still be starting)"
fi

if curl -sf http://localhost:3000 > /dev/null 2>&1; then
    log_info "Frontend health check: PASSED"
else
    log_warn "Frontend health check: PENDING (may still be starting)"
fi

echo ""
echo "=========================================="
echo "Deployment Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Point your domain DNS to this server's IP"
echo "  2. Caddy will automatically obtain SSL certificates"
echo "  3. Access your app at https://\$DOMAIN"
echo ""
# Add shell alias
ALIAS_LINE="alias trs='sudo nerdctl compose -f ~/track-leader/docker-compose.deploy.yml --env-file ~/track-leader/.env.production'"
if ! grep -q "alias trs=" ~/.bashrc 2>/dev/null; then
    echo "$ALIAS_LINE" >> ~/.bashrc
    log_info "Added 'trs' alias to ~/.bashrc (run: source ~/.bashrc)"
fi

echo "Useful commands (after 'source ~/.bashrc'):"
echo "  trs ps              Container status"
echo "  trs logs -f         Tail all logs"
echo "  trs logs -f backend Tail backend logs"
echo "  trs restart backend Restart a service"
echo "  trs down            Stop everything"
echo "  trs up -d           Start everything"
echo "  sudo nerdctl stats Resource usage"
echo ""
echo "To enable automated deployment from GitHub Actions:"
echo "  1. ssh-keygen -t ed25519 -f ~/.ssh/github_actions -N ''"
echo "  2. cat ~/.ssh/github_actions.pub >> ~/.ssh/authorized_keys"
echo "  3. cat ~/.ssh/github_actions  # copy this private key"
echo "  4. Add it as GitHub secret EC2_SSH_KEY"
echo "  5. Add this server's IP as GitHub secret EC2_HOST"
echo "  6. Add 'ec2-user' as GitHub secret EC2_USER"
echo ""
