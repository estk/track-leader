# Track Leader - Current State Analysis

**Date:** January 2026
**Status:** Early Development / Prototype

## Executive Summary

Track Leader is a GPS activity tracking application with aspirations to become an open leaderboard platform for trail segments, competing with Strava's segment feature. The project consists of:

1. **A functional Rust backend** - Well-architected Axum service with PostgreSQL/PostGIS
2. **A broken Next.js frontend** - Imports non-existent modules; not integrated with backend

The backend provides a solid foundation, but significant work is needed to realize the vision of an open segment leaderboard platform.

---

## Backend Status

### What Works

| Feature | Status | Notes |
|---------|--------|-------|
| User creation | Working | Basic user with email/name |
| GPX file upload | Working | Multipart form upload |
| Activity storage | Working | Object store abstraction |
| Metrics calculation | Working | Distance, duration, elevation gain |
| Background processing | Working | Rayon thread pool + async |
| Database persistence | Working | PostgreSQL with PostGIS |
| File download | Working | Returns original GPX |
| Health check | Working | Simple endpoint |

### Architecture Highlights

**Technology Stack:**
- Axum 0.8 (async web framework)
- SQLx 0.8 (compile-time checked SQL)
- PostgreSQL 15 + PostGIS (spatial database)
- Object Store crate (S3-compatible abstraction)
- Rayon (parallel processing)
- Tokio (async runtime)

**Key Design Patterns:**
- Trait-based metrics (`TrackMetric` trait for extensible scoring)
- Background queue for CPU-intensive GPX parsing
- Extension-based dependency injection
- Error type with HTTP response mapping

### What's Missing

- Authentication/authorization
- Tracks table not populated (activities stored but not converted to PostGIS tracks)
- Leaderboards (core feature not started)
- Segments (core feature not started)
- User preferences
- Activity-to-route promotion
- Score categorization (by time/location/user demographics)

---

## Frontend Status

### Critical Issue: Missing Modules

The frontend imports from modules that **do not exist**:

```typescript
import { db } from '@/lib/database'      // FILE DOES NOT EXIST
import { parseGPX } from '@/lib/gpx-parser'  // FILE DOES NOT EXIST
import { Track } from '@/lib/database'   // FILE DOES NOT EXIST
```

The frontend **cannot run** in its current state.

### Evidence of Abandoned SQLite Approach

- `package.json` includes `sqlite3` dependency
- `.gitignore` ignores `*.db`, `*.sqlite`, `*.sqlite3`
- File `tracks.db` exists in project root
- Frontend was apparently written to use SQLite directly, then abandoned when Rust backend was built

### What Exists (Non-Functional)

| Component | File | Status |
|-----------|------|--------|
| Home Page | `app/page.tsx` | Imports broken modules |
| Track List | `components/TrackList.tsx` | UI exists, no data source |
| Track Upload | `components/TrackUpload.tsx` | Posts to `/api/tracks` (wrong endpoint) |
| Track Map | `components/TrackMap.tsx` | Leaflet integration exists |
| Track Detail | `components/TrackDetail.tsx` | UI exists, no data source |
| API Routes | `app/api/tracks/` | Reference non-existent db module |

### UI Components Quality Assessment

**TrackList.tsx:**
- Basic table layout
- No pagination
- No sorting controls
- Minimal styling

**TrackUpload.tsx:**
- Drag-and-drop exists
- Icon sizing broken (`h-1 w-1` for upload icon)
- Uses `alert()` for feedback (poor UX)

**TrackMap.tsx:**
- Basic Leaflet Polyline
- No interactivity
- Static zoom level
- No route start/end markers

**TrackDetail.tsx:**
- Basic stats display
- Icon sizing broken (`w-44 h-44` for back chevron)
- Calorie calculation is naive (distance * 65)

---

## Database Schema Analysis

### Current Schema (001_init.sql)

