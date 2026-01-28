#!/bin/bash
# status-dev-docker.sh - Show status of Track Leader Docker development environment
#
# Usage: ./scripts/status-dev-docker.sh

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORTS_FILE="$PROJECT_ROOT/.dev-ports-docker"

echo "=== Track Leader Docker Dev Environment Status ==="
echo ""

# Load port configuration if it exists
if [ -f "$PORTS_FILE" ]; then
    source "$PORTS_FILE"
    echo "Configuration loaded from: $PORTS_FILE"
else
    # Fallback to workspace-based project name
    WORKSPACE_NAME=$(basename "$PROJECT_ROOT")
    COMPOSE_PROJECT_NAME="tl_${WORKSPACE_NAME}"
    FRONTEND_PORT=""
    BACKEND_PORT=""
    POSTGRES_PORT=""
    echo "No port configuration found (dev environment not started?)"
fi
echo "Project: $COMPOSE_PROJECT_NAME"
echo ""

cd "$PROJECT_ROOT"

# Check containers
echo "Containers:"
docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml ps 2>/dev/null || echo "  No containers found"
echo ""

# Check ports
echo "Configured Ports:"
if [ -n "$FRONTEND_PORT" ]; then
    echo "  Frontend:   http://localhost:$FRONTEND_PORT"
else
    echo "  Frontend:   unknown"
fi
if [ -n "$BACKEND_PORT" ]; then
    echo "  Backend:    http://localhost:$BACKEND_PORT"
else
    echo "  Backend:    unknown"
fi
if [ -n "$POSTGRES_PORT" ]; then
    echo "  PostgreSQL: localhost:$POSTGRES_PORT"
else
    echo "  PostgreSQL: unknown"
fi
echo ""

# Quick health check
echo "Quick Health Check:"
if [ -n "$FRONTEND_PORT" ]; then
    if curl -s --max-time 2 http://localhost:$FRONTEND_PORT > /dev/null 2>&1; then
        echo "  ✓ Frontend responding on http://localhost:$FRONTEND_PORT"
    else
        echo "  ✗ Frontend not responding on http://localhost:$FRONTEND_PORT"
    fi
fi

if [ -n "$BACKEND_PORT" ]; then
    if curl -s --max-time 2 http://localhost:$BACKEND_PORT/health > /dev/null 2>&1; then
        echo "  ✓ Backend responding on http://localhost:$BACKEND_PORT"
    elif curl -s --max-time 2 http://localhost:$BACKEND_PORT > /dev/null 2>&1; then
        echo "  ✓ Backend responding on http://localhost:$BACKEND_PORT (no /health endpoint)"
    else
        echo "  ✗ Backend not responding on http://localhost:$BACKEND_PORT"
    fi
fi
echo ""

# Volumes (project name is used as prefix by docker compose)
echo "Volumes:"
docker volume ls --filter "name=${COMPOSE_PROJECT_NAME}_" --format "  {{.Name}}" 2>/dev/null || echo "  No volumes found"
echo ""

echo "Commands:"
echo "  Start:   ./scripts/start-dev-docker.sh"
echo "  Stop:    ./scripts/stop-dev-docker.sh"
echo "  Logs:    ./scripts/logs-dev-docker.sh [backend|frontend|postgres]"
echo "  Rebuild: ./scripts/start-dev-docker.sh --build"
