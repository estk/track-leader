# Deployment Guide

Track Leader runs on AWS EC2 t4g.micro (ARM64) with nerdctl/containerd and Supabase. Deployment is automated via GitHub Actions — push a git tag and everything deploys.

## One-Time Setup

These steps are required before automated deployment works. Do them in order.

### 1. Supabase

1. Create a project at https://supabase.com
2. SQL Editor > paste `scripts/supabase-setup.sql` > run
3. Settings > Database > Connection string > select **Session** mode > copy the URI

### 2. AWS SSM Parameter Store

Store secrets in SSM so EC2 can pull them automatically during provisioning.

1. Open AWS Systems Manager > Parameter Store (same region as your EC2)
2. Create parameter `/track-leader/ghcr-token`:
   - Type: SecureString
   - Value: GitHub Personal Access Token with `read:packages` scope
   - Create one at: https://github.com/settings/tokens
3. Create parameter `/track-leader/env-production`:
   - Type: SecureString
   - Value: contents of your `.env.production` file (see `.env.production.example` for template)
   - Required values: `DATABASE_URL` (Supabase Session mode URI), `PASETO_KEY` (`openssl rand -hex 32`), `DOMAIN`

### 3. EC2 Instance

1. Create IAM role `track-leader-ec2`:
   - Attach `AmazonSSMReadOnlyAccess` managed policy
   - (Or create a custom policy allowing `ssm:GetParameter` on `arn:aws:ssm:*:*:parameter/track-leader/*`)

2. Launch EC2 instance:
   - AMI: Amazon Linux 2023 (ARM64)
   - Instance type: t4g.micro (free tier eligible for 12 months)
   - Storage: 8GB gp3
   - Security group: SSH (22) from your IP, HTTP (80) + HTTPS (443) from anywhere
   - IAM instance profile: `track-leader-ec2`
   - **User data**: paste contents of `scripts/ec2-user-data.sh`

3. Attach an Elastic IP and point your domain's A record to it.

4. The user data script will automatically:
   - Install containerd + nerdctl
   - Authenticate with ghcr.io
   - Clone the repo
   - Pull `.env.production` from SSM
   - Start all services

   Check provisioning logs: `cat /var/log/track-leader-setup.log`

**Alternative (manual setup):** If you prefer not to use user data / SSM, SSH in and run `./scripts/deploy-aws-arm64.sh` which walks through setup interactively.

### 4. SSH Key for GitHub Actions

On the EC2 instance:

```bash
ssh-keygen -t ed25519 -f ~/.ssh/github_actions -N ''
cat ~/.ssh/github_actions.pub >> ~/.ssh/authorized_keys
cat ~/.ssh/github_actions  # copy this private key
```

### 5. GitHub Secrets

In your repo: Settings > Secrets and Variables > Actions, add:

| Secret | Value |
|--------|-------|
| `EC2_HOST` | Your Elastic IP address |
| `EC2_SSH_KEY` | Private key from step 4 |
| `EC2_USER` | `ec2-user` |

---

## Architecture

```
  Developer (local)
       |
       | git tag v0.X.0 <sha> && git push origin v0.X.0
       v
  GitHub Actions (.github/workflows/deploy.yml)
       |
       | 1. Builds linux/arm64 images
       | 2. Pushes to ghcr.io (tagged :latest and :X.X.X)
       | 3. SSHes into EC2, runs scripts/deploy-update.sh
       v
  EC2 t4g.micro (Amazon Linux 2023, ARM64)
  +-----------------------+
  |  Caddy  :80/:443      |  <- automatic HTTPS via Let's Encrypt
  |    |          |        |
  | Frontend  Backend      |
  |  :3000     :3001       |
  +-----------|------------+
              |
         Supabase
        (PostgreSQL + PostGIS)
```

### Key Files

