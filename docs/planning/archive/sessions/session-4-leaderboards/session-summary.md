# Session 4: Leaderboards Implementation

**Date:** January 27, 2026
**Phase:** 4 - Leaderboards
**Status:** In Progress (Weeks 1-3 largely complete, Week 4 partial)

---

## Overview

Phase 4 implements the competitive ranking system for segments, including filtered leaderboards, demographic support, achievements (KOM/QOM), and the foundations for real-time updates.

---

## Completed Work

### Backend: Database Migrations

| Migration | Purpose | Status |
|-----------|---------|--------|
| `008_add_demographics.sql` | Add gender, birth_year, weight_kg, country, region to users | Complete |
| `009_leaderboard_cache.sql` | Create leaderboard_cache table with JSONB entries, TTL support | Complete |
| `010_achievements.sql` | Create achievements table for KOM/QOM/LocalLegend tracking | Complete |

### Backend: Models Added

- `Gender` enum: Male, Female, Other, PreferNotToSay
- `LeaderboardScope` enum: AllTime, Year, Month, Week
- `AgeGroup` enum: All age brackets (18-24 through 65+)
- `LeaderboardEntry` struct: User info + rank + time gap to leader
- `Achievement` struct: Type, scope, effort reference, achieved_at, lost_at
- `AchievementType` enum: Kom, Qom, LocalLegend, CourseRecord

### Backend: New Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/segments/{id}/leaderboard` | Enhanced with query params (scope, gender, age_group, limit, offset) |
| PATCH | `/users/me/demographics` | Update user demographic information |
| GET | `/users/{id}/achievements` | Get user's crowns and achievements |
| GET | `/segments/{id}/achievements` | Get segment's current crown holders |
| GET | `/leaderboards/global/crowns` | Global crown count rankings |
| GET | `/leaderboards/global/distance` | Global total distance rankings |

### Backend: Bug Fix

- **LocalFileSystem directory creation**: `object_store_service.rs` was failing because `LocalFileSystem::new()` requires the directory to exist. Added automatic directory creation before initialization.

### Frontend: API Client Updates

File: `src/lib/api.ts`

- `getLeaderboard(id, filters)` - Accepts LeaderboardFilters with scope, gender, ageGroup, limit, offset
- `getMyPosition(id, filters)` - Get current user's rank and nearby ranks
- `updateDemographics(data)` - Update user demographic info
- `getUserAchievements(userId)` - Get user's crowns
- `getSegmentAchievements(segmentId)` - Get segment crown holders
- `getGlobalCrownsLeaderboard()` - Global crown rankings
- `getGlobalDistanceLeaderboard()` - Global distance rankings

### Frontend: New Components

Directory: `src/components/leaderboard/`

| Component | Purpose |
|-----------|---------|
| `leaderboard-table.tsx` | Paginated table with rank, user (avatar + name), time, speed, date columns. Medal icons for top 3, highlights current user row, loading skeleton |
| `leaderboard-filters.tsx` | Dropdowns for time scope, gender, age group. URL state persistence via search params |
| `crown-badge.tsx` | Crown icons for KOM (gold), QOM (gold), Local Legend (purple), PR (green). Tooltips with achievement dates |

### Frontend: New Pages

| Page | Path | Purpose |
|------|------|---------|
| Segment Leaderboard | `/segments/[id]/leaderboard` | Full leaderboard with filters and pagination |
| Profile Settings | `/profile/settings` | Demographics form (gender, birth year, weight, location) |
| Profile Achievements | `/profile/achievements` | Crown gallery with segment links, filter by type |
| Profile Rankings | `/profile/rankings` | All segments with user's rank, sortable |
| Global Leaderboards | `/leaderboards` | Tabs for crown count, distance, elevation rankings |

---

## Learnings and Gotchas

### Backend

1. **LocalFileSystem requires existing directory**
   - `object_store::local::LocalFileSystem::new(path)` fails if directory doesn't exist
   - Solution: Create directory with `std::fs::create_dir_all()` before initialization

2. **Route ordering in Axum matters**
   - Specific routes must come before wildcard routes
   - `/segments/{id}/leaderboard` must be registered before `/segments/{id}`
   - Otherwise the wildcard captures "leaderboard" as an ID

3. **LeaderboardEntry needs Clone derive**
   - Filtered query results are cloned when building paginated responses
   - Missing `#[derive(Clone)]` caused compiler errors

### Frontend

4. **URL search params for filter persistence**
   - Use `useSearchParams()` hook to read/write filter state
   - Enables shareable filtered leaderboard URLs
   - Remember to update URL when filters change

5. **Avatar fallback pattern**
   - Users may not have avatars set
   - Use initials-based fallback: `{user.name.charAt(0).toUpperCase()}`

---

## Remaining Work (Phase 4)

### Week 4 Items Not Yet Complete

- [ ] **SSE real-time updates**
  - `GET /segments/{id}/leaderboard/stream` endpoint
  - `useLeaderboardStream` hook with auto-reconnect
  - Animate rank changes on live updates

- [ ] **Achievement processing integration**
  - Hook achievement checks into `activity_queue.rs`
  - Award KOM/QOM when new fastest effort detected
  - Handle dethroning (set `lost_at` on previous holder)

- [ ] **Local Legend calculation**
  - Count efforts per user per segment in 90-day window
  - Award/update Local Legend achievement

- [ ] **Manual testing**
  - Test all new pages with real data
  - Verify filter combinations work correctly
  - Check pagination edge cases

---

## Files Changed

### Backend (crates/tracks/src/)

| File | Changes |
|------|---------|
| `models.rs` | Added Gender, LeaderboardScope, AgeGroup, LeaderboardEntry, Achievement, AchievementType |
| `database.rs` | Added filtered leaderboard queries, achievement queries, demographics update |
| `handlers.rs` | Enhanced leaderboard handler, added demographics/achievements handlers |
| `lib.rs` | Added new routes for demographics, achievements, global leaderboards |
| `object_store_service.rs` | Fixed directory auto-creation for LocalFileSystem |

### Backend (crates/tracks/migrations/)

| File | Purpose |
|------|---------|
| `008_add_demographics.sql` | User demographic columns |
| `009_leaderboard_cache.sql` | Leaderboard cache table |
| `010_achievements.sql` | Achievements table |

### Frontend (src/)

| File | Changes |
|------|---------|
| `lib/api.ts` | Added leaderboard, demographics, achievements API methods |
| `components/leaderboard/leaderboard-table.tsx` | New component |
| `components/leaderboard/leaderboard-filters.tsx` | New component |
| `components/leaderboard/crown-badge.tsx` | New component |
| `app/segments/[id]/leaderboard/page.tsx` | New page |
| `app/profile/settings/page.tsx` | New page |
| `app/profile/achievements/page.tsx` | New page |
| `app/profile/rankings/page.tsx` | New page |
| `app/leaderboards/page.tsx` | New page |

---

## Test Data

The existing test user (esims89+1@gmail.com) has:
- 3 segments with efforts (verdi climb, pvc, pea climb)
- All efforts are PRs
- Can be used to test leaderboard functionality

To test demographic filters:
1. Update test user demographics via `/profile/settings`
2. Create additional test users with different demographics
3. Upload activities to generate varied leaderboard data

---

## Next Steps

1. Implement SSE endpoint for real-time leaderboard updates
2. Integrate achievement processing into activity_queue
3. Add notification system for crown changes
4. Complete manual testing of all new pages
5. Consider Phase 5 (Social) after Phase 4 completion
