#!/bin/bash
# dev-status.sh - Show status of Track Leader development environment
#
# Usage: ./scripts/dev-status.sh

# Add nix profile to PATH if it exists (for zellij)
[ -d "$HOME/.nix-profile/bin" ] && export PATH="$HOME/.nix-profile/bin:$PATH"

SESSION_NAME="track-leader"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=== Track Leader Dev Environment Status ==="
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
for port in 3000 3001 5432; do
    pid=$(lsof -ti :$port 2>/dev/null || true)
    if [ -n "$pid" ]; then
        process=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
        case $port in
            3000) service="Frontend (Next.js)" ;;
            3001) service="Backend (Axum)" ;;
            5432) service="PostgreSQL" ;;
        esac
        echo "  ✓ Port $port: $service (pid $pid, $process)"
    else
        case $port in
            3000) service="Frontend" ;;
            3001) service="Backend" ;;
            5432) service="PostgreSQL" ;;
        esac
        echo "  ✗ Port $port: $service not listening"
    fi
done
echo ""

# Check Docker
echo "Docker Containers:"
cd "$PROJECT_ROOT/crates/tracks"
if docker-compose ps -q postgres 2>/dev/null | grep -q .; then
    status=$(docker-compose ps postgres 2>/dev/null | tail -1 | awk '{print $NF}')
    echo "  ✓ PostgreSQL container: $status"
else
    echo "  ✗ PostgreSQL container not running"
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
if curl -s --max-time 2 http://localhost:3000 > /dev/null 2>&1; then
    echo "  ✓ Frontend responding on http://localhost:3000"
else
    echo "  ✗ Frontend not responding"
fi

if curl -s --max-time 2 http://localhost:3001/health > /dev/null 2>&1; then
    echo "  ✓ Backend responding on http://localhost:3001"
elif curl -s --max-time 2 http://localhost:3001 > /dev/null 2>&1; then
    echo "  ✓ Backend responding on http://localhost:3001 (no /health endpoint)"
else
    echo "  ✗ Backend not responding"
fi
echo ""

echo "Commands:"
echo "  Start:   ./scripts/start-dev.sh"
echo "  Stop:    ./scripts/stop-dev.sh"
echo "  Attach:  ./scripts/attach-dev.sh"
echo "  Restart: ./scripts/restart-service.sh <backend|frontend|postgres|all>"
echo "  Logs:    ./scripts/watch-logs.sh [backend|frontend|postgres|all]"
