# Phase 4 Exploration Results

**Date:** 2026-01-27
**Purpose:** Architecture analysis for Leaderboards implementation

---

## Backend Architecture Summary

### Current Database Schema

**Users Table** (migrations 001, 002)
- Core fields: `id`, `email`, `name`, `password_hash`, `auth_provider`, `avatar_url`, `bio`
- **Missing for Phase 4:** gender, birth_year, weight_kg, country, region

**Segments Table** (migration 003)
- Full geographic model with PostGIS LINESTRING geometry
- Fields: name, description, activity_type, distance, elevation metrics, climb_category
- Indexes: GIST spatial indexes on `geo`, `start_point`

**Segment Efforts Table** (migration 003)
- Primary leaderboard data: `elapsed_time_seconds`, `moving_time_seconds`, `average_speed_mps`
- Rankings: `is_personal_record` boolean
- Indexed on: `(segment_id, elapsed_time_seconds)` for fast ranking

### Key Backend Files

| File | Lines | Purpose |
|------|-------|---------|
| `crates/tracks/src/database.rs` | 986 | All DB queries including segment efforts |
| `crates/tracks/src/handlers.rs` | 1049 | HTTP handlers |
| `crates/tracks/src/models.rs` | ~200 | Domain structs |
| `crates/tracks/src/lib.rs` | ~100 | Router setup |
| `crates/tracks/src/activity_queue.rs` | ~250 | Background processing |

### Current Leaderboard Implementation

**Endpoint:** `GET /segments/{id}/leaderboard`
**Handler:** `handlers.rs:742-748`
```rust
pub async fn get_segment_leaderboard(
    Extension(db): Extension<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SegmentEffort>>, AppError> {
    let efforts = db.get_segment_efforts(id, 100).await?;
    Ok(Json(efforts))
}
```

**Limitations:**
- No time-based filters (all-time only)
- No demographic filters
- Hardcoded limit of 100
- No user info joined

### Ranking Query Pattern

From `database.rs:511-544` (get_activity_segment_efforts):
```sql
(SELECT COUNT(*) + 1 FROM segment_efforts e2
 WHERE e2.segment_id = e.segment_id
 AND e2.elapsed_time_seconds < e.elapsed_time_seconds) as rank
```

### Personal Records System

Already implemented in `database.rs:673-712`:
- Clears all PR flags for user on segment
- Sets PR on fastest effort
- Called after each new effort creation

### Background Processing Pattern

`activity_queue.rs` uses:
- Rayon thread pool for CPU-bound work
- `Arc<Mutex<HashSet<Uuid>>>` for in-flight tracking
- `mpsc::channel` for completion signaling

---

## Frontend Architecture Summary

### Tech Stack

- Next.js 14 (App Router)
- React 18 + TypeScript
- Tailwind CSS + CVA (class-variance-authority)
- MapLibre GL v5.16 with react-map-gl
- Recharts v3.7
- TanStack Query v5 (installed, not used)
- Zustand v4.5 (installed, not used)

### Key Frontend Files

| File | Purpose |
|------|---------|
| `src/lib/api.ts` | Centralized API client |
| `src/lib/auth-context.tsx` | Auth state provider |
| `src/app/segments/[id]/page.tsx` | Segment detail with leaderboard table |
| `src/app/segments/page.tsx` | List page with filters |
| `src/components/ui/` | Shadcn-style primitives |
| `src/components/activity/activity-map.tsx` | MapLibre component |
| `src/components/activity/elevation-profile.tsx` | Recharts chart |

### API Client Pattern

```typescript
class ApiClient {
  private token: string | null = null

  async getSegmentLeaderboard(id: string): Promise<SegmentEffort[]>
  async getMySegmentEfforts(id: string): Promise<SegmentEffort[]>
  // ... more methods
}

export const api = new ApiClient()
```

### UI Component Patterns

**Table rendering** (segments/[id]/page.tsx:298-340):
```jsx
<div className="grid grid-cols-4 text-sm font-medium">
  <span>Rank</span>
  {efforts.map((effort, index) => (
    <div key={effort.id} className="grid grid-cols-4">...</div>
  ))}
</div>
```

**Filter controls** (segments/page.tsx):
```jsx
<select value={sortBy} onChange={(e) => setSortBy(e.target.value)}>
  {SORT_OPTIONS.map(opt => <option>{opt.label}</option>)}
</select>
```

**Medal display:** Uses emoji `🥇 🥈 🥉`

### State Management

Currently: React hooks (`useState`, `useEffect`, `useContext`)
Available: TanStack Query, Zustand (installed but unused)

---

## Files to Create for Phase 4

### Backend (New)

1. `migrations/008_add_demographics.sql` - User demographic fields
2. `migrations/009_leaderboard_cache.sql` - Cache table
3. `migrations/010_achievements.sql` - KOM/QOM tracking
4. `src/leaderboard_service.rs` - Computation logic
5. `src/achievements_service.rs` - Crown tracking

### Backend (Modify)

1. `src/lib.rs` - Add new routes
2. `src/handlers.rs` - Enhance leaderboard handler, add new handlers
3. `src/database.rs` - Add filtered leaderboard queries
4. `src/models.rs` - Add LeaderboardEntry, Achievement structs
5. `src/activity_queue.rs` - Trigger cache invalidation

### Frontend (New)

1. `src/app/leaderboards/page.tsx` - Global leaderboards
2. `src/app/segments/[id]/leaderboard/page.tsx` - Full leaderboard page
3. `src/app/profile/[username]/rankings/page.tsx` - Personal rankings
4. `src/components/leaderboard/` - Leaderboard table, filters, crown icons

### Frontend (Modify)

1. `src/lib/api.ts` - Add leaderboard endpoints with filters
2. `src/app/segments/[id]/page.tsx` - Enhance leaderboard display

---

## Key Technical Decisions

### Caching Strategy

Per phase-4-leaderboards.md:
| Scope | TTL | Refresh |
|-------|-----|---------|
| Week | 5 minutes | On effort |
| Month | 15 minutes | Every 10 min |
| Year | 1 hour | Every 30 min |
| All-time | 1 hour | Every 30 min |

### Real-time Updates

Use Server-Sent Events (SSE):
- Axum supports streaming responses
- Frontend: `EventSource` API in `useEffect`
- Events: `connected`, `new_leader`, `rank_change`, `your_rank`

### Crown Types

| Crown | Criteria |
|-------|----------|
| KOM | Fastest all-time, men |
| QOM | Fastest all-time, women |
| Local Legend | Most efforts in 90 days |
| Course Record | Fastest all-time, any gender |

---

## Dependencies Status

### Backend - Available
- axum (SSE support via Tower)
- tokio (full features)
- sqlx (type-safe SQL)
- time (DateTime)
- serde_json

### Backend - May Need
- Redis client (if distributed caching needed)

### Frontend - Available
- TanStack Query (for data fetching)
- Recharts (for charts)
- MapLibre (for maps)
- Lucide React (for icons including crowns)
