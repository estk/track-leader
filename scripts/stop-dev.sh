#!/bin/bash
# stop-dev.sh - Stop all Track Leader development components
#
# Usage: ./scripts/stop-dev.sh

# Add nix profile to PATH if it exists (for zellij)
[ -d "$HOME/.nix-profile/bin" ] && export PATH="$HOME/.nix-profile/bin:$PATH"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORTS_FILE="$PROJECT_ROOT/.dev-ports"

# Load port configuration if it exists
if [ -f "$PORTS_FILE" ]; then
    source "$PORTS_FILE"
else
    # Fallback to workspace-based session name
    WORKSPACE_NAME=$(basename "$PROJECT_ROOT")
    SESSION_NAME="tl-${WORKSPACE_NAME}"
    FRONTEND_PORT=""
    BACKEND_PORT=""
    POSTGRES_CONTAINER_NAME="tracks_postgres_${WORKSPACE_NAME}"
fi

echo "Stopping Track Leader development environment..."
echo "Session: $SESSION_NAME"

# Kill zellij session (strip ANSI codes for matching)
if zellij list-sessions 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^$SESSION_NAME"; then
    zellij delete-session "$SESSION_NAME" --force
    echo "Killed zellij session: $SESSION_NAME"
else
    echo "No zellij session found: $SESSION_NAME"
fi

# Stop docker container
if docker ps -q -f name="$POSTGRES_CONTAINER_NAME" 2>/dev/null | grep -q .; then
    docker stop "$POSTGRES_CONTAINER_NAME"
    echo "Stopped PostgreSQL container: $POSTGRES_CONTAINER_NAME"
else
    echo "PostgreSQL container not running: $POSTGRES_CONTAINER_NAME"
fi

# Kill any stray processes on our ports (if we know them)
if [ -n "$FRONTEND_PORT" ] && [ -n "$BACKEND_PORT" ]; then
    for port in $FRONTEND_PORT $BACKEND_PORT; do
        pid=$(lsof -ti :$port 2>/dev/null || true)
        if [ -n "$pid" ]; then
            echo "Killing process on port $port (pid $pid)"
            kill $pid 2>/dev/null || true
        fi
    done
fi

# Clean up ports file
rm -f "$PORTS_FILE"

echo "Done."
