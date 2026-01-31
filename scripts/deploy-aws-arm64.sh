#!/bin/bash
# AWS EC2 ARM64 (Amazon Linux 2023) Deployment Script for Track Leader
#
# This script sets up a fresh Amazon Linux 2023 ARM64 instance to run Track Leader.
# Run this on your EC2 instance after SSHing in.
#
# Prerequisites:
#   1. EC2 t4g.micro (ARM64) instance with Amazon Linux 2023
#   2. Security group allowing ports 22, 80, 443
#   3. Elastic IP attached (recommended for stable DNS)
#   4. Supabase project with database ready
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

echo ""
log_info "Step 1: Installing Docker..."
echo ""

# Install Docker on Amazon Linux 2023
if ! command -v docker &> /dev/null; then
    sudo dnf update -y
    sudo dnf install -y docker
    sudo systemctl start docker
    sudo systemctl enable docker
    sudo usermod -aG docker ec2-user
    log_info "Docker installed. You may need to log out and back in for group changes."
else
    log_info "Docker already installed."
fi

# Install Docker Compose plugin
if ! docker compose version &> /dev/null; then
    log_info "Installing Docker Compose plugin..."
    sudo mkdir -p /usr/local/lib/docker/cli-plugins
    sudo curl -SL "https://github.com/docker/compose/releases/latest/download/docker-compose-linux-aarch64" \
        -o /usr/local/lib/docker/cli-plugins/docker-compose
    sudo chmod +x /usr/local/lib/docker/cli-plugins/docker-compose
else
    log_info "Docker Compose already installed."
fi

echo ""
log_info "Step 2: Installing Git..."
echo ""

if ! command -v git &> /dev/null; then
    sudo dnf install -y git
else
    log_info "Git already installed."
fi

echo ""
log_info "Step 3: Setting up application directory..."
echo ""

APP_DIR="/home/ec2-user/track-leader"

if [ ! -d "$APP_DIR" ]; then
    log_info "Cloning repository..."
    echo "Enter your git repository URL (or press Enter to skip if uploading manually):"
    read -r REPO_URL
    if [ -n "$REPO_URL" ]; then
        git clone "$REPO_URL" "$APP_DIR"
    else
        mkdir -p "$APP_DIR"
        log_warn "Created empty directory. Upload your code to $APP_DIR"
    fi
else
    log_info "App directory exists. Pulling latest..."
    cd "$APP_DIR"
    git pull || log_warn "Git pull failed - manual update may be needed"
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
log_info "Step 5: Building and starting services..."
echo ""

# Check if .env.production has been configured
if [ -f ".env.production" ]; then
    if grep -q "^DATABASE_URL=$" .env.production || ! grep -q "^DATABASE_URL=" .env.production; then
        log_error "DATABASE_URL not set in .env.production"
        log_info "Edit the file and re-run this script, or run manually:"
        echo ""
        echo "  cd $APP_DIR"
        echo "  docker compose -f docker-compose.supabase.yml --env-file .env.production up -d --build"
        echo ""
        exit 1
    fi
fi

# Build and start
log_info "Building containers (this may take 5-10 minutes on first run)..."
docker compose -f docker-compose.supabase.yml --env-file .env.production build

log_info "Starting services..."
docker compose -f docker-compose.supabase.yml --env-file .env.production up -d

echo ""
log_info "Step 6: Verifying deployment..."
echo ""

sleep 10  # Give services time to start

# Check container status
echo "Container status:"
docker compose -f docker-compose.supabase.yml ps

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
echo "Useful commands:"
echo "  View logs:     docker compose -f docker-compose.supabase.yml logs -f"
echo "  Stop:          docker compose -f docker-compose.supabase.yml down"
echo "  Restart:       docker compose -f docker-compose.supabase.yml restart"
echo "  Update:        git pull && docker compose -f docker-compose.supabase.yml up -d --build"
echo ""
echo "Monitor resources:"
echo "  docker stats"
echo ""
