# Session Notes - January 27, 2026

**Last verified:** 2026-01-27 - Phase 4 leaderboards IN PROGRESS.

## Current Status

**Phase 1:** Complete (except staging deployment)
**Phase 2:** Complete
**Phase 3:** Complete
**Phase 4:** In Progress (Weeks 1-3 complete, Week 4 partial)

### What's Working

1. **Authentication** - JWT + argon2, login/register/logout
2. **Activity Upload** - GPX file upload with activity type and visibility selection
3. **Activity List** - Shows user's activities with clickable cards, private indicator
4. **Activity Detail Page:**
   - Interactive map with OpenTopoMap tiles (contour lines, hill shading)
   - Elevation profile chart (Recharts) with distance/gain/range stats
   - Hover sync between elevation profile and map (orange marker)
   - Statistics display (points, start/end elevation, bounds)
   - Edit modal (name, activity type, visibility)
   - Delete with confirmation
   - Download GPX button
   - **Segment creation mode** - select start/end points on elevation profile
   - Real-time segment metrics preview (distance, elevation, grade, climb category)
   - Segment validation (min 100m, max 50km, min 10 points)
5. **Segments:**
   - Create segment from activity by selecting portion on elevation profile
   - Automatic segment matching on activity upload (PostGIS spatial queries)
   - Personal records tracking
   - Segment list page with search, filters (activity type, distance, climb category), sorting
   - Starred segments feature (star/unstar, starred tab)
   - Segment detail page with:
     - Map and elevation profile with hover sync
     - Statistics (distance, elevation, grade, climb category)
     - Global leaderboard
     - User's personal efforts section
   - Grade calculation (average and max)
   - Climb category calculation (Cat 4 through HC)
6. **User Profile Page** - Shows user info, activity summary (total/public/private counts)
7. **Mobile Responsive** - Hamburger menu, responsive layouts, touch-friendly
8. **Styling** - Tailwind CSS with shadcn/ui-style components

### Phase 3 Progress (Segments) - COMPLETE

**All Done:**
- [x] Database schema for segments (migrations 003-007)
- [x] 3D geometry support for elevation (migration 004)
- [x] Tracks spatial index (migration 005)
- [x] Grade and climb category columns (migration 006)
- [x] Effort positions for map highlighting (migration 007)
- [x] Segment and SegmentEffort models with all fields
- [x] Segment API endpoints (create, get, list, leaderboard, track, star/unstar, my-efforts, nearby, starred/efforts)
- [x] Segment creation with validation (min/max length, min points)
- [x] Duplicate segment detection (fuzzy spatial match)
- [x] Activity type inheritance (segments inherit from source activity)
- [x] Automatic segment matching on upload (PostGIS spatial query)
- [x] Personal records tracking (mark PRs on efforts)
- [x] Effort count cached counter (incremented on effort creation)
- [x] Moving time calculation (excludes stops < 1 m/s)
- [x] Segments list page with search, filters, sorting
- [x] Segment detail page with statistics, leaderboard, user efforts
- [x] Grade and climb category calculation
- [x] Starred segments feature with effort dashboard
- [x] Real-time metrics preview during segment creation
- [x] Highlight segment on activity map
- [x] Map-based segment discovery with clustering
- [x] PR history chart
- [x] Nearby segments feature ("Near Me" with geolocation)

### Phase 4 Progress (Leaderboards) - IN PROGRESS

**Backend Complete:**
- [x] Database migrations (008_add_demographics, 009_leaderboard_cache, 010_achievements)
- [x] Models: Gender, LeaderboardScope, AgeGroup, LeaderboardEntry, Achievement types
- [x] Enhanced leaderboard endpoint with filters (scope, gender, age_group, limit, offset)
- [x] Demographics update endpoint (PATCH /users/me/demographics)
- [x] Achievement endpoints (user achievements, segment achievements)
- [x] Global leaderboards (crowns, distance)
- [x] Fix: LocalFileSystem directory auto-creation in object_store_service.rs

**Frontend Complete:**
- [x] API client updates in src/lib/api.ts
- [x] Leaderboard components (leaderboard-table, leaderboard-filters, crown-badge)
- [x] Segment leaderboard page (/segments/[id]/leaderboard)
- [x] Profile settings page (/profile/settings)
- [x] Profile achievements page (/profile/achievements)
- [x] Profile rankings page (/profile/rankings)
- [x] Global leaderboards page (/leaderboards)

