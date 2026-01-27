# Phase 4: Leaderboards

**Duration:** Month 4 (4 weeks)
**Goal:** Build the competitive ranking system that makes segments compelling

> **Claude Agents:** Use `/feature-dev` for leaderboard service, caching, and real-time updates. Use `/frontend-design` for leaderboard tables and filter UI.

---

## Objectives

1. Comprehensive segment leaderboards
2. Demographic filters (age, gender, weight class)
3. Time-based filters (all-time, year, month, week)
4. KOM/QOM crowns and achievements
5. Real-time leaderboard updates

---

## Week 1: Leaderboard Infrastructure

### 1.1 Database Schema

**Tasks:**
- [ ] Add demographic fields to users
- [ ] Create leaderboard cache table
- [ ] Create achievements table
- [ ] Run migrations

**Schema:**
```sql
-- Add demographics to users
ALTER TABLE users ADD COLUMN gender TEXT;  -- 'male', 'female', 'other', 'prefer_not_to_say'
ALTER TABLE users ADD COLUMN birth_year INTEGER;
ALTER TABLE users ADD COLUMN weight_kg FLOAT;
ALTER TABLE users ADD COLUMN country TEXT;
ALTER TABLE users ADD COLUMN region TEXT;  -- state/province

-- Leaderboard cache (materialized view alternative)
CREATE TABLE leaderboard_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    segment_id UUID NOT NULL REFERENCES segments(id),
    scope TEXT NOT NULL,  -- 'all_time', 'year:2026', 'month:2026-01', 'week:2026-W04'
    filter_key TEXT NOT NULL DEFAULT 'all',  -- 'all', 'male', 'female', 'age:25-34', etc.
    entries JSONB NOT NULL,  -- Array of {user_id, effort_id, rank, time}
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE (segment_id, scope, filter_key)
);

CREATE INDEX idx_lb_cache_segment ON leaderboard_cache(segment_id);
CREATE INDEX idx_lb_cache_expires ON leaderboard_cache(expires_at);

-- Achievements/Crowns
CREATE TABLE achievements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    segment_id UUID NOT NULL REFERENCES segments(id),
    achievement_type TEXT NOT NULL,  -- 'kom', 'qom', 'local_legend', 'pr'
    scope TEXT NOT NULL,  -- 'all_time', 'year:2026', etc.
    effort_id UUID REFERENCES segment_efforts(id),
    achieved_at TIMESTAMPTZ NOT NULL,
    lost_at TIMESTAMPTZ,  -- When dethroned
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_achievements_user ON achievements(user_id);
CREATE INDEX idx_achievements_segment ON achievements(segment_id);
CREATE INDEX idx_achievements_active ON achievements(segment_id, achievement_type) WHERE lost_at IS NULL;
```

### 1.2 Leaderboard Service

**Tasks:**
- [ ] Create leaderboard computation service
- [ ] Query efforts with filters
- [ ] Rank by elapsed_time
- [ ] Cache results
- [ ] Handle cache invalidation

**Core Query:**
```sql
SELECT
    se.id as effort_id,
    se.user_id,
    se.elapsed_time,
    se.started_at,
    u.name as user_name,
    u.avatar_url,
    ROW_NUMBER() OVER (ORDER BY se.elapsed_time ASC) as rank
FROM segment_efforts se
JOIN users u ON se.user_id = u.id
WHERE se.segment_id = $1
  AND se.started_at >= $2  -- Time filter
  AND (u.gender = $3 OR $3 IS NULL)  -- Gender filter
  AND (EXTRACT(YEAR FROM NOW()) - u.birth_year BETWEEN $4 AND $5 OR $4 IS NULL)  -- Age filter
ORDER BY se.elapsed_time ASC
LIMIT $6;
```

### 1.3 Cache Strategy

**Tasks:**
- [ ] Define cache TTLs per scope
- [ ] Implement cache warming
- [ ] Background cache refresh
- [ ] On-demand computation fallback

**Cache TTLs:**
| Scope | TTL | Refresh |
|-------|-----|---------|
| Week | 5 minutes | On effort |
| Month | 15 minutes | Every 10 min |
| Year | 1 hour | Every 30 min |
| All-time | 1 hour | Every 30 min |

---

## Week 2: Leaderboard UI

### 2.1 Leaderboard Page

**Tasks:**
- [ ] Create `/segments/[id]/leaderboard` route
- [ ] Display leaderboard table
- [ ] Show rank, user, time, date
- [ ] Pagination (top 100, load more)
- [ ] Highlight current user

**Table Columns:**
| Column | Content |
|--------|---------|
| Rank | 1, 2, 3... (medal icons for top 3) |
| User | Avatar, name (link to profile) |
| Time | Formatted duration |
| Speed | Average speed |
| Date | When achieved |
| PRs | Personal record indicator |

### 2.2 Filter Controls

**Tasks:**
- [ ] Time scope dropdown (All, Year, Month, Week)
- [ ] Gender filter (All, Men, Women)
- [ ] Age group filter (All, 18-24, 25-34, 35-44, 45-54, 55-64, 65+)
- [ ] Weight class filter (if applicable)
- [ ] Region filter (All, Country, State)
- [ ] Persist filters in URL

### 2.3 Personal Position

**Tasks:**
- [ ] Show user's current rank
- [ ] "Jump to my position" button
- [ ] PR indicator if applicable
- [ ] Time gap to positions above/below

### 2.4 Real-time Updates

**Tasks:**
- [ ] Implement Server-Sent Events (SSE)
- [ ] Subscribe to leaderboard changes
- [ ] Animate rank changes
- [ ] Toast notification for new leaders

