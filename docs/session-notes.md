# Session Notes - January 26-27, 2026

## Current Status

**Phase 1:** Complete (except staging deployment)
**Phase 2:** In Progress - Core activity features working

### What's Working

1. **Authentication** - JWT + argon2, login/register/logout
2. **Activity Upload** - GPX file upload with activity type selection
3. **Activity List** - Shows user's activities with clickable cards
4. **Activity Detail Page:**
   - Interactive map with OpenTopoMap tiles (contour lines, hill shading)
   - Elevation profile chart (Recharts) with distance/gain/range stats
   - Statistics display (points, start/end elevation, bounds)
   - Edit modal (name, activity type)
   - Delete with confirmation
   - Download GPX button
5. **Styling** - Tailwind CSS with shadcn/ui-style components

### Key Fixes Made This Session

1. **PostCSS Config Missing** - Created `postcss.config.js` in project root. Without this, Tailwind CSS doesn't process.

2. **SQLx Compile-Time Macros** - Converted `sqlx::query!` and `sqlx::query_as!` macros to runtime versions (`sqlx::query`, `sqlx::query_as`) to avoid needing DATABASE_URL at compile time.

3. **Date Serialization** - Added `#[serde(with = "rfc3339")]` to `OffsetDateTime` fields in `models.rs` so dates serialize as ISO strings instead of arrays.

4. **Activity Type Enum Mismatch** - Frontend was sending lowercase values ("run", "ride") but backend expects PascalCase ("Running", "RoadCycling"). Fixed in upload and edit pages.

5. **GPX MIME Type** - Browsers send GPX files as `application/octet-stream`. Updated `object_store_service.rs` to treat octet-stream as GPX.

6. **Database Column Mismatch** - `get_activity` query referenced non-existent columns (filename, distance, etc.). Fixed to match actual schema.

7. **Map Tiles** - Changed from MapLibre demo tiles to OpenTopoMap for proper topo maps with contour lines.

8. **Docker PostGIS** - Updated `docker-compose.yml` to use `postgis/postgis:15-3.3` image instead of plain postgres.

### Files Changed This Session

**Backend (crates/tracks/src/):**
- `models.rs` - Added rfc3339 serde for dates
- `database.rs` - Fixed queries, converted to runtime sqlx
- `handlers.rs` - Added track data endpoint, update/delete handlers
- `object_store_service.rs` - Fixed MIME type handling, removed panic
- `lib.rs` - Added new routes, CORS for PATCH/DELETE
- `auth.rs` - Fixed me() to return User instead of Claims

**Frontend (src/):**
- `components/activity/activity-map.tsx` - OpenTopoMap tiles
- `components/activity/elevation-profile.tsx` - Recharts chart
- `app/activities/[id]/page.tsx` - Detail page with edit/delete
- `app/activities/upload/page.tsx` - Fixed activity types
- `lib/api.ts` - Added track, update, delete endpoints
- `lib/auth-context.tsx` - Auth state management
- `postcss.config.js` - **Created** (was missing!)

**Config:**
- `docker-compose.yml` - PostGIS image
- `.github/workflows/ci.yml` - CI pipeline

### Running the App

```bash
# Terminal 1 - Database
cd crates/tracks
docker-compose up postgres

# Terminal 2 - Backend
cd crates/tracks
DATABASE_URL="postgres://tracks_user:tracks_password@localhost:5432/tracks_db" cargo run

# Terminal 3 - Frontend
npm run dev
```

Open http://localhost:3000

### What's Next (Phase 2 Remaining)

- [ ] Privacy controls (public/private activities)
- [ ] User profile page
- [ ] Mobile responsive design
- [ ] Activity list filtering/sorting

### Phase 3 Preview (Segments)

- Segment creation from activities
- Segment matching
- Segment leaderboards

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
