#!/bin/bash
# EC2 User Data — runs as root on first boot of Amazon Linux 2023 (ARM64)
# Paste this into EC2 Launch > Advanced Details > User data
set -euo pipefail
exec > /var/log/track-leader-setup.log 2>&1

echo "[setup] Installing packages..."
dnf update -y
dnf install -y tar gzip curl git postgresql16

echo "[setup] Installing containerd + nerdctl..."
NERDCTL_VERSION="2.0.4"
ARCH=$(uname -m | sed 's/aarch64/arm64/')
curl -sSL "https://github.com/containerd/nerdctl/releases/download/v${NERDCTL_VERSION}/nerdctl-full-${NERDCTL_VERSION}-linux-${ARCH}.tar.gz" \
  -o /tmp/nerdctl.tar.gz
tar -xzf /tmp/nerdctl.tar.gz -C /usr/local
rm -f /tmp/nerdctl.tar.gz
systemctl daemon-reload
systemctl enable --now containerd

echo "[setup] Authenticating with ghcr.io..."
REGION=$(ec2-metadata --availability-zone | sed 's/placement: //;s/.$//')
GHCR_TOKEN=$(aws ssm get-parameter --name /track-leader/ghcr-token \
  --with-decryption --query Parameter.Value --output text --region "$REGION")
echo "$GHCR_TOKEN" | sudo nerdctl login ghcr.io -u estk --password-stdin

echo "[setup] Cloning repository..."
su - ec2-user -c "git clone https://github.com/estk/track-leader.git ~/track-leader"

echo "[setup] Adding shell alias..."
echo "alias trs='sudo nerdctl compose -f ~/track-leader/docker-compose.deploy.yml --env-file ~/track-leader/.env.production'" \
  >> /home/ec2-user/.bashrc

echo "[setup] Pulling secrets from SSM..."
aws ssm get-parameter --name /track-leader/env-production \
  --with-decryption --query Parameter.Value --output text --region "$REGION" \
  > /home/ec2-user/track-leader/.env.production
chown ec2-user:ec2-user /home/ec2-user/track-leader/.env.production

echo "[setup] Starting services..."
cd /home/ec2-user/track-leader
sudo nerdctl compose -f docker-compose.deploy.yml --env-file .env.production pull
sudo nerdctl compose -f docker-compose.deploy.yml --env-file .env.production up -d

echo "[setup] Provisioning complete. Check logs: /var/log/track-leader-setup.log"
