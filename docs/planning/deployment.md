# Track Leader Deployment Guide

AWS ARM64 (t4g.micro) + Supabase Free Tier

## Files Created

| File | Purpose |
|------|---------|
| `docker-compose.supabase.yml` | Slim compose with backend, frontend, Caddy (no postgres) |
| `scripts/supabase-setup.sql` | Combined migrations to run in Supabase SQL Editor |
| `caddy/Caddyfile` | Automatic HTTPS reverse proxy config |
| `.env.production.example` | Template for production secrets |
| `scripts/deploy-aws-arm64.sh` | Setup script for Amazon Linux 2023 ARM64 |

## Deployment Steps

### 1. Supabase Setup

1. Create a Supabase project at https://supabase.com
2. Go to SQL Editor, paste contents of `scripts/supabase-setup.sql`, run it
3. Get your connection string: Settings > Database > Connection string (use Session mode)

### 2. AWS Setup

1. Launch EC2 instance:
   - AMI: Amazon Linux 2023 (ARM64)
   - Instance type: t4g.micro (free tier eligible)
   - Storage: 8GB gp3 (default)

2. Security group rules:
   - SSH (22) from your IP
   - HTTP (80) from anywhere
   - HTTPS (443) from anywhere

3. Attach an Elastic IP (recommended for stable DNS)

4. Point your domain A record to the Elastic IP

### 3. On the VPS

```bash
ssh -i your-key.pem ec2-user@your-ip
git clone <your-repo> track-leader
cd track-leader
./scripts/deploy-aws-arm64.sh
```

### 4. Configure Secrets

```bash
nano .env.production
```

Required values:
- `DATABASE_URL`: Your Supabase connection string
- `PASETO_KEY`: Run `openssl rand -hex 32` to generate
- `DOMAIN`: Your domain name (e.g., tracks.example.com)

### 5. Start Services

```bash
docker compose -f docker-compose.supabase.yml --env-file .env.production up -d --build
```

Caddy handles SSL automatically once your domain resolves to the server.

## Useful Commands

```bash
# View logs
docker compose -f docker-compose.supabase.yml logs -f

# View specific service logs
docker compose -f docker-compose.supabase.yml logs -f backend

# Stop services
docker compose -f docker-compose.supabase.yml down

# Restart services
docker compose -f docker-compose.supabase.yml restart

# Update and rebuild
git pull && docker compose -f docker-compose.supabase.yml up -d --build

# Monitor resources
docker stats
```

## Cost Estimate

| Service | Cost |
|---------|------|
| AWS t4g.micro | Free for 12 months (750 hrs/mo) |
| Supabase free tier | $0 |
| **Total** | **$0/month** for the first year |

After free tier expires:
- t4g.micro: ~$6/month
- Supabase Pro (if needed): $25/month

## Architecture

```
                    ┌─────────────┐
                    │   Caddy     │ :80, :443
                    │ (auto SSL)  │
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
       ┌─────────────┐          ┌─────────────┐
       │  Frontend   │          │   Backend   │
       │  (Next.js)  │ ────────▶│   (Rust)    │
       │    :3000    │  /api/*  │    :3001    │
       └─────────────┘          └──────┬──────┘
                                       │
                                       ▼
                               ┌─────────────┐
                               │  Supabase   │
                               │ (PostgreSQL │
                               │ + PostGIS)  │
                               └─────────────┘
```

## Troubleshooting

### Backend won't start
```bash
# Check logs
docker compose -f docker-compose.supabase.yml logs backend

# Common issues:
# - DATABASE_URL incorrect or unreachable
# - PASETO_KEY not set or wrong length
```

### SSL certificate not working
```bash
# Check Caddy logs
docker compose -f docker-compose.supabase.yml logs caddy

# Common issues:
# - Domain not pointing to server IP yet
# - Ports 80/443 not open in security group
```

### Supabase connection issues
- Use "Session" mode in connection pooler (not "Transaction")
- Keep `DATABASE_MAX_CONNECTIONS` low (10) for free tier
- Check if your IP is allowed in Supabase network settings
