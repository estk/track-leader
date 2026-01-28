#!/bin/bash
# start-dev.sh - Start all Track Leader development components in zellij
#
# Usage: ./scripts/start-dev.sh [options]
#
# Options:
#   --frontend-port PORT   Set frontend port (default: random)
#   --backend-port PORT    Set backend port (default: random)
#   --postgres-port PORT   Set PostgreSQL port (default: random)
#
# Creates a zellij session with 3 panes:
#   - PostgreSQL (docker)
#   - Backend (Rust/Axum)
#   - Frontend (Next.js)
#
# Logs are saved to ./logs/ directory
# Port configuration is saved to .dev-ports

set -e

# Add nix profile to PATH if it exists (for zellij)
[ -d "$HOME/.nix-profile/bin" ] && export PATH="$HOME/.nix-profile/bin:$PATH"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Generate unique session name from workspace directory
WORKSPACE_NAME=$(basename "$PROJECT_ROOT")
SESSION_NAME="tl-${WORKSPACE_NAME}"

LOG_DIR="$PROJECT_ROOT/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
PORTS_FILE="$PROJECT_ROOT/.dev-ports"

# Parse command line arguments
FRONTEND_PORT=""
BACKEND_PORT=""
POSTGRES_PORT=""

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
        -h|--help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --frontend-port PORT   Set frontend port (default: random)"
            echo "  --backend-port PORT    Set backend port (default: random)"
            echo "  --postgres-port PORT   Set PostgreSQL port (default: random)"
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

# Unique names for this session's Docker resources
POSTGRES_CONTAINER_NAME="tracks_postgres_${WORKSPACE_NAME}"
POSTGRES_VOLUME_NAME="tracks_pgdata_${WORKSPACE_NAME}"

# Create logs directory
mkdir -p "$LOG_DIR"

# Clean up old log files (keep last 5)
ls -t "$LOG_DIR"/postgres_*.log 2>/dev/null | tail -n +6 | xargs -r rm -f
ls -t "$LOG_DIR"/backend_*.log 2>/dev/null | tail -n +6 | xargs -r rm -f
ls -t "$LOG_DIR"/frontend_*.log 2>/dev/null | tail -n +6 | xargs -r rm -f

# Log file paths for this session
POSTGRES_LOG="$LOG_DIR/postgres_${TIMESTAMP}.log"
BACKEND_LOG="$LOG_DIR/backend_${TIMESTAMP}.log"
FRONTEND_LOG="$LOG_DIR/frontend_${TIMESTAMP}.log"

# Create symlinks to latest logs
ln -sf "postgres_${TIMESTAMP}.log" "$LOG_DIR/postgres_latest.log"
ln -sf "backend_${TIMESTAMP}.log" "$LOG_DIR/backend_latest.log"
ln -sf "frontend_${TIMESTAMP}.log" "$LOG_DIR/frontend_latest.log"

# Save port configuration
cat > "$PORTS_FILE" << EOF
# Track Leader dev environment ports
# Generated: $(date)
SESSION_NAME=$SESSION_NAME
FRONTEND_PORT=$FRONTEND_PORT
BACKEND_PORT=$BACKEND_PORT
POSTGRES_PORT=$POSTGRES_PORT
POSTGRES_CONTAINER_NAME=$POSTGRES_CONTAINER_NAME
POSTGRES_VOLUME_NAME=$POSTGRES_VOLUME_NAME
EOF

echo "Starting Track Leader development environment..."
echo "Session: $SESSION_NAME"
echo ""
echo "Ports:"
echo "  Frontend:   http://localhost:$FRONTEND_PORT"
echo "  Backend:    http://localhost:$BACKEND_PORT"
echo "  PostgreSQL: localhost:$POSTGRES_PORT"
echo ""
echo "Logs: $LOG_DIR"
echo "  - PostgreSQL: $POSTGRES_LOG"
echo "  - Backend:    $BACKEND_LOG"
echo "  - Frontend:   $FRONTEND_LOG"
echo ""

