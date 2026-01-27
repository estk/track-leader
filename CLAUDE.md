# CLAUDE.md

## Development Environment

**IMPORTANT:** Always use the `start-dev.sh` script to start the development environment. Never start components individually with `npm run dev` or `cargo run` directly.

### Starting the Dev Environment

```bash
# From project root - creates detached tmux session and returns immediately
./scripts/start-dev.sh
```

The script creates a tmux session named `track-leader` with 3 panes:
- **Pane 0 (left):** PostgreSQL via docker-compose
- **Pane 1 (top-right):** Rust/Axum backend on port 3001
- **Pane 2 (bottom-right):** Next.js frontend on port 3000

### Checking Logs

```bash
# View latest backend logs
tail -f logs/backend_latest.log

# View latest frontend logs
tail -f logs/frontend_latest.log

# Check tmux pane output directly
tmux capture-pane -t track-leader:dev.1 -p | tail -20  # Backend pane
tmux capture-pane -t track-leader:dev.2 -p | tail -20  # Frontend pane
```

### Restarting Components

Panes are configured with `remain-on-exit on`, so after Ctrl-C the pane stays open (shows "Pane is dead"). To restart:

```bash
# Respawn backend pane - reruns the original command
tmux respawn-pane -t track-leader:dev.1

# Respawn frontend pane
tmux respawn-pane -t track-leader:dev.2

# Respawn postgres pane
tmux respawn-pane -t track-leader:dev.0
```

Or interactively: attach to the session, select the dead pane, and press `Ctrl+b` then type `:respawn-pane`

### Stopping Everything

```bash
tmux kill-session -t track-leader
```

### Why This Matters

- User can peek at the tmux session to see what's happening
- Logs are preserved in `logs/` directory
- All components run from correct directories
- Prevents issues like running from wrong project directory

---

## Rust Style

Always use in-place format! calls. For example:

Instead of:

```rust
let name = "Alice";
let age = 30;
println!("Hello, {}! You are {} years old.", name, age);
```

Do this:

```rust
let name = "Alice";
let age = 30;
println!("Hello, {name}! You are {age} years old.");
```
