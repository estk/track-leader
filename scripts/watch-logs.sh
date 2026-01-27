#!/bin/bash
# watch-logs.sh - Watch Track Leader logs in real-time
#
# Usage: ./scripts/watch-logs.sh [component]
#
# Components: all (default), backend, frontend, postgres

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$PROJECT_ROOT/logs"

COMPONENT="${1:-all}"

if [ ! -d "$LOG_DIR" ]; then
    echo "Log directory not found: $LOG_DIR"
    echo "Start the dev environment first: ./scripts/start-dev.sh"
    exit 1
fi

case "$COMPONENT" in
    backend)
        echo "Watching backend logs..."
        tail -f "$LOG_DIR/backend_latest.log"
        ;;
    frontend)
        echo "Watching frontend logs..."
        tail -f "$LOG_DIR/frontend_latest.log"
        ;;
    postgres)
        echo "Watching PostgreSQL logs..."
        tail -f "$LOG_DIR/postgres_latest.log"
        ;;
    all)
        echo "Watching all logs (backend errors highlighted)..."
        echo "Press Ctrl+C to stop"
        echo ""
        tail -f "$LOG_DIR/backend_latest.log" "$LOG_DIR/frontend_latest.log" 2>/dev/null | \
            sed -e 's/\(ERROR\)/\x1b[31m\1\x1b[0m/g' \
                -e 's/\(WARN\)/\x1b[33m\1\x1b[0m/g' \
                -e 's/\(INFO\)/\x1b[32m\1\x1b[0m/g'
        ;;
    *)
        echo "Unknown component: $COMPONENT"
        echo "Usage: $0 [all|backend|frontend|postgres]"
        exit 1
        ;;
esac