# Kill existing session if it exists
zellij delete-session "$SESSION_NAME" --force 2>/dev/null || true

# Kill any stray processes on our ports
for port in $FRONTEND_PORT $BACKEND_PORT; do
    pid=$(lsof -ti :$port 2>/dev/null || true)
    if [ -n "$pid" ]; then
        echo "Killing process on port $port (pid $pid)"
        kill $pid 2>/dev/null || true
        sleep 0.5
    fi
done

# Generate the layout file with actual paths embedded
LAYOUT_FILE="$PROJECT_ROOT/scripts/.dev-layout-generated.kdl"

cat > "$LAYOUT_FILE" << EOF
// Auto-generated zellij layout for Track Leader
// Generated at: $(date)
// Session: $SESSION_NAME

layout {
    pane size=1 borderless=true {
        plugin location="tab-bar"
    }

    pane split_direction="vertical" {
        // PostgreSQL pane (narrower, on the left)
        pane size="30%" name="postgres" {
            command "bash"
            args "-c" "echo '=== PostgreSQL (port $POSTGRES_PORT) ===' && cd '$PROJECT_ROOT/crates/tracks' && POSTGRES_PORT=$POSTGRES_PORT POSTGRES_CONTAINER_NAME=$POSTGRES_CONTAINER_NAME POSTGRES_VOLUME_NAME=$POSTGRES_VOLUME_NAME docker-compose up postgres 2>&1 | tee '$POSTGRES_LOG'"
        }

        // Right side split horizontally
        pane split_direction="horizontal" {
            // Backend pane
            pane name="backend" {
                command "bash"
                args "-c" "echo '=== Backend (port $BACKEND_PORT) ===' && echo 'Waiting for PostgreSQL...' && sleep 3 && cd '$PROJECT_ROOT/crates/tracks' && RUST_LOG=info PORT=$BACKEND_PORT DATABASE_URL='postgres://tracks_user:tracks_password@localhost:$POSTGRES_PORT/tracks_db' cargo run 2>&1 | tee '$BACKEND_LOG'"
            }

            // Frontend pane
            pane name="frontend" {
                command "bash"
                args "-c" "echo '=== Frontend (port $FRONTEND_PORT) ===' && echo 'Waiting for backend...' && sleep 5 && cd '$PROJECT_ROOT' && PORT=$FRONTEND_PORT BACKEND_PORT=$BACKEND_PORT npm run dev 2>&1 | tee '$FRONTEND_LOG'"
            }
        }
    }

    pane size=2 borderless=true {
        plugin location="status-bar"
    }
}
EOF

# Create background session
zellij attach -b -c "$SESSION_NAME"
sleep 0.5

# Load layout into the session (creates new tab)
ZELLIJ_SESSION_NAME="$SESSION_NAME" zellij action new-tab -l "$LAYOUT_FILE" -n "dev"

# Close the initial empty tab
ZELLIJ_SESSION_NAME="$SESSION_NAME" zellij action go-to-tab 1
ZELLIJ_SESSION_NAME="$SESSION_NAME" zellij action close-tab

echo "zellij session '$SESSION_NAME' created!"
echo ""
echo "To attach: zellij attach $SESSION_NAME"
echo "           or: ./scripts/attach-dev.sh"
echo "To detach: Ctrl+o, then d"
echo "To kill:   ./scripts/stop-dev.sh"
echo ""
echo "Restart services with:"
echo "  ./scripts/restart-service.sh backend"
echo "  ./scripts/restart-service.sh frontend"
echo "  ./scripts/restart-service.sh postgres"
echo ""
echo "Monitor logs with:"
echo "  ./scripts/watch-logs.sh"
echo "  tail -f $LOG_DIR/backend_latest.log"
echo "  tail -f $LOG_DIR/frontend_latest.log"
