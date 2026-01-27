# CLAUDE.md

## Development Environment

**IMPORTANT:** Always use the `start-dev.sh` script to start the development environment. Never start components individually with `npm run dev` or `cargo run` directly.

### Starting the Dev Environment

```bash
# From project root - runs in background, creates persistent tmux session
./scripts/start-dev.sh &
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
tmux capture-pane -t track-leader:0.1 -p | tail -20  # Backend pane
tmux capture-pane -t track-leader:0.2 -p | tail -20  # Frontend pane
```

### Restarting Components

If a component crashes or needs restart:

```bash
# Restart backend (pane 1)
tmux send-keys -t track-leader:0.1 C-c
tmux send-keys -t track-leader:0.1 "cd /Users/estk/git/tl-ws1/crates/tracks && RUST_LOG=info DATABASE_URL='postgres://tracks_user:tracks_password@localhost:5432/tracks_db' cargo run" Enter

# Restart frontend (pane 2)
tmux send-keys -t track-leader:0.2 C-c
tmux send-keys -t track-leader:0.2 "cd /Users/estk/git/tl-ws1 && npm run dev" Enter
```

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
