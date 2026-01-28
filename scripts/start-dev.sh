#!/bin/bash
# start-dev.sh - Start all Track Leader development components in zellij
#
# Usage: ./scripts/start-dev.sh
#
# Creates a zellij session with 3 panes:
#   - PostgreSQL (docker)
#   - Backend (Rust/Axum on port 3001)
#   - Frontend (Next.js on port 3000)
#
# Logs are saved to ./logs/ directory

set -e

# Add nix profile to PATH if it exists (for zellij)
[ -d "$HOME/.nix-profile/bin" ] && export PATH="$HOME/.nix-profile/bin:$PATH"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

SESSION_NAME="track-leader"
LOG_DIR="$PROJECT_ROOT/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

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

echo "Starting Track Leader development environment..."
echo "Logs will be saved to: $LOG_DIR"
echo "  - PostgreSQL: $POSTGRES_LOG"
echo "  - Backend:    $BACKEND_LOG"
echo "  - Frontend:   $FRONTEND_LOG"
echo ""

# Kill existing session if it exists
zellij delete-session "$SESSION_NAME" --force 2>/dev/null || true

# Kill any stray processes on our ports
for port in 3000 3001; do
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

layout {
    pane size=1 borderless=true {
        plugin location="tab-bar"
    }

    pane split_direction="vertical" {
        // PostgreSQL pane (narrower, on the left)
        pane size="30%" name="postgres" {
            command "bash"
            args "-c" "echo '=== PostgreSQL ===' && cd '$PROJECT_ROOT/crates/tracks' && docker-compose up postgres 2>&1 | tee '$POSTGRES_LOG'"
        }

        // Right side split horizontally
        pane split_direction="horizontal" {
            // Backend pane
            pane name="backend" {
                command "bash"
                args "-c" "echo '=== Backend (port 3001) ===' && echo 'Waiting for PostgreSQL...' && sleep 3 && cd '$PROJECT_ROOT/crates/tracks' && RUST_LOG=info DATABASE_URL='postgres://tracks_user:tracks_password@localhost:5432/tracks_db' cargo run 2>&1 | tee '$BACKEND_LOG'"
            }

            // Frontend pane
            pane name="frontend" {
                command "bash"
                args "-c" "echo '=== Frontend (port 3000) ===' && echo 'Waiting for backend...' && sleep 5 && cd '$PROJECT_ROOT' && npm run dev 2>&1 | tee '$FRONTEND_LOG'"
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
