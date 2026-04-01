#!/bin/bash
set -euo pipefail

APP_DIR="$HOME/track-leader"
BACKUP_DIR="$HOME/backups"
COMPOSE="sudo nerdctl compose -f docker-compose.deploy.yml --env-file .env.production"
VERSION="${1:-latest}"

cd "$APP_DIR"

# Load DATABASE_URL for pg_dump
set -a
source .env.production
set +a

echo "[deploy] Backing up database before deploy..."
mkdir -p "$BACKUP_DIR"
BACKUP_FILE="$BACKUP_DIR/pre-deploy-$(date +%Y%m%d-%H%M%S).sql.gz"
pg_dump "$DATABASE_URL" | gzip > "$BACKUP_FILE"
echo "[deploy] Backup saved: $BACKUP_FILE"

# Keep only the 5 most recent backups
ls -t "$BACKUP_DIR"/pre-deploy-*.sql.gz 2>/dev/null | tail -n +6 | xargs -r rm -f

echo "[deploy] Pulling repo changes..."
git pull --ff-only origin main

echo "[deploy] Pulling images (version: $VERSION)..."
if [ "$VERSION" != "latest" ]; then
  sudo nerdctl pull "ghcr.io/estk/track-leader/backend:$VERSION"
  sudo nerdctl pull "ghcr.io/estk/track-leader/frontend:$VERSION"
  sudo nerdctl tag "ghcr.io/estk/track-leader/backend:$VERSION" "ghcr.io/estk/track-leader/backend:latest"
  sudo nerdctl tag "ghcr.io/estk/track-leader/frontend:$VERSION" "ghcr.io/estk/track-leader/frontend:latest"
else
  $COMPOSE pull
fi

echo "[deploy] Recreating containers..."
$COMPOSE up -d --force-recreate

echo "[deploy] Health checks..."
for i in {1..12}; do
  curl -sf http://localhost:3001/health > /dev/null 2>&1 && echo "[deploy] Backend: OK" && break
  [ "$i" -eq 12 ] && echo "[deploy] ERROR: Backend failed" && $COMPOSE logs --tail=30 backend && exit 1
  sleep 5
done

for i in {1..12}; do
  curl -sf http://localhost:3000 > /dev/null 2>&1 && echo "[deploy] Frontend: OK" && break
  [ "$i" -eq 12 ] && echo "[deploy] ERROR: Frontend failed" && $COMPOSE logs --tail=30 frontend && exit 1
  sleep 5
done

echo "[deploy] Pruning old images..."
sudo nerdctl image prune -f

echo "[deploy] Done (version: $VERSION)."