| File | Purpose |
|------|---------|
| `.github/workflows/deploy.yml` | CI/CD: build, push, deploy on tag |
| `docker-compose.deploy.yml` | Production compose (pulls pre-built images) |
| `caddy/Caddyfile` | Reverse proxy with automatic HTTPS |
| `.env.production.example` | Template for production secrets |
| `scripts/deploy-update.sh` | Deploy script called by CI (or manually for rollback) |
| `scripts/deploy-aws-arm64.sh` | Interactive first-time EC2 setup |
| `scripts/ec2-user-data.sh` | Non-interactive EC2 provisioning (cloud-init) |
| `scripts/supabase-setup.sql` | Database schema for Supabase SQL Editor |

---

## Releasing a Version

```bash
# Get the git commit SHA for the current jj change
SHA=$(jj log --no-graph -r @ -T 'commit_id' | head -c 40)

# Create and push a git tag (triggers CI/CD)
git tag v0.2.0 "$SHA"
git push origin v0.2.0
```

This triggers the full pipeline: build images → push to ghcr.io → SSH to EC2 → pull → restart → health check.

---

## Rollback

### Container Rollback

CI pushes version-tagged images (e.g., `backend:0.1.0`) alongside `:latest`. Old versions remain in ghcr.io indefinitely.

To roll back to a previous version, SSH into EC2:

```bash
cd ~/track-leader
./scripts/deploy-update.sh 0.1.0
```

This pulls the old versioned images, tags them as `:latest`, and recreates the containers.

If a deploy fails health checks, the CI job exits non-zero. The containers may be in a bad state — roll back with the command above.

### Database Rollback

Every deploy takes a `pg_dump` snapshot before touching anything. Backups are stored in `~/backups/` on EC2 (last 5 kept).

To restore from a pre-deploy snapshot:

```bash
# Stop the backend so no new writes happen
sudo nerdctl compose -f docker-compose.deploy.yml stop backend

# Restore (gunzip pipes into psql)
gunzip -c ~/backups/pre-deploy-YYYYMMDD-HHMMSS.sql.gz | psql "$DATABASE_URL"

# Roll back the container image too
./scripts/deploy-update.sh 0.1.0
```

For surgical rollbacks, new migrations use reversible format (`.up.sql`/`.down.sql` pairs) — run the `down.sql` manually via Supabase SQL Editor or psql.

### Full Rollback Checklist

1. Identify the last known-good version (check GitHub Actions history)
2. If the migration was destructive (dropped columns/tables), restore from snapshot first
3. If the migration was additive (added columns/indexes), you may skip the DB restore — the old code simply won't use the new columns
4. Roll back containers: `./scripts/deploy-update.sh <good-version>`
5. Verify: `curl https://<domain>/health`

---

## Database Migrations

SQLx migrations run automatically on backend startup. A `pg_dump` snapshot is taken before each deploy (see Rollback above).

### Writing Migrations

Existing migrations (001-012) use simple `.sql` format (non-reversible). New migrations must use the reversible format:

```
migrations/
  013_add_feature.up.sql    # forward migration
  013_add_feature.down.sql  # reverse migration
```

SQLx supports mixing simple and reversible migrations. The `down.sql` should cleanly undo what `up.sql` did. Example:

```sql
-- 013_add_feature.up.sql
ALTER TABLE activities ADD COLUMN elevation_gain_m DOUBLE PRECISION;

-- 013_add_feature.down.sql
ALTER TABLE activities DROP COLUMN elevation_gain_m;
```

### Concurrent Indexes (Large Tables)

Some index migrations lock the table during creation. For production tables with existing data, create these indexes manually using `CONCURRENTLY` to avoid blocking writes:

```sql
-- Run each index separately (CONCURRENTLY cannot be inside a transaction)

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_activities_user_type_date
  ON activities(user_id, activity_type, submitted_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_efforts_segment_time
  ON segment_efforts(segment_id, elapsed_time_seconds ASC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_efforts_user_segment
  ON segment_efforts(user_id, segment_id, elapsed_time_seconds ASC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notifications_user_unread
  ON notifications(user_id, read_at) WHERE read_at IS NULL;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notifications_user_time
  ON notifications(user_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_activities_feed
  ON activities(submitted_at DESC) WHERE visibility = 'public';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_follows_follower
  ON follows(follower_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_follows_following
  ON follows(following_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_kudos_activity
  ON kudos(activity_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_comments_activity
  ON comments(activity_id, created_at ASC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_segments_type
  ON segments(activity_type, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_segment_stars_user
  ON segment_stars(user_id, created_at DESC);
```

