#!/bin/bash
# peek-logs.sh - Show recent log entries without following
#
# Usage: ./scripts/peek-logs.sh [component] [lines]
#
# Components: backend (default), frontend, postgres
# Lines: number of lines to show (default: 50)

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$PROJECT_ROOT/logs"

COMPONENT="${1:-backend}"
LINES="${2:-50}"

if [ ! -d "$LOG_DIR" ]; then
    echo "Log directory not found: $LOG_DIR"
    echo "Start the dev environment first: ./scripts/start-dev.sh"
    exit 1
fi

case "$COMPONENT" in
    backend|frontend|postgres)
        LOG_FILE="$LOG_DIR/${COMPONENT}_latest.log"
        if [ -f "$LOG_FILE" ]; then
            echo "=== Last $LINES lines of $COMPONENT log ==="
            echo ""
            tail -n "$LINES" "$LOG_FILE" | \
                sed -e 's/\(ERROR\)/\x1b[31m\1\x1b[0m/g' \
                    -e 's/\(WARN\)/\x1b[33m\1\x1b[0m/g' \
                    -e 's/\(INFO\)/\x1b[32m\1\x1b[0m/g'
        else
            echo "No log file found for $COMPONENT"
        fi
        ;;
    *)
        echo "Unknown component: $COMPONENT"
        echo "Usage: $0 [backend|frontend|postgres] [lines]"
        exit 1
        ;;
esac
