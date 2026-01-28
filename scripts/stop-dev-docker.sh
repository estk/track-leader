#!/bin/bash
# stop-dev-docker.sh - Stop all Track Leader Docker development components
#
# Usage: ./scripts/stop-dev-docker.sh [--volumes]
#
# Options:
#   --volumes    Also remove volumes (database data, caches)

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORTS_FILE="$PROJECT_ROOT/.dev-ports-docker"

# Parse arguments
VOLUMES_FLAG=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --volumes|-v)
            VOLUMES_FLAG="--volumes"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

# Load port configuration if it exists
if [ -f "$PORTS_FILE" ]; then
    source "$PORTS_FILE"
else
    # Fallback to workspace-based project name
    WORKSPACE_NAME=$(basename "$PROJECT_ROOT")
    COMPOSE_PROJECT_NAME="tl_${WORKSPACE_NAME}"
fi

echo "Stopping Track Leader Docker development environment..."
echo "Project: $COMPOSE_PROJECT_NAME"

cd "$PROJECT_ROOT"

# Stop and remove containers
docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml down $VOLUMES_FLAG

if [ -n "$VOLUMES_FLAG" ]; then
    echo "Volumes removed."
fi

# Clean up ports file
rm -f "$PORTS_FILE"

echo "Done."
