# Development Environment

Two options for running the development environment. Both support random ports to allow multiple workspaces to run simultaneously without conflicts.

## Option 1: Zellij-Based (Native Processes)

Uses zellij terminal multiplexer with native processes. PostgreSQL runs in Docker, but frontend and backend run natively.

### Starting

```bash
# Random ports (default)
./scripts/start-dev.sh

# Explicit ports
./scripts/start-dev.sh --frontend-port 3000 --backend-port 3001 --postgres-port 5432
```

Creates a zellij session named `tl-<workspace-dir>` with 3 panes:
- **Left (30%):** PostgreSQL via docker-compose
- **Top-right:** Rust/Axum backend
- **Bottom-right:** Next.js frontend

Port configuration saved to `.dev-ports`.

### Scripts

| Script | Description |
|--------|-------------|
| `./scripts/start-dev.sh` | Start all services in a zellij session |
| `./scripts/stop-dev.sh` | Stop all services and kill the session |
| `./scripts/attach-dev.sh` | Attach to the running zellij session |
| `./scripts/restart-service.sh <service>` | Restart a specific service |
| `./scripts/dev-status.sh` | Show status of all services and ports |
| `./scripts/watch-logs.sh [component]` | Follow logs in real-time |
| `./scripts/peek-logs.sh [component] [lines]` | Show recent log entries |

### Logs

```bash
# Follow all logs
./scripts/watch-logs.sh

# Follow specific component
./scripts/watch-logs.sh backend
./scripts/watch-logs.sh frontend
./scripts/watch-logs.sh postgres

# Peek at recent entries
./scripts/peek-logs.sh backend
./scripts/peek-logs.sh frontend 100

# Direct tail
tail -f logs/backend_latest.log
```

### Session Management

```bash
# Attach to session
./scripts/attach-dev.sh

# Detach: Ctrl+o, then d

# Stop everything
./scripts/stop-dev.sh
```

---

## Option 2: Docker-Based (Recommended)

Fully containerized with hot reloading. Multi-arch support for Apple Silicon (M1/M2) and x86_64.

### Starting

```bash
# Foreground with logs (Ctrl+C to stop)
./scripts/start-dev-docker.sh

# Detached (background)
./scripts/start-dev-docker.sh -d

# Force rebuild images
./scripts/start-dev-docker.sh --build

# Explicit ports
./scripts/start-dev-docker.sh --frontend-port 3000 --backend-port 3001 --postgres-port 5432
```

Port configuration saved to `.dev-ports-docker`.

### Scripts

| Script | Description |
|--------|-------------|
| `./scripts/start-dev-docker.sh` | Start all services via Docker Compose |
| `./scripts/stop-dev-docker.sh` | Stop all containers |
| `./scripts/stop-dev-docker.sh --volumes` | Stop and remove volumes (clears database) |
| `./scripts/status-dev-docker.sh` | Show container and port status |
| `./scripts/logs-dev-docker.sh [service]` | View logs |
| `./scripts/restart-dev-docker.sh <service>` | Restart a specific service |

### Hot Reloading

- **Backend**: Uses `cargo-watch` to automatically rebuild on source changes
- **Frontend**: Uses Next.js dev server with file watching

### Volumes

Each workspace gets isolated volumes (prefixed with project name):
- `pgdata` - PostgreSQL data
- `cargo_cache` - Cargo registry cache
- `target_cache` - Rust build target cache
- `node_modules` - Node dependencies
- `next_cache` - Next.js build cache
- `uploads` - File uploads

---

## Parallel Workspaces

Both environments support running multiple workspaces simultaneously:

1. **Random Ports**: By default, each workspace gets random available ports
2. **Unique Session/Project Names**: Based on workspace directory name
3. **Isolated Data**: Each workspace has its own PostgreSQL container and data volume

Example running two workspaces:
```bash
# In workspace 1 (tl-ws1/)
./scripts/start-dev-docker.sh
# Frontend: http://localhost:12345, Backend: http://localhost:23456

# In workspace 2 (tl-ws2/)
./scripts/start-dev-docker.sh
# Frontend: http://localhost:14567, Backend: http://localhost:25678
```

---

## Connecting to Database

```bash
# Zellij setup (container name includes workspace)
docker exec -it tracks_postgres_<workspace> psql -U tracks_user -d tracks_db

# Docker setup
docker exec -it tl_<workspace>_postgres psql -U tracks_user -d tracks_db
```

Useful queries:
```sql
-- Check migrations
SELECT version, description, success FROM _sqlx_migrations ORDER BY version;

-- List tables
\dt

-- Check indexes
\di segment_stars*
```

---

## Troubleshooting

### Port Already in Use

If you get port conflicts, either:
1. Let the scripts pick random ports (default behavior)
2. Stop the conflicting process manually
3. Specify different ports with `--frontend-port`, `--backend-port`, `--postgres-port`

### Docker Build Failures

```bash
# Rebuild from scratch
./scripts/start-dev-docker.sh --build

# Clear all volumes and rebuild
./scripts/stop-dev-docker.sh --volumes
./scripts/start-dev-docker.sh --build
```

### Zellij Session Issues

```bash
# Force kill session
zellij delete-session tl-<workspace> --force

# Check what's running
./scripts/dev-status.sh
```
