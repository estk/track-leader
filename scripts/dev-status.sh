#!/bin/bash
# dev-status.sh - Show status of Track Leader development environment
#
# Usage: ./scripts/dev-status.sh

# Add nix profile to PATH if it exists (for zellij)
[ -d "$HOME/.nix-profile/bin" ] && export PATH="$HOME/.nix-profile/bin:$PATH"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORTS_FILE="$PROJECT_ROOT/.dev-ports"

echo "=== Track Leader Dev Environment Status ==="
echo ""

# Load port configuration if it exists
if [ -f "$PORTS_FILE" ]; then
    source "$PORTS_FILE"
    echo "Configuration loaded from: $PORTS_FILE"
else
    # Fallback to workspace-based session name
    WORKSPACE_NAME=$(basename "$PROJECT_ROOT")
    SESSION_NAME="tl-${WORKSPACE_NAME}"
    FRONTEND_PORT=""
    BACKEND_PORT=""
    POSTGRES_PORT=""
    POSTGRES_CONTAINER_NAME="tracks_postgres_${WORKSPACE_NAME}"
    echo "No port configuration found (dev environment not started?)"
fi
echo ""

# Check zellij session (strip ANSI codes for matching)
echo "Zellij Session:"
if zellij list-sessions 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^$SESSION_NAME"; then
    echo "  ✓ Session '$SESSION_NAME' is running"
else
    echo "  ✗ Session '$SESSION_NAME' not found"
fi
echo ""

# Check ports
echo "Service Ports:"
if [ -n "$FRONTEND_PORT" ]; then
    pid=$(lsof -ti :$FRONTEND_PORT 2>/dev/null || true)
    if [ -n "$pid" ]; then
        process=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
        echo "  ✓ Port $FRONTEND_PORT: Frontend (Next.js) (pid $pid, $process)"
    else
        echo "  ✗ Port $FRONTEND_PORT: Frontend not listening"
    fi
else
    echo "  ? Frontend port: unknown"
fi

if [ -n "$BACKEND_PORT" ]; then
    pid=$(lsof -ti :$BACKEND_PORT 2>/dev/null || true)
    if [ -n "$pid" ]; then
        process=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
        echo "  ✓ Port $BACKEND_PORT: Backend (Axum) (pid $pid, $process)"
    else
        echo "  ✗ Port $BACKEND_PORT: Backend not listening"
    fi
else
    echo "  ? Backend port: unknown"
fi

if [ -n "$POSTGRES_PORT" ]; then
    pid=$(lsof -ti :$POSTGRES_PORT 2>/dev/null || true)
    if [ -n "$pid" ]; then
        process=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
        echo "  ✓ Port $POSTGRES_PORT: PostgreSQL (pid $pid, $process)"
    else
        echo "  ✗ Port $POSTGRES_PORT: PostgreSQL not listening"
    fi
else
    echo "  ? PostgreSQL port: unknown"
fi
echo ""

# Check Docker
echo "Docker Container:"
if docker ps -q -f name="$POSTGRES_CONTAINER_NAME" 2>/dev/null | grep -q .; then
    status=$(docker ps -f name="$POSTGRES_CONTAINER_NAME" --format "{{.Status}}" 2>/dev/null)
    echo "  ✓ $POSTGRES_CONTAINER_NAME: $status"
else
    echo "  ✗ $POSTGRES_CONTAINER_NAME: not running"
fi
echo ""

# Check log files
echo "Latest Log Files:"
LOG_DIR="$PROJECT_ROOT/logs"
if [ -d "$LOG_DIR" ]; then
    for log in backend frontend postgres; do
        if [ -L "$LOG_DIR/${log}_latest.log" ]; then
            target=$(readlink "$LOG_DIR/${log}_latest.log")
            size=$(wc -c < "$LOG_DIR/${log}_latest.log" 2>/dev/null || echo "0")
            echo "  $log: $target ($(numfmt --to=iec $size 2>/dev/null || echo "${size}B"))"
        else
            echo "  $log: no log file"
        fi
    done
else
    echo "  Log directory not found"
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

echo "Commands:"
echo "  Start:   ./scripts/start-dev.sh"
echo "  Stop:    ./scripts/stop-dev.sh"
echo "  Attach:  ./scripts/attach-dev.sh"
echo "  Restart: ./scripts/restart-service.sh <backend|frontend|postgres|all>"
echo "  Logs:    ./scripts/watch-logs.sh [backend|frontend|postgres|all]"
