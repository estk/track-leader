#!/bin/bash
# Polls for new container images and deploys if changed.
# Designed to run from a systemd timer every minute.
set -euo pipefail

APP_DIR="$HOME/track-leader"
COMPOSE="sudo nerdctl compose -f docker-compose.deploy.yml --env-file .env.production"
DIGEST_FILE="/var/tmp/track-leader-image-digests"
BACKUP_DIR="$HOME/backups"
TAG="auto-deploy"

# Prevent concurrent runs
LOCK_FILE="/var/tmp/track-leader-deploy.lock"
exec 200>"$LOCK_FILE"
flock -n 200 || exit 0

cd "$APP_DIR"

# Pull latest images
$COMPOSE pull 2>/dev/null || { echo "[$TAG] Pull failed"; exit 1; }

# Compare image digests with last deployment
BACKEND_DIGEST=$(sudo nerdctl image inspect ghcr.io/estk/track-leader/backend:latest \
    -f '{{index .RepoDigests 0}}' 2>/dev/null || echo "")
FRONTEND_DIGEST=$(sudo nerdctl image inspect ghcr.io/estk/track-leader/frontend:latest \
    -f '{{index .RepoDigests 0}}' 2>/dev/null || echo "")
CURRENT="${BACKEND_DIGEST}|${FRONTEND_DIGEST}"
PREVIOUS=$(cat "$DIGEST_FILE" 2>/dev/null || echo "")

if [ "$CURRENT" = "$PREVIOUS" ]; then
    exit 0
fi

# Verify both images were built from the same git SHA
BACKEND_SHA=$(sudo nerdctl image inspect ghcr.io/estk/track-leader/backend:latest \
    -f '{{index .Config.Labels "org.opencontainers.image.revision"}}' 2>/dev/null || echo "")
FRONTEND_SHA=$(sudo nerdctl image inspect ghcr.io/estk/track-leader/frontend:latest \
    -f '{{index .Config.Labels "org.opencontainers.image.revision"}}' 2>/dev/null || echo "")

if [ -z "$BACKEND_SHA" ] || [ -z "$FRONTEND_SHA" ]; then
    echo "[$TAG] Images missing revision label, skipping deploy"
    exit 0
fi

if [ "$BACKEND_SHA" != "$FRONTEND_SHA" ]; then
    echo "[$TAG] SHA mismatch: backend=$BACKEND_SHA frontend=$FRONTEND_SHA, skipping"
    exit 0
fi

# Verify the image SHA exists on main
git fetch origin main --quiet
if ! git merge-base --is-ancestor "$BACKEND_SHA" origin/main 2>/dev/null; then
    echo "[$TAG] Image SHA $BACKEND_SHA is not on main, skipping deploy"
    exit 0
fi

echo "[$TAG] New images detected (sha: ${BACKEND_SHA:0:8}), deploying..."

# Pull repo changes (config, compose files, Caddyfile, etc.)
git pull --ff-only origin main 2>/dev/null || true

# Backup database before deploy
set -a; source .env.production; set +a
mkdir -p "$BACKUP_DIR"
BACKUP_FILE="$BACKUP_DIR/pre-deploy-$(date +%Y%m%d-%H%M%S).sql.gz"
pg_dump "$DATABASE_URL" | gzip > "$BACKUP_FILE"
ls -t "$BACKUP_DIR"/pre-deploy-*.sql.gz 2>/dev/null | tail -n +6 | xargs -r rm -f
echo "[$TAG] Backup: $BACKUP_FILE"

# Recreate containers with new images
$COMPOSE up -d --force-recreate

# Health checks (wait up to 60s each)
for i in {1..12}; do
    curl -sf http://localhost:3001/health > /dev/null 2>&1 && echo "[$TAG] Backend: OK" && break
    [ "$i" -eq 12 ] && echo "[$TAG] Backend health check failed" && $COMPOSE logs --tail=20 backend && exit 1
    sleep 5
done

for i in {1..12}; do
    curl -sf http://localhost:3000 > /dev/null 2>&1 && echo "[$TAG] Frontend: OK" && break
    [ "$i" -eq 12 ] && echo "[$TAG] Frontend health check failed" && $COMPOSE logs --tail=20 frontend && exit 1
    sleep 5
done

# Record digests only after successful deploy
echo "$CURRENT" > "$DIGEST_FILE"

# Clean up old images
sudo nerdctl image prune -f > /dev/null 2>&1

echo "[$TAG] Deploy complete."
