#!/bin/bash
# start-dev-docker.sh - Start all Track Leader development components via Docker Compose
#
# Usage: ./scripts/start-dev-docker.sh [options]
#
# Options:
#   --frontend-port PORT   Set frontend port (default: random)
#   --backend-port PORT    Set backend port (default: random)
#   --postgres-port PORT   Set PostgreSQL port (default: random)
#   --build                Force rebuild of images
#   --detach, -d           Run in detached mode (default: attached with logs)
#
# Creates isolated Docker containers for this workspace with random ports.
# Port configuration is saved to .dev-ports-docker

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Generate unique project name from workspace directory
WORKSPACE_NAME=$(basename "$PROJECT_ROOT")
COMPOSE_PROJECT_NAME="tl_${WORKSPACE_NAME}"

PORTS_FILE="$PROJECT_ROOT/.dev-ports-docker"

# Parse command line arguments
FRONTEND_PORT=""
BACKEND_PORT=""
POSTGRES_PORT=""
BUILD_FLAG=""
DETACH_FLAG=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --frontend-port)
            FRONTEND_PORT="$2"
            shift 2
            ;;
        --backend-port)
            BACKEND_PORT="$2"
            shift 2
            ;;
        --postgres-port)
            POSTGRES_PORT="$2"
            shift 2
            ;;
        --build)
            BUILD_FLAG="--build"
            shift
            ;;
        --detach|-d)
            DETACH_FLAG="-d"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --frontend-port PORT   Set frontend port (default: random)"
            echo "  --backend-port PORT    Set backend port (default: random)"
            echo "  --postgres-port PORT   Set PostgreSQL port (default: random)"
            echo "  --build                Force rebuild of images"
            echo "  --detach, -d           Run in detached mode"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Function to find an available port in range
find_available_port() {
    local start=${1:-10000}
    local end=${2:-60000}
    local port

    for _ in {1..100}; do
        port=$((RANDOM % (end - start + 1) + start))
        if ! lsof -i :$port > /dev/null 2>&1; then
            echo $port
            return 0
        fi
    done

    echo "Error: Could not find available port" >&2
    return 1
}

# Assign ports (use passed values or find random available ones)
if [ -z "$FRONTEND_PORT" ]; then
    FRONTEND_PORT=$(find_available_port 10000 19999)
fi
if [ -z "$BACKEND_PORT" ]; then
    BACKEND_PORT=$(find_available_port 20000 29999)
fi
if [ -z "$POSTGRES_PORT" ]; then
    POSTGRES_PORT=$(find_available_port 30000 39999)
fi

# Save port configuration
cat > "$PORTS_FILE" << EOF
# Track Leader Docker dev environment ports
# Generated: $(date)
COMPOSE_PROJECT_NAME=$COMPOSE_PROJECT_NAME
FRONTEND_PORT=$FRONTEND_PORT
BACKEND_PORT=$BACKEND_PORT
POSTGRES_PORT=$POSTGRES_PORT
EOF

echo "Starting Track Leader development environment (Docker)..."
echo "Project: $COMPOSE_PROJECT_NAME"
echo ""
echo "Ports:"
echo "  Frontend:   http://localhost:$FRONTEND_PORT"
echo "  Backend:    http://localhost:$BACKEND_PORT"
echo "  PostgreSQL: localhost:$POSTGRES_PORT"
echo ""

# Stop any existing containers for this project
docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml down 2>/dev/null || true

# Export environment variables for docker-compose
export COMPOSE_PROJECT_NAME
export FRONTEND_PORT
export BACKEND_PORT
export POSTGRES_PORT
# Internal ports stay fixed - external ports are what change
export FRONTEND_INTERNAL_PORT=3000
export BACKEND_INTERNAL_PORT=3001

# Start the services
if [ -n "$DETACH_FLAG" ]; then
    docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml up $BUILD_FLAG $DETACH_FLAG

    echo ""
    echo "Services started in detached mode."
    echo ""
    echo "Commands:"
    echo "  View logs:    ./scripts/logs-dev-docker.sh"
    echo "  Stop:         ./scripts/stop-dev-docker.sh"
    echo "  Status:       ./scripts/status-dev-docker.sh"
else
    echo "Starting services (Ctrl+C to stop)..."
    echo ""

    # Run attached - logs will stream to terminal
    docker compose -p "$COMPOSE_PROJECT_NAME" -f docker-compose.dev.yml up $BUILD_FLAG
fi
