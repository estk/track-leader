# Track Leader - Current State Analysis

**Date:** January 2026
**Last Updated:** January 27, 2026
**Status:** Phase 4 Complete, Beginning Phase 5

## Executive Summary

Track Leader is a GPS activity tracking application with aspirations to become an open leaderboard platform for trail segments, competing with Strava's segment feature. The project consists of:

1. **A functional Rust backend** - Well-architected Axum service with PostgreSQL/PostGIS
2. **A functional Next.js frontend** - Integrated with backend, mobile responsive

**Current Phase:** Phase 5 (Social Features) - Starting

**Completed Phases:**
- ✅ Phase 1: Foundation (Auth, Activity Upload, Basic UI)
- ✅ Phase 2: Core Features (Activity Management, Maps, Profiles)
- ✅ Phase 3: Segments (Creation, Matching, PRs, Starring)
- ✅ Phase 4: Leaderboards (Filters, Demographics, Achievements, Global Leaderboards)

---

## What's Working

### Backend Features

| Feature | Status | Notes |
|---------|--------|-------|
| User authentication | ✅ | JWT + argon2 |
| GPX file upload | ✅ | Multipart form upload |
| Activity storage | ✅ | Object store abstraction |
| Metrics calculation | ✅ | Distance, duration, elevation gain |
| Background processing | ✅ | Rayon thread pool + async |
| Database persistence | ✅ | PostgreSQL with PostGIS |
| Track geometry storage | ✅ | LINESTRING with GIST index |
| Segment creation | ✅ | PostGIS geometry operations |
| Segment matching | ✅ | ST_DWithin + ST_LineLocatePoint |
| Personal records | ✅ | Auto-updated on effort |
| Starred segments | ✅ | User favorites |
| Filtered leaderboards | ✅ | Scope, gender, age group |
| Demographics | ✅ | Gender, birth year, weight, location |
| Achievements | ✅ | KOM/QOM/Local Legend schema |
| Global leaderboards | ✅ | Crown count, distance rankings |

### Frontend Features

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

---

## Architecture

### Technology Stack

**Backend:**
- Axum 0.8 (async web framework)
- SQLx 0.8 (compile-time checked SQL)
- PostgreSQL 15 + PostGIS (spatial database)
- Object Store crate (S3-compatible abstraction)
- Rayon (parallel processing)
- Tokio (async runtime)

**Frontend:**
- Next.js 14 (App Router)
- React 18 + TypeScript
- Tailwind CSS + CVA
- MapLibre GL v5.16
- Recharts v3.7

### Database Schema (Migrations 001-010)

**Core Tables:**
- `users` - Auth, profile, demographics
- `activities` - User activities with metadata
- `tracks` - PostGIS GEOGRAPHY LineString Z
- `scores` - Computed metrics
- `segments` - User-created trail segments
- `segment_efforts` - Matched efforts with timing
- `starred_segments` - User favorites
- `leaderboard_cache` - Cached filtered leaderboards
- `achievements` - KOM/QOM/Local Legend records

---

## File Structure

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

## Phase 4 Completion Summary

### What Was Built

1. **Database Migrations (008-010)**
   - User demographics (gender, birth year, weight, country, region)
   - Leaderboard cache with JSONB entries and TTL
   - Achievements table for KOM/QOM/Local Legend

2. **Backend Endpoints**
   - Filtered leaderboard with scope/gender/age filters
   - Demographics PATCH endpoint
   - Achievement endpoints (user and segment)
   - Global leaderboards (crowns, distance)

3. **Frontend Pages**
   - `/segments/[id]/leaderboard` - Full filtered leaderboard
   - `/profile/settings` - Demographics form
   - `/profile/achievements` - Crown gallery
   - `/profile/rankings` - Personal segment rankings
   - `/leaderboards` - Global crown/distance rankings

### Deferred to Polish Phase

- SSE real-time leaderboard updates
- Leaderboard caching service
- Auto-achievement processing on effort creation

---

## What's Next: Phase 5 Social Features

### Core Scope
- Follow system
- Activity feed from followed users
- Kudos and comments
- Notifications

### Potential Extensions (Your Ideas)
- **Teams** - Team visibility, team pages, team feeds (likely Phase 5 extension or Phase 7)
- **Synthetic test data** - Generate or import test data
- **API type generation** - Protobuf/Swagger for Rust-Node interface

### Stretch Goals
- Enhanced leaderboard filters (weight class, equipment)
- GPS refresh rate in track stats
- Data quality-based segment matching tolerance

---

## Development Environment

Start with:
```bash
./scripts/start-dev.sh
```

Creates tmux session `track-leader` with:
- Pane 0: PostgreSQL (docker-compose)
- Pane 1: Rust backend (port 3001)
- Pane 2: Next.js frontend (port 3000)

Monitor logs:
```bash
tail -f logs/backend_latest.log
tail -f logs/frontend_latest.log
```
