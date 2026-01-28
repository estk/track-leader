#!/bin/bash
# restart-service.sh - Restart a specific service in the zellij session
#
# Usage: ./scripts/restart-service.sh <service>
#
# Services: backend, frontend, postgres, all

set -e

# Add nix profile to PATH if it exists (for zellij)
[ -d "$HOME/.nix-profile/bin" ] && export PATH="$HOME/.nix-profile/bin:$PATH"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SESSION_NAME="track-leader"
LOG_DIR="$PROJECT_ROOT/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

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

# Check if session exists (strip ANSI codes for matching)
if ! zellij list-sessions 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^$SESSION_NAME"; then
    echo "Error: zellij session '$SESSION_NAME' not found"
    echo "Start it with: ./scripts/start-dev.sh"
    exit 1
fi

restart_backend() {
    echo "Restarting backend..."

    # Create new log file
    BACKEND_LOG="$LOG_DIR/backend_${TIMESTAMP}.log"
    ln -sf "backend_${TIMESTAMP}.log" "$LOG_DIR/backend_latest.log"

    # Kill process on port 3001
    pid=$(lsof -ti :3001 2>/dev/null || true)
    if [ -n "$pid" ]; then
        kill $pid 2>/dev/null || true
        sleep 0.5
    fi

    # Use zellij action to run command in the backend pane
    # First focus the pane by name, then close it and rerun
    zellij --session "$SESSION_NAME" action write-chars $'\x03'  # Ctrl+C in current pane
    sleep 0.5

    # Run the backend command
    zellij --session "$SESSION_NAME" action focus-pane-in-tab "backend" 2>/dev/null || true

    CMD="cd '$PROJECT_ROOT/crates/tracks' && RUST_LOG=info DATABASE_URL='postgres://tracks_user:tracks_password@localhost:5432/tracks_db' cargo run 2>&1 | tee '$BACKEND_LOG'"
    zellij --session "$SESSION_NAME" action write-chars "$CMD"
    zellij --session "$SESSION_NAME" action write-chars $'\n'

    echo "Backend restarting. Logs: $BACKEND_LOG"
}

restart_frontend() {
    echo "Restarting frontend..."

    # Create new log file
    FRONTEND_LOG="$LOG_DIR/frontend_${TIMESTAMP}.log"
    ln -sf "frontend_${TIMESTAMP}.log" "$LOG_DIR/frontend_latest.log"

    # Kill process on port 3000
    pid=$(lsof -ti :3000 2>/dev/null || true)
    if [ -n "$pid" ]; then
        kill $pid 2>/dev/null || true
        sleep 0.5
    fi

    zellij --session "$SESSION_NAME" action focus-pane-in-tab "frontend" 2>/dev/null || true
    zellij --session "$SESSION_NAME" action write-chars $'\x03'  # Ctrl+C
    sleep 0.5

    CMD="cd '$PROJECT_ROOT' && npm run dev 2>&1 | tee '$FRONTEND_LOG'"
    zellij --session "$SESSION_NAME" action write-chars "$CMD"
    zellij --session "$SESSION_NAME" action write-chars $'\n'

    echo "Frontend restarting. Logs: $FRONTEND_LOG"
}

restart_postgres() {
    echo "Restarting PostgreSQL..."

    # Create new log file
    POSTGRES_LOG="$LOG_DIR/postgres_${TIMESTAMP}.log"
    ln -sf "postgres_${TIMESTAMP}.log" "$LOG_DIR/postgres_latest.log"

    # Stop container first
    cd "$PROJECT_ROOT/crates/tracks"
    docker-compose stop postgres 2>/dev/null || true
    sleep 1

    zellij --session "$SESSION_NAME" action focus-pane-in-tab "postgres" 2>/dev/null || true
    zellij --session "$SESSION_NAME" action write-chars $'\x03'  # Ctrl+C
    sleep 0.5

    CMD="cd '$PROJECT_ROOT/crates/tracks' && docker-compose up postgres 2>&1 | tee '$POSTGRES_LOG'"
    zellij --session "$SESSION_NAME" action write-chars "$CMD"
    zellij --session "$SESSION_NAME" action write-chars $'\n'

    echo "PostgreSQL restarting. Logs: $POSTGRES_LOG"
}

case "$SERVICE" in
    backend)
        restart_backend
        ;;
    frontend)
        restart_frontend
        ;;
    postgres)
        restart_postgres
        ;;
    all)
        restart_postgres
        echo "Waiting for PostgreSQL to start..."
        sleep 3
        restart_backend
        echo "Waiting for backend to start..."
        sleep 3
        restart_frontend
        ;;
    *)
        echo "Unknown service: $SERVICE"
        echo "Valid services: backend, frontend, postgres, all"
        exit 1
        ;;
esac

echo "Done."
