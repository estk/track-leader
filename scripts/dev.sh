#!/bin/bash
# dev.sh - Unified Track Leader development environment management
#
# Usage: ./scripts/dev.sh [command] [options]
#
# Commands:
#   (none)           Start in foreground with logs streaming (default)
#   status           Show status of all services with health
#   logs [service]   Show logs (all or specific service: backend, frontend, postgres)
#   stop             Stop all services
#   restart [svc]    Restart specific service or all
#
# Options:
#   -d, --detach           Start in detached mode
#   --build                Force rebuild of images
#   --frontend-port PORT   Set frontend port (default: random)
#   --backend-port PORT    Set backend port (default: random)
#   --postgres-port PORT   Set PostgreSQL port (default: random)
#   --volumes              With stop: also remove volumes (clears database)
#   --tail N               With logs: number of lines (default: 100)
#   --no-follow            With logs: don't follow, just show current logs
#
# Examples:
#   ./scripts/dev.sh                    # Start in foreground
#   ./scripts/dev.sh -d                 # Start detached
#   ./scripts/dev.sh status             # Check service health
#   ./scripts/dev.sh logs backend       # Follow backend logs
#   ./scripts/dev.sh restart backend    # Restart backend service
#   ./scripts/dev.sh stop               # Stop everything

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Generate unique project name from workspace directory
WORKSPACE_NAME=$(basename "$PROJECT_ROOT")
COMPOSE_PROJECT_NAME="tl_${WORKSPACE_NAME}"

PORTS_FILE="$PROJECT_ROOT/.dev-ports"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.dev.yml"

# Default values
COMMAND=""
DETACH_FLAG=""
BUILD_FLAG=""
VOLUMES_FLAG=""
FRONTEND_PORT=""
BACKEND_PORT=""
POSTGRES_PORT=""
LOG_FOLLOW="-f"
LOG_TAIL="--tail=100"
SERVICE=""

# Function to find an available port in range
find_available_port() {
    local start=${1:-10000}
    local end=${2:-60000}
    local port

    for _ in {1..100}; do
        port=$((RANDOM % (end - start + 1) + start))
        if ! lsof -i :"$port" > /dev/null 2>&1; then
            echo "$port"
            return 0
        fi
    done

    echo "Error: Could not find available port" >&2
    return 1
}

# Load port configuration if it exists
load_ports() {
    if [ -f "$PORTS_FILE" ]; then
        # shellcheck source=/dev/null
        source "$PORTS_FILE"
    else
        COMPOSE_PROJECT_NAME="tl_${WORKSPACE_NAME}"
    fi
}

# Show help
show_help() {
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  (none)           Start in foreground with logs streaming (default)"
    echo "  status           Show status of all services with health"
    echo "  logs [service]   Show logs (all or specific service)"
    echo "  stop             Stop all services"
    echo "  restart [svc]    Restart specific service or all"
    echo ""
    echo "Options:"
    echo "  -d, --detach           Start in detached mode"
    echo "  --build                Force rebuild of images"
    echo "  --frontend-port PORT   Set frontend port (default: random)"
    echo "  --backend-port PORT    Set backend port (default: random)"
    echo "  --postgres-port PORT   Set PostgreSQL port (default: random)"
    echo "  --volumes              With stop: also remove volumes"
    echo "  --tail N               With logs: number of lines (default: 100)"
    echo "  --no-follow            With logs: don't follow"
    echo ""
    echo "Services: backend, frontend, postgres"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            status|logs|stop|restart)
                COMMAND="$1"
                shift
                ;;
            backend|frontend|postgres|all)
                SERVICE="$1"
                shift
                ;;
            -d|--detach)
                DETACH_FLAG="-d"
                shift
                ;;
            --build)
                BUILD_FLAG="--build"
                shift
                ;;
            --volumes|-v)
                VOLUMES_FLAG="--volumes"
                shift
                ;;
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
            --tail)
                LOG_TAIL="--tail=$2"
                shift 2
                ;;
            --tail=*)
                LOG_TAIL="$1"
                shift
                ;;
            --no-follow)
                LOG_FOLLOW=""
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Start services
cmd_start() {
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

    echo "Starting Track Leader development environment..."
    echo "Project: $COMPOSE_PROJECT_NAME"
    echo ""
    echo "Ports:"
    echo "  Frontend:   http://localhost:$FRONTEND_PORT"
    echo "  Backend:    http://localhost:$BACKEND_PORT"
    echo "  PostgreSQL: localhost:$POSTGRES_PORT"
    echo ""

    # Stop any existing containers for this project
    docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" down 2>/dev/null || true

    # Export environment variables for docker-compose
    export COMPOSE_PROJECT_NAME
    export FRONTEND_PORT
    export BACKEND_PORT
    export POSTGRES_PORT
    export FRONTEND_INTERNAL_PORT=3000
    export BACKEND_INTERNAL_PORT=3001

    # Start the services
    if [ -n "$DETACH_FLAG" ]; then
        docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" up $BUILD_FLAG $DETACH_FLAG

        echo ""
        echo "Services started in detached mode."
        echo ""
        echo "Commands:"
        echo "  View status:  ./scripts/dev.sh status"
        echo "  View logs:    ./scripts/dev.sh logs"
        echo "  Stop:         ./scripts/dev.sh stop"
    else
        echo "Starting services (Ctrl+C to stop)..."
        echo ""
        docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" up $BUILD_FLAG
    fi
}