**Remaining:**
- [ ] SSE real-time leaderboard updates
- [ ] Achievement processing integration with activity_queue
- [ ] Local Legend calculation (90-day effort window)
- [ ] Manual testing of all new pages

### Key Fixes Made This Session (Jan 27) - Phase 3

1. **Elevation Profile Click Handler** - Changed from using `activePayload` (unreliable) to storing hovered index in a ref and using it on click.

2. **React Render Loop Warning** - The Tooltip content was calling `onHover` during render causing "Cannot update a component while rendering" warning. Fixed by using `queueMicrotask()` to defer state updates.

3. **Tokio Runtime Panic** - `ActivityQueue` was creating a nested tokio runtime with `Runtime::new()`. Changed to use `Handle::current()` to get a handle to the existing runtime.

4. **Segment Auth** - `create_segment` handler had placeholder `Uuid::nil()` for creator_id. Added `AuthUser` extractor to get real user ID from JWT.

5. **PostGIS 3D Geometry** - Segments need elevation data. Changed from `LINESTRING` to `LINESTRING Z` format and altered column with migration 004.

6. **Segment Track Endpoint** - Added `/segments/{id}/track` to return segment geometry with elevation for map and elevation profile display.

7. **Starred Segments Race Condition** - Frontend was using `isLoggedIn` state which wasn't set yet when useEffect ran. Changed to use synchronous `api.getToken()` check instead.

8. **INT4/INT8 Type Mismatch** - `SELECT 1` returns INT4 in PostgreSQL but Rust code expected `i64` (INT8). Changed `segment_effort_exists()` and `is_segment_starred()` to use `i32` instead.

### Key Fixes Made - Phase 4

9. **LocalFileSystem Directory Creation** - `object_store::local::LocalFileSystem::new(path)` fails if directory doesn't exist. Added `std::fs::create_dir_all()` before initialization in `object_store_service.rs`.

10. **Axum Route Ordering** - Specific routes must be registered before wildcard routes. `/segments/{id}/leaderboard` must come before `/segments/{id}` or the wildcard captures "leaderboard" as an ID.

11. **LeaderboardEntry Clone Derive** - Filtered leaderboard results needed to be cloned for pagination. Added `#[derive(Clone)]` to `LeaderboardEntry` struct.

### Files Changed This Session (Jan 27)

**Backend (crates/tracks/src/):**
- `activity_queue.rs` - Fixed tokio runtime issue (use Handle::current() not Runtime::new())
- `handlers.rs` - Added AuthUser to create_segment, added get_segment_track endpoint, store 3D coordinates
- `database.rs` - Added get_segment_geometry() method, fixed INT4/INT8 type mismatch in existence checks
- `lib.rs` - Added /segments/{id}/track route

**Backend (crates/tracks/migrations/):**
- `004_segments_z.sql` - Alter geo column to support Z dimension

**Frontend (src/):**
- `components/activity/elevation-profile.tsx` - Fixed click handler and hover state management
- `app/segments/[id]/page.tsx` - Added map, elevation profile, hover sync
- `app/segments/page.tsx` - Fixed starred segments race condition (use api.getToken() instead of isLoggedIn state)
- `lib/api.ts` - Added SegmentTrackData interface and getSegmentTrack method

### Running the App

**Quick Start (recommended):**
```bash
./scripts/start-dev.sh   # Starts all components in tmux with logging
```

**Monitor logs:**
```bash
./scripts/watch-logs.sh          # Watch all logs (errors highlighted)
./scripts/watch-logs.sh backend  # Watch backend only
tail -f logs/backend_latest.log  # Direct tail
```

**Stop everything:**
```bash
./scripts/stop-dev.sh
```

**Manual Start (alternative):**
```bash
# Terminal 1 - Database
cd crates/tracks
docker-compose up postgres

# Terminal 2 - Backend
cd crates/tracks
RUST_LOG=info DATABASE_URL="postgres://tracks_user:tracks_password@localhost:5432/tracks_db" cargo run

# Terminal 3 - Frontend
npm run dev
```

Open http://localhost:3000

### How to Continue Phase 3

**Next task: Automatic Segment Matching**

When a user uploads an activity, detect if they rode/ran through any existing segments and create SegmentEffort records automatically.

