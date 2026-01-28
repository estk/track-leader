# Tracks

Rust backend service for Track Leader. Built with Axum, PostgreSQL/PostGIS, and SQLx.

## Running

Use the project-level dev script:

```bash
./scripts/dev.sh
```

Or run manually:

```bash
DATABASE_URL="postgres://tracks_user:tracks_password@localhost:5432/tracks_db" \
  RUST_LOG=info cargo run
```

## Testing

```bash
cargo nextest run -p tracks
```

## Database Migrations

Migrations run automatically on startup. Located in `migrations/`.

See [API Reference](../../docs/api-reference.md) for endpoint documentation.
