# Todo

- you should be able to follow users and that should be in your feed
- all activities for user page not found: http://localhost:16542/profile/a79dbe78-7e3c-4b9e-9ec8-fcbd634844ee/activities
- achivements not found http://localhost:16542/profile/a79dbe78-7e3c-4b9e-9ec8-fcbd634844ee/achievements
- no visible change when clicking follow user button

- we need to add a heartrate and cadence and power graph

## Filter Implementation

### Current State

**Already Filtered:**
- ✅ **Segments Page** (`/segments`) - Full filtering (activity type, distance, climb category, sort, search, location)
- ✅ **Segment Leaderboard** (`/segments/[id]/leaderboard`) - Full filtering (time scope, gender, age, weight, country) with URL persistence
- ✅ **Achievements Page** (`/profile/achievements`) - Partial (type toggle, lost/current toggle)

### Priority 1 - High Value

| Page | Proposed Filters |
|------|------------------|
| **Activities** (`/activities`) | Activity type, date range, visibility, sort, search |
| **Activity Feed** (`/feed`) | Activity type, date range, specific users filter |

### Priority 2 - Medium Value

| Page | Proposed Filters |
|------|------------------|
| **Global Leaderboards** (`/leaderboards`) | Activity type, time period (week/month/year/all), country |
| **Rankings** (`/profile/rankings`) | Expand beyond sort-only to include activity type, time period |

### Priority 3 - Lower Value

| Page | Proposed Filters |
|------|------------------|
| **Notifications** (`/notifications`) | Type (follow, kudos, comment, crown), read status |
| **Teams** (`/teams`) | Team size, activity focus, search by name |
| **Followers/Following** (`/profile/[userId]/followers`) | Country, activity level, alphabetical/recent sort |

### Implementation Notes

- Use URL-based filters (like Segment Leaderboard's `useLeaderboardFilters()` pattern) for shareability
- Backend APIs for Activities, Feed, and Global Leaderboards don't support filtering yet - will need either:
  - Client-side filtering (quick but limits dataset size)
  - Backend API extensions (preferred for large datasets)
- Consider creating reusable filter components for common filters (Activity Type, Date Range)
