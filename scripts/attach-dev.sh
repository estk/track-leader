#!/bin/bash
# attach-dev.sh - Attach to the Track Leader zellij session
#
# Usage: ./scripts/attach-dev.sh

# Add nix profile to PATH if it exists (for zellij)
[ -d "$HOME/.nix-profile/bin" ] && export PATH="$HOME/.nix-profile/bin:$PATH"

SESSION_NAME="track-leader"

if ! zellij list-sessions 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^$SESSION_NAME"; then
    echo "No zellij session found: $SESSION_NAME"
    echo ""
    echo "Start it with: ./scripts/start-dev.sh"
    exit 1
fi

exec zellij attach "$SESSION_NAME"
