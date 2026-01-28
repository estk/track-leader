#!/bin/bash
# attach-dev.sh - Attach to the Track Leader zellij session
#
# Usage: ./scripts/attach-dev.sh

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
fi

# Check if session exists (strip ANSI codes for matching)
if ! zellij list-sessions 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^$SESSION_NAME"; then
    echo "Error: zellij session '$SESSION_NAME' not found"
    echo "Start it with: ./scripts/start-dev.sh"
    exit 1
fi

echo "Attaching to session: $SESSION_NAME"
echo "Detach with: Ctrl+o, then d"
echo ""

exec zellij attach "$SESSION_NAME"