Monitor progress:

```sql
SELECT a.pid, a.query, p.phase,
  round(100.0 * p.blocks_done / nullif(p.blocks_total, 0), 1) AS "% done"
FROM pg_stat_activity a
JOIN pg_stat_progress_create_index p ON p.pid = a.pid;
```

---

## Manual Operations

### Shell Alias

Add this to `~/.bashrc` on EC2 to avoid typing the full compose command every time:

```bash
alias trs='sudo nerdctl compose -f ~/track-leader/docker-compose.deploy.yml --env-file ~/track-leader/.env.production'
```

Then reload: `source ~/.bashrc`

### Common Commands

| Command | What it does |
|---------|-------------|
| `trs ps` | Container status |
| `trs logs -f` | Tail all logs |
| `trs logs -f backend` | Tail backend logs |
| `trs logs --tail=100 backend` | Last 100 lines of backend |
| `trs restart backend` | Restart a service |
| `trs stop backend` | Stop a service |
| `trs down` | Stop everything |
| `trs up -d` | Start everything |
| `trs up -d --force-recreate` | Recreate all containers |
| `sudo nerdctl stats` | Resource usage |
| `sudo nerdctl image prune -a` | Clean up old images (important on 8GB disk) |
| `ls -lht ~/backups/` | List DB snapshots |

---

## Troubleshooting

### Backend won't start
```bash
sudo nerdctl compose -f docker-compose.deploy.yml logs backend
```
Common causes: `DATABASE_URL` wrong, `PASETO_KEY` not set or not 64 hex chars.

### Can't reach Supabase (IPv6)

Supabase only exposes an IPv6 address (`AAAA` record). EC2 instances need IPv6 connectivity.

- Verify: `dig AAAA db.<project-ref>.supabase.co` should return an IPv6 address
- Test: `curl -6 -sv https://db.<project-ref>.supabase.co:5432 2>&1 | head -10`
- Fix: Ensure your VPC subnet has IPv6 CIDR assigned, security group allows outbound IPv6
- Use the **Session mode** connection pooler (not Transaction mode) — it routes via `aws-0-<region>.pooler.supabase.com` which has IPv4

### SSL not working
```bash
sudo nerdctl compose -f docker-compose.deploy.yml logs caddy
```
Common causes: domain not pointing to server IP, ports 80/443 blocked in security group.

### Ghost containers (nerdctl)

If `nerdctl compose down` leaves zombie containers:

```bash
# Nuclear option: restart containerd
sudo systemctl restart containerd
# Then recreate
sudo nerdctl compose -f docker-compose.deploy.yml --env-file .env.production up -d --force-recreate
```

### Can't pull images
```bash
# Re-authenticate
sudo nerdctl login ghcr.io
# Verify image exists
sudo nerdctl pull ghcr.io/estk/track-leader/backend:latest
```

### Disk full
```bash
# Remove all unused images (not just dangling)
sudo nerdctl image prune -a
# Check disk
df -h
```

### Health check fails in CI

The deploy script waits up to 60 seconds for each service. If it times out:
1. Check logs (the script prints them on failure)
2. SSH in and check manually: `curl http://localhost:3001/health`
3. If the backend is still starting (migration running), increase retry count in `scripts/deploy-update.sh`

---

## Cost Estimate

| Service | Cost |
|---------|------|
| AWS t4g.micro | Free for 12 months (750 hrs/mo) |
| Supabase free tier | $0 |
| ghcr.io (private, <500MB) | $0 |
| GitHub Actions (~10 min/build) | Free tier (2000 min/mo) |
| SSM Parameter Store (2 params) | $0 |
| **Total** | **$0/month** for the first year |

After free tier expires:
- t4g.micro: ~$6/month
- Supabase Pro (if needed): $25/month
