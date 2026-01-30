# Todo

- the visibility of an activity should be easily understandable on the my activities page (pub priv what teams etc)
- auto detect non-moving time on activity and allow user to easily tag as dig on activity upload
- when showing crown leaderboard, dont distinguish between qom vs kom
- need daily map on team home page
- we need to add a heartrate and cadence and power graph
- we need the ability to process tcx and fit files

## Filter Implementation

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
