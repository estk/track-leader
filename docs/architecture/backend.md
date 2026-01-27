# Backend Architecture

## Overview

The Track Leader backend is a Rust service built with the Axum web framework. It handles GPS activity file processing, storage, and metric calculations.

## Technology Stack

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Web Framework | Axum | 0.8 | HTTP routing, middleware |
| Async Runtime | Tokio | 1.0 | Async I/O, task scheduling |
| Database | SQLx | 0.8 | Type-safe SQL queries |
| Database Engine | PostgreSQL | 15 | Relational storage |
| Spatial Extension | PostGIS | - | Geographic data types |
| File Storage | object_store | 0.12.4 | S3-compatible abstraction |
| GPX Parsing | gpx | 0.10 | GPS file parsing |
| Geospatial Math | geo | 0.31 | Distance calculations |
| Serialization | Serde | 1.0 | JSON encoding/decoding |
| Parallelism | Rayon | 1.11 | CPU-bound work |
| Logging | tracing | 0.41 | Structured logging |
| Error Handling | thiserror | - | Error type derivation |

## Module Structure

```
crates/tracks/src/
├── main.rs                 # Entry point, initialization
├── lib.rs                  # Router creation, server startup
├── handlers.rs             # HTTP request handlers
├── models.rs               # Domain models and types
├── database.rs             # Data access layer
├── scoring.rs              # GPS metric calculations
├── activity_queue.rs       # Background processing
├── object_store_service.rs # File storage abstraction
└── errors.rs               # Error types and HTTP mapping
```

## Key Components

### Router (`lib.rs`)

Creates the Axum router with all routes and middleware:

```rust
Router::new()
    .route("/health", get(health_check))
    .route("/users/new", get(new_user))
    .route("/users", get(all_users))
    .route("/activities/new", post(new_activity))
    .route("/activities/{id}", get(get_activity))
    .route("/activities/{id}/download", get(download_gpx_file))
    .route("/users/{id}/activities", get(get_user_activities))
    .layer(Extension(db))
    .layer(Extension(store))
    .layer(Extension(aq))
    .layer(cors)
```

**Design Decision:** Uses Axum's `Extension` layer for dependency injection rather than application state. This allows handlers to receive typed dependencies directly.

### Handlers (`handlers.rs`)

HTTP handlers follow a consistent pattern:

```rust
pub async fn handler_name(
    Extension(db): Extension<Database>,      // Dependencies via Extension
    Path(id): Path<Uuid>,                    // Path parameters
    Query(params): Query<ParamStruct>,       // Query string
) -> Result<Json<ResponseType>, AppError> {  // Typed result
    // Business logic
    Ok(Json(response))
}
```

**Current Handlers:**

| Handler | Route | Method | Purpose |
|---------|-------|--------|---------|
| `health_check` | `/health` | GET | Liveness probe |
| `new_user` | `/users/new` | GET | Create user (should be POST) |
| `all_users` | `/users` | GET | List all users |
| `new_activity` | `/activities/new` | POST | Upload GPX file |
| `get_activity` | `/activities/{id}` | GET | Get activity details |
| `get_user_activities` | `/users/{id}/activities` | GET | List user's activities |
| `download_gpx_file` | `/activities/{id}/download` | GET | Download original file |

### Scoring System (`scoring.rs`)

Implements a trait-based metric calculation system:

```rust
pub trait TrackMetric {
    type Score;
    fn next_point(&mut self, point: &TrackPoint);
    fn finish(&mut self) -> Self::Score;
}
```

**Current Metrics:**

| Metric | Output | Algorithm |
|--------|--------|-----------|
| DistanceMetric | f64 (meters) | Haversine formula between consecutive points |
| DurationMetric | f64 (seconds) | End time minus start time from GPX timestamps |
| ElevationGainMetric | f64 (meters) | Sum of positive elevation changes |

**Extensibility:** Adding new metrics requires:
1. Implement `TrackMetric` trait
2. Add to `Metrics` struct
3. Include in `finish()` aggregation

### Activity Queue (`activity_queue.rs`)

Background processing for CPU-intensive GPX parsing:

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│   Handler   │────►│  Rayon Pool  │────►│   Database   │
│  (submit)   │     │ (GPX parse)  │     │ (save scores)│
└─────────────┘     └──────────────┘     └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  Done Signal │
                    │  (mpsc chan) │
                    └──────────────┘
```

**Key Design:**
- Rayon thread pool handles CPU-bound GPX parsing
- Tokio runtime wrapped in `Arc<Runtime>` for async DB writes from sync context
- In-flight tracking via `HashSet<Uuid>` prevents duplicate processing

### Object Store (`object_store_service.rs`)

Abstraction over file storage:

```rust
pub struct ObjectStoreService {
    store: Arc<dyn ObjectStore>,  // Trait object for pluggability
    _base_path: String,
}
```

**Storage Path Format:** `activities/{user_id}/{activity_id}`

**Current Implementation:** Local filesystem via `LocalFileSystem`

**Future:** Can swap to S3, GCS, or Azure Blob by changing initialization.

### Error Handling (`errors.rs`)

Custom error type with HTTP response mapping:

```rust
pub enum AppError {
    Database(sqlx::Error),
    GpxParsing(String),
    Io(std::io::Error),
    InvalidInput(String),
    NotFound,
    Internal,
    Queue(anyhow::Error),
}
```

**HTTP Status Mapping:**
- `InvalidInput`, `GpxParsing` → 400 Bad Request
- `NotFound` → 404 Not Found
- `Database`, `Io`, `Queue`, `Internal` → 500 Internal Server Error

## Data Flow

### Activity Upload Flow

```
1. Client POSTs multipart form to /activities/new
2. Handler extracts file bytes and MIME type
3. File stored in object store
4. Activity record created (not yet scored)
5. Activity submitted to queue for background processing
6. Rayon worker parses GPX
7. Scores calculated via TrackMetric implementations
8. Scores saved to database
9. Worker signals completion via channel
```

### Activity Retrieval Flow

```
1. Client GETs /activities/{id}
2. Handler queries database by UUID
3. Activity returned as JSON (no scores attached currently)
```

## Configuration

**Environment Variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://docker:pg@0.0.0.0` | PostgreSQL connection string |
| `OBJECT_STORE_PATH` | `./uploads` | Local storage directory |
| `PORT` | `3000` | HTTP server port |
| `RUST_LOG` | (none) | Logging level filter |

## Current Limitations

1. **No authentication** - All endpoints are public
2. **Scores not joined** - Activity endpoint doesn't include scores
3. **Tracks table unused** - Activities not converted to PostGIS geography
4. **No pagination** - User activities return all records
5. **Sync unwraps** - Background queue uses `.unwrap()` without error handling
6. **Single file type** - Only GPX supported, FIT/TCX not implemented

## Recommended Improvements

### High Priority

1. Add authentication middleware (JWT or session-based)
2. Join scores when returning activities
3. Populate tracks table with PostGIS LineString
4. Add pagination to list endpoints
5. Proper error handling in activity queue

### Medium Priority

1. Support FIT and TCX file formats
2. Add rate limiting
3. Implement request tracing
4. Add metrics endpoint (Prometheus format)
5. Cache frequently accessed data

### Architecture Evolution

For segment leaderboard features, the backend will need:

1. **Segment matching service** - Match activity tracks to defined segments
2. **Leaderboard aggregation** - Compute and cache rankings
3. **Real-time updates** - SSE or WebSocket for live leaderboards
4. **Background jobs** - Scheduled recomputation of rankings
