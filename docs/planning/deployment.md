# Track Leader Deployment Guide

AWS ARM64 (t4g.micro) + Supabase Free Tier. Images built in GitHub CI, pulled on EC2.

## Architecture

```
  GitHub Actions (on tag push)
       |
       | builds linux/arm64 images
       v
    ghcr.io
       |
       | nerdctl compose pull
       v
  EC2 t4g.micro
  +--------------------+
  |  Caddy  :80/:443   |
  |    |          |     |
  | Frontend  Backend   |
  |  :3000     :3001    |
  +-----------|--------+
              |
         Supabase
        (PostgreSQL)
```

## Files

| File | Purpose |
|------|---------|
| `.github/workflows/deploy.yml` | CI: builds + pushes images to ghcr.io on tag |
| `docker-compose.deploy.yml` | Production compose (pulls pre-built images) |
| `caddy/Caddyfile` | Automatic HTTPS reverse proxy config |
| `.env.production.example` | Template for production secrets |
| `scripts/deploy-aws-arm64.sh` | EC2 setup script (installs nerdctl, pulls images) |
| `scripts/supabase-setup.sql` | Combined migrations for Supabase SQL Editor |

## Workflow

### Release a version

```bash
# Tag and push (triggers CI build)
jj commit -m "Release v0.1.0"
jj bookmark set v0.1.0
jj git push --bookmark v0.1.0
```

CI builds both images for `linux/arm64` and pushes to `ghcr.io/estk/track-leader/{backend,frontend}`.

### First-time EC2 setup

1. Launch EC2 instance:
   - AMI: Amazon Linux 2023 (ARM64)
   - Instance type: t4g.micro (free tier eligible)
   - Storage: 8GB gp3

2. Security group: SSH (22) from your IP, HTTP (80) + HTTPS (443) from anywhere

3. Attach Elastic IP, point domain A record to it

4. SSH in and run:
   ```bash
   git clone <your-repo> track-leader
   cd track-leader
   ./scripts/deploy-aws-arm64.sh
   ```

5. The script will:
   - Install containerd + nerdctl
   - Prompt for GitHub PAT (needs `read:packages` scope) to authenticate with ghcr.io
   - Create `.env.production` from the example
   - Pull images and start services

6. Edit `.env.production`:
   - `DATABASE_URL`: Supabase connection string (Session mode)
   - `PASETO_KEY`: `openssl rand -hex 32`
   - `DOMAIN`: your domain

### Deploy an update

On the EC2 instance:

```bash
cd ~/track-leader
sudo nerdctl compose -f docker-compose.deploy.yml pull
sudo nerdctl compose -f docker-compose.deploy.yml up -d
```

### Supabase setup

1. Create a project at https://supabase.com
2. SQL Editor > paste `scripts/supabase-setup.sql` > run
3. Settings > Database > Connection string (Session mode)

## Useful Commands

```bash
# View logs
sudo nerdctl compose -f docker-compose.deploy.yml logs -f

# View specific service
sudo nerdctl compose -f docker-compose.deploy.yml logs -f backend

# Stop
sudo nerdctl compose -f docker-compose.deploy.yml down

# Restart
sudo nerdctl compose -f docker-compose.deploy.yml restart

# Monitor resources
sudo nerdctl stats

# Clean up old images
sudo nerdctl image prune -a
```

## Cost Estimate

| Service | Cost |
|---------|------|
| AWS t4g.micro | Free for 12 months (750 hrs/mo) |
| Supabase free tier | $0 |
| ghcr.io (private, <500MB) | $0 |
| GitHub Actions (~10 min/build) | Free tier (2000 min/mo) |
| **Total** | **$0/month** for the first year |

After free tier expires:
- t4g.micro: ~$6/month
- Supabase Pro (if needed): $25/month

## Troubleshooting

### Backend won't start
```bash
sudo nerdctl compose -f docker-compose.deploy.yml logs backend
# Common: DATABASE_URL wrong, PASETO_KEY not set
```

### SSL not working
```bash
sudo nerdctl compose -f docker-compose.deploy.yml logs caddy
# Common: domain not pointing to server, ports 80/443 blocked
```

### Can't pull images
```bash
# Re-authenticate
sudo nerdctl login ghcr.io
# Check the image exists
sudo nerdctl pull ghcr.io/estk/track-leader/backend:latest
```

### Supabase connection issues
- Use "Session" mode pooler (not "Transaction")
- Keep `DATABASE_MAX_CONNECTIONS` low (10) for free tier
- Check Supabase network settings allow your EC2 IP
