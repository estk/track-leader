# Development Environment

Fully containerized development environment using Docker Compose with hot reloading. Multi-arch support for Apple Silicon (M1/M2) and x86_64.

## Quick Start

```bash
# Start in foreground with logs (Ctrl+C to stop)
./scripts/dev.sh

# Start detached (background)
./scripts/dev.sh -d

# Check status and health
./scripts/dev.sh status

# View logs
./scripts/dev.sh logs [backend|frontend|postgres]

# Stop all services
./scripts/dev.sh stop

# Restart a service
./scripts/dev.sh restart backend
```

## Commands Reference

| Command | Description |
|---------|-------------|
| `./scripts/dev.sh` | Start all services in foreground |
| `./scripts/dev.sh -d` | Start in detached mode |
| `./scripts/dev.sh --build` | Force rebuild of images |
| `./scripts/dev.sh status` | Show container and health status |
| `./scripts/dev.sh logs [service]` | View logs (follows by default) |
| `./scripts/dev.sh logs --no-follow` | Show logs without following |
| `./scripts/dev.sh logs --tail 50` | Show last 50 lines |
| `./scripts/dev.sh stop` | Stop all containers |
| `./scripts/dev.sh stop --volumes` | Stop and remove volumes (clears database) |
| `./scripts/dev.sh restart <service>` | Restart specific service |
| `./scripts/dev.sh restart all` | Restart all services |

Services: `backend`, `frontend`, `postgres`

## Port Configuration

By default, random ports are assigned to avoid conflicts when running multiple workspaces. You can specify ports explicitly:

```bash
./scripts/dev.sh --frontend-port 3000 --backend-port 3001 --postgres-port 5432
```

Port configuration is saved to `.dev-ports` and used by subsequent commands.

## Health Checks

All services have health checks configured:
- **PostgreSQL**: `pg_isready` command
- **Backend**: HTTP check to `/health` endpoint
- **Frontend**: HTTP check to root path

Use `./scripts/dev.sh status` to see health status of all services.

## Hot Reloading

- **Backend**: Uses `cargo-watch` to automatically rebuild on source changes
- **Frontend**: Uses Next.js dev server with file watching

## Volumes

Each workspace gets isolated volumes (prefixed with project name):
- `pgdata` - PostgreSQL data
- `cargo_cache` - Cargo registry cache
- `target_cache` - Rust build target cache
- `node_modules` - Node dependencies
- `next_cache` - Next.js build cache
- `uploads` - File uploads

---

## Parallel Workspaces

Multiple workspaces can run simultaneously:

1. **Random Ports**: By default, each workspace gets random available ports
2. **Unique Project Names**: Based on workspace directory name
3. **Isolated Data**: Each workspace has its own PostgreSQL container and data volume

Example running two workspaces:
```bash
# In workspace 1 (tl-ws1/)
./scripts/dev.sh -d
# Frontend: http://localhost:12345, Backend: http://localhost:23456

# In workspace 2 (tl-ws2/)
./scripts/dev.sh -d
# Frontend: http://localhost:14567, Backend: http://localhost:25678
```

---

## Connecting to Database

```bash
# Connect to PostgreSQL
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
./scripts/dev.sh --build

# Clear all volumes and rebuild
./scripts/dev.sh stop --volumes
./scripts/dev.sh --build
```

### Service Not Starting

Check logs for specific service:
```bash
./scripts/dev.sh logs backend
./scripts/dev.sh logs frontend
./scripts/dev.sh logs postgres
```

### Health Check Failing

The backend health check has a 60-second start period to allow for initial Rust compilation. If it's still failing after that:

```bash
# Check backend logs
./scripts/dev.sh logs backend

# Restart the service
./scripts/dev.sh restart backend
```