**Backend:**
```rust
#[derive(Clone)]
pub struct LeaderboardUpdate {
    pub segment_id: Uuid,
    pub scope: String,
    pub filter_key: String,
    pub new_leader: Option<LeaderboardEntry>,
    pub changes: Vec<RankChange>,
}

// SSE endpoint
pub async fn leaderboard_stream(
    Path(segment_id): Path<Uuid>,
    Query(filters): Query<LeaderboardFilters>,
) -> Sse<impl Stream<Item = Event>> {
    // Subscribe to updates for this segment
}
```

---

## Week 3: KOM/QOM System

### 3.1 Crown Logic

**Tasks:**
- [ ] Define KOM (King of Mountain) criteria
- [ ] Define QOM (Queen of Mountain) criteria
- [ ] Award crowns on new efforts
- [ ] Handle dethroning
- [ ] Track crown history

**Crown Types:**
| Crown | Criteria |
|-------|----------|
| KOM | Fastest time, all-time, men |
| QOM | Fastest time, all-time, women |
| Local Legend | Most efforts on segment in past 90 days |
| Course Record | Fastest time, all-time, any gender |

### 3.2 Achievement Notifications

**Tasks:**
- [ ] Create notification system
- [ ] Notify on crown achievement
- [ ] Notify on dethroning
- [ ] Notify on PR
- [ ] In-app notification center

### 3.3 Crown Display

**Tasks:**
- [ ] Crown icons on user profiles
- [ ] Crown count on profile page
- [ ] Crown gallery/list
- [ ] Historical crowns (with dates)
- [ ] Crown on segment leaderboard

### 3.4 Local Legend

**Tasks:**
- [ ] Count efforts per user per segment (90-day window)
- [ ] Award "Local Legend" to most efforts
- [ ] Display on segment page
- [ ] Historical tracking

---

## Week 4: Advanced Features

### 4.1 Personal Rankings

**Tasks:**
- [ ] Create `/profile/[username]/rankings` page
- [ ] Show all segments with rankings
- [ ] Filter by activity type
- [ ] Sort by rank, improvement potential
- [ ] PR history per segment

### 4.2 Comparison Feature

**Tasks:**
- [ ] Compare two users on a segment
- [ ] Show side-by-side stats
- [ ] Highlight winner per metric
- [ ] Historical comparison chart

### 4.3 Improvement Suggestions

**Tasks:**
- [ ] Calculate "close" rankings (within 5%)
- [ ] Show segments where user could move up
- [ ] Estimate time needed to improve
- [ ] "Challenge" button to save goal

### 4.4 Global Leaderboards

**Tasks:**
- [ ] Aggregate user stats across segments
- [ ] Total crown count leaderboard
- [ ] Total distance leaderboard
- [ ] Total elevation leaderboard
- [ ] Activity count leaderboard

---

## Deliverables

### End of Phase 4 Checklist

- [ ] Segment leaderboards display correctly
- [ ] Time filters work (all-time, year, month, week)
- [ ] Demographic filters work
- [ ] KOM/QOM crowns awarded
- [ ] Dethroning notifications work
- [ ] Real-time updates functional
- [ ] Personal rankings page
- [ ] Global leaderboards
- [ ] Crown gallery on profiles

### API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/segments/{id}/leaderboard` | No | Get leaderboard |
| GET | `/segments/{id}/leaderboard/stream` | Yes | SSE for updates |
| GET | `/users/{id}/rankings` | Mixed | User's segment rankings |
| GET | `/users/{id}/achievements` | Mixed | User's crowns/achievements |
| GET | `/leaderboards/global/crowns` | No | Crown count rankings |
| GET | `/leaderboards/global/distance` | No | Total distance rankings |

### Query Parameters for Leaderboard

| Parameter | Values | Description |
|-----------|--------|-------------|
| scope | `all_time`, `year`, `month`, `week` | Time scope |
| gender | `all`, `male`, `female` | Gender filter |
| age_group | `all`, `18-24`, `25-34`, etc. | Age filter |
| region | Country/region code | Location filter |
| limit | 1-100 | Results per page |
| offset | 0+ | Pagination offset |

---

## Performance Considerations

### Caching Strategy

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Request    │────►│    Cache     │────►│   Response   │
└──────────────┘     └──────────────┘     └──────────────┘
                           │ miss
                           ▼
                    ┌──────────────┐
                    │   Compute    │
                    │ (background) │
                    └──────────────┘
```

### Cache Invalidation

- On new effort: Invalidate affected scopes
- On user demographic change: Invalidate filtered views
- Batch invalidation for bulk imports

### Read vs Write Optimization

- Optimize for reads (leaderboards viewed 1000x more than updated)
- Async effort processing
- Eventual consistency acceptable (seconds)

---

## Real-time Architecture

### SSE Implementation

```
Client                     Server
  │                          │
  │──GET /leaderboard/stream─►│
  │                          │
  │◄──event: connected───────│
  │                          │
  │                          │◄── New effort
  │                          │
  │◄──event: update──────────│
  │  {rank_changes: [...]}   │
  │                          │
```

### Event Types

| Event | Payload | Trigger |
|-------|---------|---------|
| `connected` | Current top 10 | On connect |
| `new_leader` | Leader info | Crown change |
| `rank_change` | Changes array | New effort affects visible ranks |
| `your_rank` | User's new rank | User's effort processed |

---

## Success Criteria

1. **Rankings work:** Correct ordering by time
2. **Filters work:** Demographics filter correctly
3. **Scopes work:** Time-based filtering accurate
4. **Crowns work:** Awarded and revoked correctly
5. **Real-time works:** Updates appear within seconds
6. **Performance works:** Leaderboard loads < 200ms
