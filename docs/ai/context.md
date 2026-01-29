# AI Context for Track Leader

Code patterns and gotchas for AI assistants working on this codebase.

## Development Environment

See [development.md](./development.md) for full setup instructions.

**Quick start:**
```bash
./scripts/dev.sh     # Docker-based (recommended)
./scripts/dev.sh -d  # Detached mode
```

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
  cargo sqlx migrate info --source crates/tracks/migrations

DATABASE_URL="postgres://tracks_user:tracks_password@localhost:5432/tracks_db" \
  cargo sqlx migrate run --source crates/tracks/migrations
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

### Authentication & Authorization

**All auth happens in Rust.** See [docs/architecture/security.md](../architecture/security.md).

```rust
// CORRECT: Get user from auth token
pub async fn create_activity(
    AuthUser(claims): AuthUser,
    // ...
) {
    let user_id = claims.sub;  // From verified PASETO token
}

// WRONG: User ID from request params (security vulnerability)
pub async fn create_activity(
    Query(params): Query<UploadQuery>,  // params.user_id is UNTRUSTED
) {
    let user_id = params.user_id;  // NEVER DO THIS
}
```

Key rules:
- Use `AuthUser` extractor for all mutation endpoints
- Extract user ID from `claims.sub`, never from query/body params
- Team membership checked via database (not in token) for immediate revocation
- Frontend permission checks are UX only - backend enforces all access control

### Backend/Frontend Architecture

**Business logic lives in Rust.** The frontend should only:
- Render UI
- Handle user input
- Call Rust APIs
- Display results

Key calculations are in `handlers.rs`:
- `haversine_distance()` - distance between GPS points
- `calculate_total_distance()` - segment length
- `calculate_elevation_change()` - gain/loss
- `calculate_grades()` - average and max grade
- `calculate_climb_category()` - HC through Cat 4

**Segment filtering/sorting** is server-side via query parameters on `GET /segments`:
- `search`, `sort_by`, `sort_order`, `min_distance_meters`, `max_distance_meters`, `climb_category`

**Segment preview** via `POST /segments/preview` returns metrics without creating a segment. Used for real-time preview in segment creation UI.

### Dynamic SQL in Database Layer

For endpoints with optional filters, use dynamic SQL building:

```rust
let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
let mut param_idx = 1;

if params.search.is_some() {
    conditions.push(format!("LOWER(name) LIKE ${param_idx}"));
    param_idx += 1;
}

let query = format!("SELECT ... WHERE {} LIMIT ${}", conditions.join(" AND "), param_idx);
let mut q = sqlx::query_as::<_, Model>(&query);

// Bind in same order as conditions were added
if let Some(ref pattern) = search_pattern {
    q = q.bind(pattern);
}
```

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
