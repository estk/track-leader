# Todo

- [ ] Simplify activity types when there are two adjacent activitys of the same type

- [~] leaderboards: dig time is still empty, and dig percentage leaderboard should not include 0% users

- [~] I tried to upload a multi-sport activity and request failed:
  - I found this in the console:
  -  [API_ERROR] {"type":"api_error","timestamp":"2026-02-01T23:45:48.768Z","request_id":"48f4df42-948b-4f28-9328-18124120c460","method":"POST","path":"/auth/login","status":401,"duration_ms":59,"error":"Unauthorized"}
  - [API_ERROR] {"type":"api_error","timestamp":"2026-02-01T23:46:54.768Z","request_id":null,"method":"GET","path":"/users/0814f1da-a3a5-4e58-bbe3-78f7a0363790/activities","status":500,"duration_ms":41,"error":"Request failed"}

### Current State

**Fully Filtered:**
- ✅ **Segments Page** (`/segments`) - Full filtering (activity type, distance, climb category, sort, search, location)
- ✅ **Segment Leaderboard** (`/segments/[id]/leaderboard`) - Full filtering (time scope, gender, age, weight, country) with URL persistence
- ✅ **Activities** (`/activities`) - Activity type, date range, visibility, sort by (recent/oldest/distance/duration), search with URL persistence
- ✅ **Activity Feed** (`/feed`) - Activity type, date range
- ✅ **Global Leaderboards** (`/leaderboards`) - Time scope, gender, age group, weight class, country, activity type (crowns only)

**Partially Filtered:**
- ✅ **Achievements Page** (`/profile/achievements`) - Type toggle, lost/current toggle

### Remaining - Lower Priority

| Page | Proposed Filters |
|------|------------------|
| **Rankings** (`/profile/rankings`) | Expand beyond sort-only to include activity type, time period |
| **Notifications** (`/notifications`) | Type (follow, kudos, comment, crown), read status |
| **Teams** (`/teams`) | Team size, activity focus, search by name |
| **Followers/Following** (`/profile/[userId]/followers`) | Country, activity level, alphabetical/recent sort |

### Implementation Notes

- ✅ URL-based filters implemented using `useUrlFilters` hook for shareability
- ✅ Backend APIs extended with full filtering support:
  - `get_user_activities_filtered` - activity type, date range, visibility, sort (joins scores for distance/duration), search
  - `get_activity_feed_filtered` - activity type, date range
  - `get_crown_leaderboard_filtered` / `get_distance_leaderboard_filtered` - demographic filters
- ✅ Reusable `QueryBuilder` module created for dynamic SQL WHERE clauses
- ✅ Filter enums added to models: `DateRangeFilter`, `VisibilityFilter`, `ActivitySortBy`, `SortOrder`


## Deferred Features

These items were planned but deferred for post-launch:

### General
- Strava import
- Mobile app (PWA or native)

### Need mobile app
- Live tracking, based on your privacy prefs (teams, followers etc) you can show your live location to logged in users with the right permissions
- Allow manual logging of events such as shuttle entry/exit, dig start/end

### Real-Time Updates
- SSE for leaderboard updates
- SSE for notification updates

### Perf
- Virtual scrolling for long lists
- Leaderboard caching service
- Rate limiting integration (tower_governor in Cargo.toml)

### Low priority
- User defined activity metrics that allow them to create and share a custom leaderboard
- Screen reader testing
- Sentry error tracking integration
- Internationalization
