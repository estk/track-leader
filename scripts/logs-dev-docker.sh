#!/bin/bash
# logs-dev-docker.sh - View logs from Track Leader Docker development environment
#
# Usage: ./scripts/logs-dev-docker.sh [service] [options]
#
# Services: backend, frontend, postgres (default: all)
# Options:
#   -f, --follow    Follow log output (default)
#   --tail N        Number of lines to show from end (default: 100)
#   --no-follow     Don't follow, just show current logs

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORTS_FILE="$PROJECT_ROOT/.dev-ports-docker"

# Load port configuration if it exists
if [ -f "$PORTS_FILE" ]; then
    source "$PORTS_FILE"
else
    # Fallback to workspace-based project name
    WORKSPACE_NAME=$(basename "$PROJECT_ROOT")
    COMPOSE_PROJECT_NAME="tl_${WORKSPACE_NAME}"
fi

cd "$PROJECT_ROOT"

# Parse arguments
SERVICE=""
FOLLOW="-f"
TAIL="--tail=100"

while [[ $# -gt 0 ]]; do
    case $1 in
        backend|frontend|postgres)
            SERVICE="$1"
            shift
            ;;
        -f|--follow)
            FOLLOW="-f"
            shift
            ;;
        --no-follow)
            FOLLOW=""
            shift
            ;;
        --tail)
            TAIL="--tail=$2"
            shift 2
            ;;
        --tail=*)
            TAIL="$1"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [service] [options]"
            echo ""
            echo "Services: backend, frontend, postgres (default: all)"
            echo ""
            echo "Options:"
            echo "  -f, --follow    Follow log output (default)"
            echo "  --tail N        Number of lines to show from end (default: 100)"
            echo "  --no-follow     Don't follow, just show current logs"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "Viewing logs for: ${SERVICE:-all services}"
echo "Project: $COMPOSE_PROJECT_NAME"
echo ""

docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml logs $FOLLOW $TAIL $SERVICE
