# Phase 3: Segments

**Duration:** Month 3 (4-5 weeks)
**Goal:** Implement the core segment system - the heart of Track Leader

> **Claude Agents:** Use `/feature-dev` for segment matching algorithm and PostGIS queries. Use `/frontend-design` for segment creation UI and discovery pages.

---

## Objectives

1. Segment creation from activity portions
2. Automatic segment matching on activity upload
3. Segment detail pages with effort history
4. Personal records tracking
5. Segment discovery and search

---

## Week 1: Segment Data Model

### 1.1 Database Schema

**Tasks:**
- [x] Create segments table
- [x] Create segment_efforts table
- [x] Create segment_stars table
- [x] Add spatial indexes
- [x] Run migration

**Schema:**
```sql
CREATE TABLE segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    description TEXT,
    activity_type activity_type NOT NULL,
    geo GEOGRAPHY(LineString, 4326) NOT NULL,
    distance FLOAT NOT NULL,
    elevation_gain FLOAT,
    elevation_loss FLOAT,
    average_grade FLOAT,
    max_grade FLOAT,
    climb_category INTEGER,  -- 0=NC, 1-5, 5=HC
    is_hazardous BOOLEAN DEFAULT false,
    is_public BOOLEAN DEFAULT true,
    star_count INTEGER DEFAULT 0,
    effort_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_segments_geo ON segments USING GIST (geo);
CREATE INDEX idx_segments_creator ON segments(creator_id);
CREATE INDEX idx_segments_type ON segments(activity_type);

CREATE TABLE segment_efforts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    segment_id UUID NOT NULL REFERENCES segments(id),
    activity_id UUID NOT NULL REFERENCES activities(id),
    user_id UUID NOT NULL REFERENCES users(id),
    elapsed_time FLOAT NOT NULL,
    moving_time FLOAT,
    start_index INTEGER NOT NULL,
    end_index INTEGER NOT NULL,
    average_speed FLOAT,
    max_speed FLOAT,
    average_hr INTEGER,
    max_hr INTEGER,
    average_power INTEGER,
    pr_rank INTEGER,  -- 1, 2, 3 for personal top 3
    started_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_efforts_segment ON segment_efforts(segment_id);
CREATE INDEX idx_efforts_user ON segment_efforts(user_id);
CREATE INDEX idx_efforts_segment_time ON segment_efforts(segment_id, elapsed_time);
CREATE INDEX idx_efforts_user_segment ON segment_efforts(user_id, segment_id);

CREATE TABLE segment_stars (
    user_id UUID NOT NULL REFERENCES users(id),
    segment_id UUID NOT NULL REFERENCES segments(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, segment_id)
);
```

### 1.2 Backend Models

**Tasks:**
- [x] Create Segment model
- [x] Create SegmentEffort model
- [x] Add database methods
- [x] Add PostGIS query helpers

### 1.3 Segment Metrics Calculation

**Tasks:**
- [x] Calculate distance from LineString
- [x] Calculate elevation gain/loss
- [ ] Calculate average/max grade
- [ ] Determine climb category
- [x] Pre-compute on segment creation

**Climb Categories:**
| Category | Points | Criteria |
|----------|--------|----------|
| 4 | 20-39 | Short climbs |
| 3 | 40-79 | Medium climbs |
| 2 | 80-159 | Significant climbs |
| 1 | 160-319 | Major climbs |
| HC | 320+ | Epic climbs |

Points = elevation_gain * length_km * grade_factor

---

## Week 2: Segment Creation

### 2.1 Segment Creation UI

**Tasks:**
- [x] Add "Create Segment" button to activity detail
- [x] Create segment editor component
- [x] Allow selecting start/end points on elevation profile
- [x] Preview segment as user selects
- [ ] Show calculated metrics in real-time
- [x] Name and description input
- [x] Submit segment creation

**UI Flow:**
1. User views activity
2. Clicks "Create Segment"
3. Map enters selection mode
4. User clicks start point
5. User clicks end point (or drags range)
6. Preview shows segment stats
7. User enters name, description
8. User submits

### 2.2 Backend Segment Creation

**Tasks:**
- [x] Implement `POST /segments` endpoint
- [x] Extract segment geometry from activity
- [x] Calculate segment metrics
- [x] Store in database
- [x] Return created segment

**Request:**
```typescript
interface CreateSegmentRequest {
  activity_id: string;
  name: string;
  description?: string;
  start_index: number;
  end_index: number;
}
```

### 2.3 Segment Validation

**Tasks:**
- [ ] Minimum segment length (100m)
- [ ] Maximum segment length (50km)
- [ ] Minimum point count (10 points)
- [ ] No duplicate segments (fuzzy match)
- [ ] Activity type inheritance

