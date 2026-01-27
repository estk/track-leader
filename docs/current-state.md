# Track Leader - Current State Analysis

**Date:** January 2026
**Last Updated:** January 27, 2026
**Status:** Active Development - Phase 4 (Leaderboards) In Progress

## Executive Summary

Track Leader is a GPS activity tracking application with aspirations to become an open leaderboard platform for trail segments, competing with Strava's segment feature. The project consists of:

1. **A functional Rust backend** - Well-architected Axum service with PostgreSQL/PostGIS
2. **A functional Next.js frontend** - Integrated with backend, mobile responsive

**Current Phase:** Phase 4 (Leaderboards) - Weeks 1-3 complete, Week 4 in progress

**Completed Phases:**
- Phase 1: Foundation (Auth, Activity Upload, Basic UI)
- Phase 2: Core Features (Activity Management, Maps, Profiles)
- Phase 3: Segments (Creation, Matching, PRs, Starring)
- Phase 4: Leaderboards (Filters, Demographics, Achievements) - In Progress

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

### What's Been Added (Phases 1-4)

- Authentication/authorization (JWT + argon2)
- Tracks table populated with PostGIS geometry
- Segments (creation, matching, PRs, starring)
- Leaderboards with demographic filters
- User demographics (gender, birth year, weight, location)
- Achievement system (KOM/QOM/Local Legend)
- Global leaderboards (crowns, distance)

### What's Still Missing

- SSE real-time leaderboard updates
- Achievement processing automation
- Social features (following, kudos, comments)
- Activity-to-route promotion

---

## Frontend Status

### Current State: Functional

The frontend has been completely rebuilt and integrated with the Rust backend.

### Tech Stack

- Next.js 14 (App Router)
- React 18 + TypeScript
- Tailwind CSS + CVA (class-variance-authority)
- MapLibre GL v5.16 with react-map-gl
- Recharts v3.7
- TanStack Query v5 (available)
- Zustand v4.5 (available)

### Working Features

| Feature | Status |
|---------|--------|
| Authentication | Login, register, logout with JWT |
| Activity Upload | GPX upload with activity type selection |
| Activity List | Cards with private indicator |
| Activity Detail | Map, elevation profile, hover sync, edit/delete |
| Segment Creation | Select start/end on elevation profile |
| Segments List | Search, filters, sorting, starring |
| Segment Detail | Map, stats, leaderboard, user efforts |
| Leaderboard | Filtered by scope, gender, age group |
| Profile | User info, activity counts |
| Profile Settings | Demographics form |
| Achievements | Crown gallery with filters |
| Rankings | Personal segment rankings |
| Global Leaderboards | Crown count, distance rankings |
| Mobile Responsive | Hamburger menu, touch-friendly |

### Key Frontend Files

| File | Purpose |
|------|---------|
| `src/lib/api.ts` | Centralized API client |
| `src/lib/auth-context.tsx` | Auth state provider |
| `src/components/leaderboard/` | Leaderboard table, filters, badges |
| `src/components/activity/` | Map, elevation profile |
| `src/components/ui/` | Shadcn-style primitives |

---

## Database Schema Analysis

### Current Schema (Migrations 001-010)

**Users** (001, 002, 008)
- Core: id, email, name, password_hash, auth_provider, avatar_url, bio
- Demographics (008): gender, birth_year, weight_kg, country, region

**Activities** (001)
- id, user_id, activity_type, name, visibility, object_store_path, submitted_at

**Tracks** (001, 005)
- id, user_id, activity_id, geo (GEOGRAPHY LineString Z)
- GIST spatial index

**Scores** (001)
- id, user_id, activity_id, distance, duration, elevation_gain

**Segments** (003, 004, 006)
- id, name, description, activity_type, creator_id
- Geometry: geo, start_point, end_point (all GEOGRAPHY)
- Metrics: distance_meters, elevation_gain, elevation_loss, max_grade, average_grade, climb_category
- Counters: effort_count

**Segment Efforts** (003, 007)
- id, segment_id, activity_id, user_id
- Timing: started_at, elapsed_time_seconds, moving_time_seconds
- Performance: average_speed_mps, is_personal_record
- Position: start_index, end_index (for map highlighting)

**Starred Segments** (003)
- user_id, segment_id, starred_at

**Leaderboard Cache** (009)
- id, segment_id, scope, filter_key, entries (JSONB), computed_at, expires_at

**Achievements** (010)
- id, user_id, segment_id, achievement_type, scope, effort_id, achieved_at, lost_at

### Tables Still Needed (Future Phases)

- `follows` - Social connections
- `kudos` - Activity likes
- `comments` - Activity comments
- `notifications` - In-app notifications

---

## File Structure Overview

```
track-leader/
├── src/                    # Next.js frontend
│   ├── app/                # App Router pages
│   │   ├── activities/     # Activity pages
│   │   ├── segments/       # Segment pages (including [id]/leaderboard)
│   │   ├── profile/        # Profile, settings, achievements, rankings
│   │   ├── leaderboards/   # Global leaderboards
│   │   └── ...
│   ├── components/         # React components
│   │   ├── activity/       # Map, elevation profile
│   │   ├── leaderboard/    # Table, filters, badges
│   │   └── ui/             # Shadcn-style primitives
│   └── lib/                # Utilities
│       ├── api.ts          # API client
│       └── auth-context.tsx
├── crates/tracks/          # Rust backend
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
│       ├── 001_init.sql through 010_achievements.sql
├── docs/                   # Documentation
│   ├── sessions/           # Session summaries
│   ├── planning/           # Phase plans
│   └── architecture/       # Technical specs
├── scripts/                # Dev scripts
│   ├── start-dev.sh
│   ├── stop-dev.sh
│   └── watch-logs.sh
└── package.json            # Node dependencies
```

---

## Next Steps

### Immediate (Complete Phase 4)

1. **SSE Real-time Updates** - Implement `/segments/{id}/leaderboard/stream` endpoint
2. **Achievement Processing** - Hook into activity_queue to auto-award KOM/QOM
3. **Manual Testing** - Verify all new pages work with real data

### Phase 5: Social Features

- Following/followers
- Activity feed
- Kudos and comments
- Notifications

### Phase 6: Polish

- Performance optimization
- Mobile app (React Native or PWA)
- OAuth providers (Strava, Google)
- Staging deployment

### Architecture Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Frontend framework | Next.js 14 | React ecosystem, App Router, good DX |
| Auth system | Email + JWT | Simple to start, OAuth can be added |
| API style | REST | Simple, cacheable |
| Real-time | SSE (planned) | Simpler than WebSockets for read-only updates |
| Maps | MapLibre GL | Open source, performant |

---

## Conclusion

Track Leader has evolved from a prototype to a functional application with:
- Complete authentication system
- Full activity management
- Segment creation and matching
- Filtered leaderboards with demographics
- Achievement system foundation

The remaining Phase 4 work (SSE, achievement automation) and future phases will complete the vision of a Strava segments competitor.
