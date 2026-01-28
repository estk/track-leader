#!/bin/bash
# restart-dev-docker.sh - Restart a specific service in Docker dev environment
#
# Usage: ./scripts/restart-dev-docker.sh <service>
#
# Services: backend, frontend, postgres, all

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORTS_FILE="$PROJECT_ROOT/.dev-ports-docker"

SERVICE="${1:-}"

if [ -z "$SERVICE" ]; then
    echo "Usage: $0 <service>"
    echo ""
    echo "Services:"
    echo "  backend   - Restart the Rust/Axum backend"
    echo "  frontend  - Restart the Next.js frontend"
    echo "  postgres  - Restart PostgreSQL container"
    echo "  all       - Restart all services"
    exit 1
fi

# Load port configuration
if [ -f "$PORTS_FILE" ]; then
    source "$PORTS_FILE"
else
    echo "Error: Port configuration not found at $PORTS_FILE"
    echo "Start the dev environment first with: ./scripts/start-dev-docker.sh"
    exit 1
fi

cd "$PROJECT_ROOT"

case "$SERVICE" in
    backend)
        echo "Restarting backend..."
        docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml restart backend
        ;;
    frontend)
        echo "Restarting frontend..."
        docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml restart frontend
        ;;
    postgres)
        echo "Restarting PostgreSQL..."
        docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml restart postgres
        ;;
    all)
        echo "Restarting all services..."
        docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml restart
        ;;
    *)
        echo "Unknown service: $SERVICE"
        echo "Valid services: backend, frontend, postgres, all"
        exit 1
        ;;
esac

echo "Done."