### 2.4 Creator's First Effort

**Tasks:**
- [x] Auto-create first effort from creator's activity
- [x] Set as baseline for segment
- [x] Calculate PR rank

---

## Week 3: Segment Matching

### 3.1 Matching Algorithm

**Tasks:**
- [x] Implement segment matching service
- [x] Use PostGIS spatial queries
- [x] Define match criteria:
  - Start point within tolerance (50m)
  - End point within tolerance (50m)
  - Direction verification (start before end)
- [x] Return matched segments with fractional positions

**Algorithm Overview:**
```
1. Find candidate segments near activity bounding box
2. For each candidate:
   a. Find closest activity point to segment start
   b. Find closest activity point to segment end
   c. Check path similarity (Fréchet distance or similar)
   d. If above threshold, record match
3. Return matches with start/end indices
```

### 3.2 PostGIS Queries

**Key Queries:**
```sql
-- Find segments near activity
SELECT s.* FROM segments s
WHERE ST_DWithin(
    s.geo,
    (SELECT geo FROM tracks WHERE activity_id = $1),
    100  -- meters
);

-- Find closest point on activity to segment start
SELECT
    ST_LineLocatePoint(t.geo, ST_StartPoint(s.geo)) as position,
    ST_Distance(t.geo, ST_StartPoint(s.geo)) as distance
FROM tracks t, segments s
WHERE t.activity_id = $1 AND s.id = $2;
```

### 3.3 Background Matching Job

**Tasks:**
- [x] Integrate matching into activity queue
- [x] After scoring, run segment matching
- [x] Create segment_efforts for matches
- [x] Calculate PR ranks (update_personal_records)
- [ ] Update segment effort_count (cached counter)

### 3.4 Effort Calculation

