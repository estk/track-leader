# CLAUDE.md

## Development Environment

**IMPORTANT:** Always use the `start-dev.sh` script to start the development environment. Never start components individually with `npm run dev` or `cargo run` directly.

### Starting the Dev Environment

```bash
# From project root - creates detached zellij session and returns immediately
./scripts/start-dev.sh
```

The script creates a zellij session named `track-leader` with 3 panes:
- **Left (30%):** PostgreSQL via docker-compose
- **Top-right:** Rust/Axum backend on port 3001
- **Bottom-right:** Next.js frontend on port 3000

### Available Scripts

| Script | Description |
|--------|-------------|
| `./scripts/start-dev.sh` | Start all services in a zellij session |
| `./scripts/stop-dev.sh` | Stop all services and kill the session |
| `./scripts/attach-dev.sh` | Attach to the running zellij session |
| `./scripts/restart-service.sh <service>` | Restart a specific service (backend, frontend, postgres, all) |
| `./scripts/dev-status.sh` | Show status of all services and ports |
| `./scripts/watch-logs.sh [component]` | Follow logs in real-time |
| `./scripts/peek-logs.sh [component] [lines]` | Show recent log entries |

### Checking Logs

```bash
# Follow all logs (with color highlighting)
./scripts/watch-logs.sh

# Follow specific component logs
./scripts/watch-logs.sh backend
./scripts/watch-logs.sh frontend
./scripts/watch-logs.sh postgres

# Peek at recent log entries (default: last 50 lines)
./scripts/peek-logs.sh backend
./scripts/peek-logs.sh frontend 100

# Direct tail access
tail -f logs/backend_latest.log
tail -f logs/frontend_latest.log
```

### Restarting Components

```bash
# Restart specific services
./scripts/restart-service.sh backend
./scripts/restart-service.sh frontend
./scripts/restart-service.sh postgres

# Restart everything (with proper sequencing)
./scripts/restart-service.sh all
```

### Attaching to the Session

```bash
# Attach to see pane output directly
./scripts/attach-dev.sh
# or: zellij attach track-leader

# Detach: Ctrl+o, then d
```

### Stopping Everything

```bash
./scripts/stop-dev.sh
```

### Why This Matters

- User can peek at the zellij session to see what's happening
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
