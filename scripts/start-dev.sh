#!/bin/bash
# start-dev.sh - Start all Track Leader development components in tmux
#
# Usage: ./scripts/start-dev.sh
#
# Creates a tmux session with 3 panes:
#   - PostgreSQL (docker)
#   - Backend (Rust/Axum on port 3001)
#   - Frontend (Next.js on port 3000)
#
# Logs are saved to ./logs/ directory

set -e

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
tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

# Create new tmux session with PostgreSQL pane
tmux new-session -d -s "$SESSION_NAME" -n "dev" \
    "echo '=== PostgreSQL ===' && cd '$PROJECT_ROOT/crates/tracks' && docker-compose up postgres 2>&1 | tee '$POSTGRES_LOG'"

# Wait a moment for tmux to initialize
sleep 0.5

# Split horizontally for backend
tmux split-window -h -t "$SESSION_NAME:dev" \
    "echo '=== Backend (port 3001) ===' && echo 'Waiting for PostgreSQL...' && sleep 3 && cd '$PROJECT_ROOT/crates/tracks' && RUST_LOG=info DATABASE_URL='postgres://tracks_user:tracks_password@localhost:5432/tracks_db' cargo run 2>&1 | tee '$BACKEND_LOG'"

# Split the right pane vertically for frontend
tmux split-window -v -t "$SESSION_NAME:dev.1" \
    "echo '=== Frontend (port 3000) ===' && echo 'Waiting for backend...' && sleep 5 && cd '$PROJECT_ROOT' && npm run dev 2>&1 | tee '$FRONTEND_LOG'"

# Keep panes alive after process exits (allows Ctrl-C then respawn)
tmux set-option -t "$SESSION_NAME" remain-on-exit on

# Adjust pane sizes (make left pane narrower for postgres)
tmux select-layout -t "$SESSION_NAME:dev" main-vertical

# Set pane titles (requires tmux 2.6+)
tmux select-pane -t "$SESSION_NAME:dev.0" -T "PostgreSQL"
tmux select-pane -t "$SESSION_NAME:dev.1" -T "Backend"
tmux select-pane -t "$SESSION_NAME:dev.2" -T "Frontend"

# Enable pane border status to show titles
tmux set-option -t "$SESSION_NAME" pane-border-status top
tmux set-option -t "$SESSION_NAME" pane-border-format "#{pane_title}"

# Select the backend pane by default
tmux select-pane -t "$SESSION_NAME:dev.1"

echo "tmux session '$SESSION_NAME' created!"
echo ""
echo "To attach: tmux attach -t $SESSION_NAME"
echo "To detach: Ctrl+b, then d"
echo "To kill:   tmux kill-session -t $SESSION_NAME"
echo ""
echo "Monitor logs with:"
echo "  tail -f $LOG_DIR/backend_latest.log"
echo "  tail -f $LOG_DIR/frontend_latest.log"
echo ""

# Attach to the session
tmux attach -t "$SESSION_NAME"