```
users
├── id (UUID, PK)
├── email (TEXT, UNIQUE)
├── name (TEXT)
└── created_at (TIMESTAMPTZ)

activities
├── id (UUID, PK)
├── user_id (UUID)
├── activity_type (ENUM)
├── name (TEXT)
├── object_store_path (TEXT)
└── submitted_at (TIMESTAMPTZ)

tracks (UNUSED)
├── id (UUID, PK)
├── user_id (UUID)
├── activity_id (UUID)
├── created_at (TIMESTAMPTZ)
└── geo (GEOGRAPHY LineString)

scores
├── id (UUID, PK)
├── user_id (UUID)
├── activity_id (UUID)
├── distance (FLOAT)
├── duration (FLOAT)
├── elevation_gain (FLOAT)
└── created_at (TIMESTAMPTZ)
```

### Missing Tables for Full Vision

- `segments` - Defined portions of trails for competition
- `segment_efforts` - User attempts on segments
- `trails` - Named trail routes
- `leaderboards` - Aggregated rankings
- `user_preferences` - Settings and demographics
- `follows` - Social connections
- `kudos` / `comments` - Social engagement

---

## File Structure Overview

```
track-leader/
├── app/                    # Next.js app (BROKEN)
│   ├── api/tracks/         # API routes (import missing modules)
│   ├── tracks/[id]/        # Dynamic track page
│   ├── layout.tsx          # Root layout
│   └── page.tsx            # Home page
├── components/             # React components (UI only)
├── crates/tracks/          # Rust backend (FUNCTIONAL)
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── lib.rs          # Router setup
│   │   ├── handlers.rs     # HTTP handlers
│   │   ├── models.rs       # Domain models
│   │   ├── database.rs     # Data access
│   │   ├── scoring.rs      # Metrics calculation
│   │   ├── activity_queue.rs # Background processing
│   │   ├── object_store_service.rs # File storage
│   │   └── errors.rs       # Error handling
│   └── migrations/
│       └── 001_init.sql    # Database schema
├── tracks.db               # Abandoned SQLite database
├── package.json            # Node dependencies
└── Cargo.toml              # (in crates/tracks)
```

---

## Recommendations

### Immediate Actions

1. **Delete the frontend** - It cannot be salvaged. The architecture (SQLite + Next.js API routes) conflicts with the Rust backend.

2. **Build new frontend** - Choose between:
   - Next.js App Router with server actions calling Rust API
   - SPA (React/Vue/Svelte) consuming Rust REST API
   - HTMX + server-rendered templates from Rust

3. **Complete backend fundamentals:**
   - User authentication (OAuth2 or email/password)
   - Activity → Track conversion (populate `tracks` table with PostGIS)
   - Basic user profile API

### Medium-term Goals

4. **Implement segments:**
   - Define segment as polyline subset
   - Match activities to segments
   - Calculate segment times

5. **Build leaderboards:**
   - Per-segment rankings
   - Filtering by demographics/time

6. **Trail management:**
   - Split activities into trail sections
   - Crowdsourced trail definitions

### Architecture Decisions Needed

| Decision | Options | Recommendation |
|----------|---------|----------------|
| Frontend framework | Next.js, SvelteKit, HTMX | SvelteKit (modern, fast, good DX) |
| Auth system | OAuth only, Email+OAuth, Custom | OAuth2 (Google/Strava) + email |
| API style | REST, GraphQL, tRPC | REST (simple, cacheable) |
| Real-time | WebSockets, SSE, Polling | SSE for leaderboard updates |
| Hosting | Self-hosted, Fly.io, Railway | Fly.io (Postgres + Rust support) |

---

## Conclusion

Track Leader has a solid backend foundation but requires:
1. Complete frontend rewrite
2. Authentication implementation
3. Core segment/leaderboard features

The 6-month development plan in `docs/planning/` outlines how to transform this prototype into a legitimate Strava segments competitor.
