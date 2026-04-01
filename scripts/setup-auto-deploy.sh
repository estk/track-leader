#!/bin/bash
# Install systemd timer for auto-deployment on EC2.
# Run this once on the instance: ./scripts/setup-auto-deploy.sh
set -euo pipefail

echo "[setup] Installing auto-deploy systemd units..."

sudo tee /etc/systemd/system/track-leader-deploy.service > /dev/null <<'EOF'
[Unit]
Description=Track Leader auto-deploy check
After=network-online.target containerd.service
Wants=network-online.target

[Service]
Type=oneshot
User=ec2-user
ExecStart=/home/ec2-user/track-leader/scripts/auto-deploy.sh
TimeoutStartSec=300
EOF

sudo tee /etc/systemd/system/track-leader-deploy.timer > /dev/null <<'EOF'
[Unit]
Description=Check for new Track Leader images every minute

[Timer]
OnBootSec=60
OnUnitActiveSec=60
AccuracySec=5

[Install]
WantedBy=timers.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now track-leader-deploy.timer

echo "[setup] Auto-deploy timer installed and running."
echo ""
echo "Useful commands:"
echo "  sudo systemctl status track-leader-deploy.timer    # timer status"
echo "  journalctl -u track-leader-deploy.service -f       # deployment logs"
echo "  sudo systemctl disable --now track-leader-deploy.timer  # disable"