**Tasks:**
- [x] Extract time from start_index to end_index (via fractional position interpolation)
- [ ] Calculate moving time (exclude stops)
- [ ] Calculate average/max speed for segment
- [x] Determine PR rank (compare to user's other efforts)

---

## Week 4: Segment UI

### 4.1 Segment Detail Page

**Tasks:**
- [x] Create `/segments/[id]` route
- [x] Display segment on map
- [x] Show segment statistics
- [x] Show elevation profile
- [x] Display creator info
- [x] Star/unstar button

### 4.2 Segment Leaderboard Preview

**Tasks:**
- [x] Show top 10 efforts on segment page
- [x] Display rank, user, time, date
- [ ] Link to full leaderboard (Phase 4)
- [x] Highlight current user's position

### 4.3 Personal Efforts

**Tasks:**
- [ ] Show user's efforts on segment
- [ ] PR history chart
- [ ] Effort comparison
- [x] Link to source activity (click leaderboard row)

### 4.4 Segment on Activity

**Tasks:**
- [x] Show matched segments on activity detail
- [x] Display segment time and rank
- [x] Link to segment page
- [ ] Highlight segment on map

---

## Week 5: Segment Discovery

### 5.1 Segment Browser

**Tasks:**
- [x] Create `/segments` route
- [ ] Map-based segment discovery
- [ ] Show segments as clickable lines
- [ ] Clustering for dense areas
- [ ] Popup on hover

### 5.2 Segment Search

**Tasks:**
- [x] Search by name
- [x] Filter by activity type
- [ ] Filter by distance range
- [ ] Filter by climb category
- [ ] Sort by popularity, distance, elevation

### 5.3 Nearby Segments

**Tasks:**
- [ ] "Segments near me" feature
- [ ] Request location permission
- [ ] Find segments within radius
- [ ] Show on map and list

### 5.4 Starred Segments

**Tasks:**
- [x] Star/unstar functionality
- [x] Starred segments page (tab on /segments)
- [ ] Track starred segment efforts

---

## Deliverables

### End of Phase 3 Checklist

- [x] Segments can be created from activities
- [x] Segment matching runs on upload
- [x] Segment efforts calculated correctly
- [x] PR tracking working
- [x] Segment detail page complete
- [ ] Segment browser/search working (basic list done, no search/filter)
- [x] Starred segments feature (API + button on detail page)
- [x] Segments shown on activity detail

### API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/segments` | Yes | Create segment |
| GET | `/segments` | No | List/search segments |
| GET | `/segments/{id}` | No | Get segment detail |
| PATCH | `/segments/{id}` | Yes | Update segment (creator only) |
| DELETE | `/segments/{id}` | Yes | Delete segment (creator only) |
| POST | `/segments/{id}/star` | Yes | Star segment |
| DELETE | `/segments/{id}/star` | Yes | Unstar segment |
| GET | `/segments/{id}/efforts` | No | Get segment efforts |
| GET | `/segments/nearby` | No | Find nearby segments |
| GET | `/users/{id}/starred-segments` | Mixed | User's starred segments |

---

## Matching Algorithm Details

### Tolerance Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Start tolerance | 50m | How close activity must be to segment start |
| End tolerance | 50m | How close activity must be to segment end |
| Path tolerance | 30m | How far activity can deviate from segment |
| Min overlap | 90% | Minimum segment coverage required |

### Match Quality Score

```
score = (
    0.4 * start_proximity_score +
    0.4 * end_proximity_score +
    0.2 * path_similarity_score
)
```

Only create effort if score > 0.8

### Edge Cases

| Case | Handling |
|------|----------|
| Activity crosses segment multiple times | Create effort for best (fastest) crossing |
| Activity goes wrong direction | No match (direction matters) |
| Activity stops mid-segment | Create effort if > 90% complete |
| GPS drift | Use path tolerance buffer |

---

## Performance Considerations

### Matching Performance
- Use spatial indexes effectively
- Limit candidate segments with bounding box
- Parallelize matching with Rayon
- Cache frequently-matched segments

### Storage Considerations
- Simplify segment geometry for storage
- Store detailed geometry separately
- Index by activity type for faster queries

---

## Success Criteria

1. **Creation works:** Can create segment from any activity
2. **Matching works:** Activities match to relevant segments
3. **Efforts work:** Times calculated correctly
4. **PRs work:** Personal records tracked
5. **Discovery works:** Can find segments on map and search
6. **Stars work:** Can favorite segments

---

## Implementation Notes (Added 2026-01-26)

### Segment Matching Implementation

The segment matching system is implemented with the following architecture:

**Database Layer (`database.rs`):**
- `save_track_geometry()` - Stores activity track as LINESTRING with upsert
- `find_matching_segments()` - Finds segments that activity passes through
- `find_matching_activities_for_segment()` - Inverse: finds activities for a segment
- `segment_effort_exists()` - Idempotency check
- `update_personal_records()` - Marks fastest effort as PR

**Matching Algorithm:**
```sql
-- Core matching query
SELECT s.id, s.distance_meters,
       ST_LineLocatePoint(t.geo::geometry, s.start_point::geometry) as start_pos,
       ST_LineLocatePoint(t.geo::geometry, s.end_point::geometry) as end_pos
FROM segments s
JOIN tracks t ON t.activity_id = $1
WHERE ST_DWithin(t.geo, s.start_point, 50)  -- 50m tolerance
  AND ST_DWithin(t.geo, s.end_point, 50)
  AND ST_LineLocatePoint(...) < ST_LineLocatePoint(...)  -- direction check
```

**Timing Extraction (`segment_matching.rs`):**
- Calculates cumulative distance along track
- Normalizes to fractions (0-1)
- Interpolates GPX timestamps at start/end fractions
- Returns `SegmentTiming { started_at, elapsed_time_seconds }`

**Key Design Decisions:**
1. **50m tolerance** - Balances accuracy with GPS variance
2. **Direction verification** - Ensures segment traversed in correct direction
3. **Fractional position** - Uses ST_LineLocatePoint for precise timing extraction
4. **Idempotency** - Checks for existing efforts before creating duplicates
5. **Auto-reprocess** - When segment created, automatically finds matching activities

**API Endpoints:**
- `POST /segments` - Now auto-creates efforts for matching activities
- `POST /segments/{id}/reprocess` - Manual reprocess trigger

**Files Added:**
- `migrations/005_tracks_spatial_index.sql` - GIST index + unique constraint
- `segment_matching.rs` - SegmentMatch, ActivityMatch, timing extraction

---

## Known Issues

### FIXED: Original activity not counted as segment effort on segment creation

**Reported:** 2026-01-26
**Fixed:** 2026-01-26

**Symptom:** When creating a segment from an activity, the original activity is not automatically counted as a segment effort.

**Root cause:** Activities uploaded before the track storage feature was implemented don't have their tracks in the `tracks` table. The spatial matching query requires tracks to be in the database.

**Fix implemented:**
1. Added `source_activity_id` to `CreateSegmentRequest` (optional field)
2. When provided, the handler checks if the track exists in the `tracks` table
3. If not, it loads the GPX, builds the WKT, and saves the track geometry
4. Then the spatial matching query finds the activity as expected
5. Frontend updated to pass `source_activity_id` when creating segments

**Files changed:**
- `handlers.rs` - Added source_activity_id handling and build_track_wkt helper
- `src/lib/api.ts` - Added source_activity_id to CreateSegmentRequest interface
- `src/app/activities/[id]/page.tsx` - Pass activity.id as source_activity_id
