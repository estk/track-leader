#!/bin/bash
# stop-dev.sh - Stop all Track Leader development components
#
# Usage: ./scripts/stop-dev.sh

# Add nix profile to PATH if it exists (for zellij)
[ -d "$HOME/.nix-profile/bin" ] && export PATH="$HOME/.nix-profile/bin:$PATH"

SESSION_NAME="track-leader"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Stopping Track Leader development environment..."

# Kill zellij session (strip ANSI codes for matching)
if zellij list-sessions 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^$SESSION_NAME"; then
    zellij delete-session "$SESSION_NAME" --force
    echo "Killed zellij session: $SESSION_NAME"
else
    echo "No zellij session found: $SESSION_NAME"
fi

# Stop docker containers
cd "$PROJECT_ROOT/crates/tracks"
if docker-compose ps -q postgres 2>/dev/null | grep -q .; then
    docker-compose stop postgres
    echo "Stopped PostgreSQL container"
else
    echo "PostgreSQL container not running"
fi

# Kill any stray processes on our ports
for port in 3000 3001; do
    pid=$(lsof -ti :$port 2>/dev/null || true)
    if [ -n "$pid" ]; then
        echo "Killing process on port $port (pid $pid)"
        kill $pid 2>/dev/null || true
    fi
done

echo "Done."
