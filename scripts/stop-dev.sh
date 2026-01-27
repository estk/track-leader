#!/bin/bash
# stop-dev.sh - Stop all Track Leader development components
#
# Usage: ./scripts/stop-dev.sh

SESSION_NAME="track-leader"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Stopping Track Leader development environment..."

# Kill tmux session
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    tmux kill-session -t "$SESSION_NAME"
    echo "Killed tmux session: $SESSION_NAME"
else
    echo "No tmux session found: $SESSION_NAME"
fi

# Stop docker containers
cd "$PROJECT_ROOT/crates/tracks"
if docker-compose ps -q postgres 2>/dev/null | grep -q .; then
    docker-compose stop postgres
    echo "Stopped PostgreSQL container"
else
    echo "PostgreSQL container not running"
fi

echo "Done."