Implementation approach:
1. In `handlers.rs` `new_activity`, after saving the activity, query for segments that the track passes through
2. Use PostGIS `ST_DWithin` or `ST_Intersects` to find segments near the activity track
3. For matching segments, extract the portion of the activity track that covers the segment
4. Calculate elapsed time for that portion
5. Create SegmentEffort record

Key PostGIS functions:
- `ST_DWithin(geog1, geog2, distance_meters)` - check if geometries are within distance
- `ST_Intersects(geog1, geog2)` - check if geometries intersect
- `ST_LineLocatePoint(line, point)` - find position along line (0-1) where point is closest

**Personal Records Tracking**

After creating a SegmentEffort:
1. Query user's previous best time on that segment
2. If new effort is faster, mark it as `is_personal_record = true`
3. Update previous PR to `is_personal_record = false`

## Important Context Not in Code

### User Preferences (from CLAUDE.md)
- Uses **jj (Jujutsu)** not git - never use git commands
- Uses `cargo +nightly fmt` for formatting
- Uses `cargo nextest run` for tests
- Prefers simple solutions, minimal changes
- TDD approach when writing new code
- Address user as "PrimusMan" or "JagulonPrime"

### Browser Testing
- Chrome automation available via `mcp__claude-in-chrome__*` tools
- Can take screenshots, navigate, click to verify UI changes

### Test User Created
- Email: esims89+1@gmail.com (display name: evan)
- Password: password
- User ID: 4b5f1cde-e8b5-4baa-9cad-1834011aaefa
- Has one activity: "reno tour" (MountainBiking, ~107km near Reno NV)
  - Activity ID: f18ab674-b4a9-4f44-9501-59167e461bb7
  - GPX file: `crates/tracks/uploads/activities/{user_id}/{activity_id}`
- Has 3 segments: verdi climb, pvc, pea climb (all MountainBiking)
- Has segment efforts on all 3 segments (all PRs)
- Track geometry stored in `tracks` table (4042 points)

## Architecture Notes

### Backend Port
Backend runs on port **3001** (not 3000). Frontend proxies `/api/*` to backend via `next.config.js` rewrites.

### Database
PostgreSQL 15 with PostGIS extension. Migrations run automatically on backend startup.

### Object Store
GPX files stored locally in `./uploads` directory, organized as `activities/{user_id}/{activity_id}`.

### Auth Flow
1. Register/Login returns JWT token
2. Token stored in localStorage
3. AuthContext manages state, auto-fetches user on page load
4. Protected routes redirect to /login if not authenticated
5. Backend handlers use `AuthUser` extractor for authenticated endpoints

### Component Reuse
- `ActivityMap` and `ElevationProfile` components are reused for both activities and segments
- Convert `SegmentTrackData` to `TrackData` format by adding `time: null` to each point
- Pass `highlightIndex` and `onHover` props to enable hover sync between map and elevation profile

## Learnings & Gotchas

### PostgreSQL/PostGIS
- `SELECT 1` returns INT4 (32-bit), not INT8 (64-bit). Use `i32` in Rust, not `i64`.
- `COUNT(*)` returns INT8 (BIGINT), so `i64` is correct for count queries.
- PostGIS LINESTRING Z format: `LINESTRING Z(lon lat elev, lon lat elev, ...)`
- `ST_LineLocatePoint` returns 0-1 fraction for position along line.

### React/Next.js
- Auth state (`isLoggedIn`) may not be set when useEffect runs on page load.
- Use synchronous `api.getToken()` for immediate auth checks in useEffect.
- Recharts Tooltip `content` function runs during render - use `queueMicrotask()` to defer state updates.

### Rust/Axum
- Don't create nested tokio runtimes with `Runtime::new()`. Use `Handle::current()` to get existing runtime handle.
- `AuthUser` extractor pulls user ID from JWT for authenticated endpoints.

### Testing Segment Matching
To manually test segment matching without uploading new activities:
1. Insert track geometry: Extract coords from GPX, build WKT LINESTRING, insert into `tracks` table
2. Call `/segments/{id}/reprocess` for each segment to create efforts
3. Verify in `segment_efforts` table

### Useful SQL Queries
```sql
-- Check tracks table
SELECT id, activity_id, ST_NPoints(geo::geometry) as points FROM tracks;

-- Check segment efforts
SELECT se.id, s.name, se.elapsed_time_seconds, se.is_personal_record
FROM segment_efforts se JOIN segments s ON s.id = se.segment_id;

-- Check segments schema
\d segments
```