# Show status
cmd_status() {
    load_ports

    echo "=== Track Leader Dev Environment Status ==="
    echo ""
    echo "Project: $COMPOSE_PROJECT_NAME"
    echo ""

    # Check containers with health status
    echo "Containers:"
    docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" ps 2>/dev/null || echo "  No containers found"
    echo ""

    # Show configured ports
    if [ -f "$PORTS_FILE" ]; then
        echo "Configured Ports:"
        echo "  Frontend:   http://localhost:$FRONTEND_PORT"
        echo "  Backend:    http://localhost:$BACKEND_PORT"
        echo "  PostgreSQL: localhost:$POSTGRES_PORT"
        echo ""

        # Quick health check via HTTP
        echo "HTTP Health Check:"
        if curl -s --max-time 2 "http://localhost:$FRONTEND_PORT" > /dev/null 2>&1; then
            echo "  Frontend:  healthy (responding)"
        else
            echo "  Frontend:  not responding"
        fi

        if curl -s --max-time 2 "http://localhost:$BACKEND_PORT/health" > /dev/null 2>&1; then
            echo "  Backend:   healthy (responding)"
        else
            echo "  Backend:   not responding"
        fi
        echo ""
    else
        echo "No port configuration found. Dev environment not started?"
        echo ""
    fi

    # Volumes
    echo "Volumes:"
    docker volume ls --filter "name=${COMPOSE_PROJECT_NAME}_" --format "  {{.Name}}" 2>/dev/null || echo "  No volumes found"
    echo ""

    echo "Commands:"
    echo "  Start:   ./scripts/dev.sh"
    echo "  Stop:    ./scripts/dev.sh stop"
    echo "  Logs:    ./scripts/dev.sh logs [backend|frontend|postgres]"
    echo "  Restart: ./scripts/dev.sh restart [backend|frontend|postgres|all]"
}

# Show logs
cmd_logs() {
    load_ports

    echo "Viewing logs for: ${SERVICE:-all services}"
    echo "Project: $COMPOSE_PROJECT_NAME"
    echo ""

    docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" logs $LOG_FOLLOW $LOG_TAIL $SERVICE
}

# Stop services
cmd_stop() {
    load_ports

    echo "Stopping Track Leader Docker development environment..."
    echo "Project: $COMPOSE_PROJECT_NAME"

    docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" down $VOLUMES_FLAG

    if [ -n "$VOLUMES_FLAG" ]; then
        echo "Volumes removed."
    fi

    # Clean up ports file
    rm -f "$PORTS_FILE"

    echo "Done."
}

# Restart services
cmd_restart() {
    load_ports

    if [ -z "$SERVICE" ]; then
        echo "Usage: $0 restart <service>"
        echo ""
        echo "Services:"
        echo "  backend   - Restart the Rust/Axum backend"
        echo "  frontend  - Restart the Next.js frontend"
        echo "  postgres  - Restart PostgreSQL container"
        echo "  all       - Restart all services"
        exit 1
    fi

    # Export environment variables needed for docker compose
    export COMPOSE_PROJECT_NAME
    export FRONTEND_PORT
    export BACKEND_PORT
    export POSTGRES_PORT
    export FRONTEND_INTERNAL_PORT=3000
    export BACKEND_INTERNAL_PORT=3001

    case "$SERVICE" in
        backend|frontend|postgres)
            echo "Restarting $SERVICE..."
            docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" restart "$SERVICE"
            ;;
        all)
            echo "Restarting all services..."
            docker compose -p "$COMPOSE_PROJECT_NAME" -f "$COMPOSE_FILE" restart
            ;;
        *)
            echo "Unknown service: $SERVICE"
            echo "Valid services: backend, frontend, postgres, all"
            exit 1
            ;;
    esac

    echo "Done."
}

# Main
parse_args "$@"

case "$COMMAND" in
    "")
        cmd_start
        ;;
    status)
        cmd_status
        ;;
    logs)
        cmd_logs
        ;;
    stop)
        cmd_stop
        ;;
    restart)
        cmd_restart
        ;;
    *)
        echo "Unknown command: $COMMAND"
        show_help
        exit 1
        ;;
esac
