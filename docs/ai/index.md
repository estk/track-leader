# AI Context - Track Leader

Reference documentation for Claude when working on Track Leader.

## Documentation Index

- [index.md](./index.md) - Project overview and tech stack (this file)
- [development.md](./development.md) - Development environment setup
- [context.md](./context.md) - Code patterns and gotchas
- [learnings/](./learnings/) - Bug investigations and learnings

## Project Overview

Track Leader is an open leaderboard platform for trail segments - a competitor to Strava's segment feature with key differentiators:
- **Open by default** - Public segments, transparent rankings
- **User-defined metrics** - Compete on any measurable dimension
- **Trail-first design** - Split activities into shareable trail routes
- **Community-driven** - Segment creation, verification, and curation

## Technology Stack

### Frontend
| Technology | Purpose |
|------------|---------|
| Next.js 14 | Framework (App Router) |
| TypeScript | Type safety |
| Tailwind CSS | Styling |
| shadcn/ui | Component library |
| MapLibre GL | Maps |
| Recharts | Charts/graphs |

### Backend
| Technology | Purpose |
|------------|---------|
| Rust + Axum | Web framework |
| PostgreSQL + PostGIS | Database |
| SQLx | Database access |
| object_store | File storage |

## Key Directories

```
track-leader/
├── src/                    # Next.js frontend
│   ├── app/                # App Router pages
│   ├── components/         # React components
│   └── lib/                # Utilities (api.ts, auth-context.tsx)
├── crates/tracks/          # Rust backend
│   ├── src/
│   │   ├── handlers.rs     # HTTP handlers (~1500 lines)
│   │   ├── database.rs     # Data access (~1000 lines)
│   │   ├── models.rs       # Domain structs
│   │   └── lib.rs          # Router setup
│   └── migrations/         # Database migrations (001-015)
├── e2e/                    # Playwright E2E tests
├── load-tests/             # k6 performance tests
└── docs/
    ├── ai/                 # AI reference (this file)
    ├── architecture/       # Technical specs
    ├── user/               # User documentation
    └── planning/           # Planning docs (index.md for current state)
```

## Development Environment

See [development.md](./development.md) for full details.

**Quick start:**
```bash
./scripts/start-dev.sh          # Zellij-based (native)
./scripts/start-dev-docker.sh   # Docker-based (recommended)
```

Both support random ports for running multiple workspaces simultaneously.

## Database Schema Highlights

### Core Tables
- `users` - Auth, profile, demographics, follower counts
- `activities` - User activities with metadata, visibility (public/private/teams_only)
- `tracks` - PostGIS GEOGRAPHY LineStringZM (lon, lat, elevation, timestamp)
- `segments` - User-created trail segments, visibility (public/private/teams_only)
- `segment_efforts` - Matched efforts with timing

### Social Tables
- `follows` - User relationships with denormalized counts
- `kudos` - Activity likes
- `comments` - Activity comments
- `notifications` - User notifications (actor/target pattern)

### Teams Tables
- `teams` - Named groups with visibility and join policies
- `team_memberships` - User-team relationships with roles (owner/admin/member)
- `team_invitations` - Email-based invitations with tokens
- `team_join_requests` - Request-to-join workflow
- `activity_teams` - Many-to-many for team-shared activities
- `segment_teams` - Many-to-many for team-shared segments

## Important Patterns

### PostGIS LineStringZM
Track data is stored as 4D geometry:
- X = longitude
- Y = latitude
- Z = elevation (meters)
- M = timestamp (unix epoch)

See [learnings/bug-p6-001-track-storage.md](./learnings/bug-p6-001-track-storage.md) for details.

### Auth Flow
1. Register/Login returns JWT token
2. Token stored in localStorage
3. AuthContext manages state
4. Backend uses `AuthUser` extractor

### Denormalized Counts
Tables like `users` and `activities` have denormalized count columns (`follower_count`, `kudos_count`) updated atomically in transactions to avoid COUNT queries.

## Common Gotchas

### PostgreSQL
- `SELECT 1` returns INT4 (i32), not INT8 (i64)
- `COUNT(*)` returns INT8 (BIGINT)
- PostGIS uses `lon lat` order in WKT, not `lat lon`

### React/Next.js
- Auth state may not be set when useEffect runs - use `api.getToken()` for sync checks
- Recharts Tooltip runs during render - use `queueMicrotask()` to defer updates

### Rust/Axum
- Don't create nested tokio runtimes - use `Handle::current()`
- Route order matters - specific routes before wildcards

## Deferred Features

These were planned but deferred:
- SSE real-time leaderboard updates
- Leaderboard caching service
- Rate limiting (tower_governor added but not integrated)
- Sentry error tracking

## Test User

For development testing:
- Email: esims89+1@gmail.com
- Password: password
- Has activities, segments, and segment efforts

## Additional Resources

- [API Reference](../api-reference.md) - Full endpoint documentation
- [Architecture Overview](../architecture/overview.md) - System design
- [Deployment Guide](../deployment.md) - Production setup
- [Performance Targets](../performance.md) - Load testing targets
- [Runbook](../runbook.md) - Operations procedures
