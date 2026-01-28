# AI Context for Track Leader

Context and gotchas for AI assistants working on this codebase.

## Project Overview

Track Leader is a GPS activity tracking app with segment leaderboards (think Strava-like).

- **Frontend**: Next.js (TypeScript) on port 3000
- **Backend**: Rust/Axum on port 3001
- **Database**: PostgreSQL with PostGIS on port 5432

## Development Environment

**Always use the dev scripts** - never start components individually:
```bash
./scripts/start-dev.sh   # Start everything in zellij
./scripts/stop-dev.sh    # Stop everything
./scripts/dev-status.sh  # Check what's running
```

The restart script (`restart-service.sh`) has a bug where it creates broken symlinks for log files. Prefer full stop/start if restart doesn't work.

## Version Control

**This project uses jj (Jujutsu), not git.** Key commands:
- `jj status` / `jj diff` / `jj log`
- `jj commit -m "message"`
- `jj git push -c @` (for PRs)

## Database Migrations

Migrations are in `crates/tracks/migrations/` and use SQLx.

### Known Issues

1. **CONCURRENTLY indexes**: SQLx runs migrations in transactions, but `CREATE INDEX CONCURRENTLY` can't run in a transaction. Even with `-- no-transaction` directive, Postgres creates an implicit transaction when there are multiple statements.
   - **Solution**: Don't use CONCURRENTLY in migrations. For production, create indexes manually (see `docs/architecture/production-deployment.md`).

2. **Table naming**: Code uses `segment_stars` table. Migration 015 originally referenced `starred_segments` (wrong name) - this was fixed in migration 016.

### Running Migrations

Migrations run automatically on backend startup. To check/run manually:
```bash
DATABASE_URL="postgres://tracks_user:tracks_password@localhost:5432/tracks_db" \
  cargo sqlx migrate info --source /Users/estk/git/tl-ws1/crates/tracks/migrations

DATABASE_URL="postgres://tracks_user:tracks_password@localhost:5432/tracks_db" \
  cargo sqlx migrate run --source /Users/estk/git/tl-ws1/crates/tracks/migrations
```

### Current Schema (16 migrations)

1. init - users, activities, scores tables
2. add_constraints - foreign keys
3. segments - segments table
4. segments_z - elevation data
5. tracks_spatial_index - PostGIS index
6. segment_grades - grade calculations
7. effort_positions - leaderboard positions
8. add_demographics - user demographics
9. leaderboard_cache - cached rankings
10. achievements - user achievements
11. social_follows - follow relationships
12. notifications - user notifications
13. kudos_comments - social features
14. tracks_linestringzm - 4D track data
15. performance_indexes - query optimization indexes
16. segment_stars - starred/bookmarked segments

## Code Patterns

### Rust Backend

- Handlers in `src/handlers.rs` use Axum extractors
- Database layer in `src/database.rs`
- Models in `src/models.rs`
- Use `cargo +nightly fmt` for formatting
- Use `cargo nextest run` for tests
- Import unused traits as `use MyTrait as _`

### Unused Imports Warning

There are currently unused imports in `database.rs` (line 5-6):
- `Notification`, `NotificationWithActor`, `UserProfile`, `UserSummary`

These are likely for upcoming features.

## File Locations

- Backend crate: `crates/tracks/`
- Frontend app: `src/app/`
- Migrations: `crates/tracks/migrations/`
- Docker compose: `crates/tracks/docker-compose.yml`
- Dev scripts: `scripts/`
- Architecture docs: `docs/architecture/`

## Connecting to Database

```bash
docker exec -it tracks_postgres psql -U tracks_user -d tracks_db
```

Useful queries:
```sql
-- Check migrations
SELECT version, description, success FROM _sqlx_migrations ORDER BY version;

-- List tables
\dt

-- Check indexes on a table
\di segment_stars*
```
