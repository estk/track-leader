# Database Architecture

## Current Schema

PostgreSQL 15 with PostGIS extension.

### Entity Relationship Diagram (Current)

```
┌──────────────────┐
│      users       │
├──────────────────┤
│ id (UUID) PK     │
│ email (UNIQUE)   │
│ name             │
│ created_at       │
└────────┬─────────┘
         │
         │ 1:N
         ▼
┌──────────────────┐       ┌──────────────────┐
│   activities     │       │     scores       │
├──────────────────┤       ├──────────────────┤
│ id (UUID) PK     │──────►│ id (UUID) PK     │
│ user_id          │  1:1  │ user_id          │
│ activity_type    │       │ activity_id      │
│ name             │       │ distance         │
│ object_store_path│       │ duration         │
│ submitted_at     │       │ elevation_gain   │
└────────┬─────────┘       │ created_at       │
         │                 └──────────────────┘
         │ 1:1 (UNUSED)
         ▼
┌──────────────────┐
│     tracks       │
├──────────────────┤
│ id (UUID) PK     │
│ user_id          │
│ activity_id      │
│ geo (GEOGRAPHY)  │
│ created_at       │
└──────────────────┘
```

### Tables in Detail

#### `users`
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

**Notes:**
- Minimal user model (no password hash - auth not implemented)
- No profile fields (bio, avatar, location)
- No settings/preferences

#### `activities`
```sql
CREATE TYPE activity_type AS ENUM (
    'walking', 'running', 'hiking',
    'road_cycling', 'mountain_biking', 'unknown'
);

CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    activity_type activity_type NOT NULL,
    name TEXT NOT NULL,
    object_store_path TEXT NOT NULL,
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_activities_user_id ON activities(user_id);
CREATE INDEX idx_activities_submitted_at ON activities(submitted_at);
```

**Notes:**
- No foreign key constraint to users (should have)
- No description/notes field
- No visibility (public/private/followers)
- No gear tracking

#### `tracks`
```sql
CREATE TABLE tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    activity_id UUID NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    geo GEOGRAPHY(LineStringZM, 4326) NOT NULL
);

CREATE INDEX idx_tracks_user_id ON tracks(user_id);
CREATE INDEX idx_tracks_activity_id ON tracks(activity_id);
CREATE INDEX idx_tracks_geo ON tracks USING GIST (geo);
```

**Notes:**
- Uses **LineStringZM** (4D geometry): X=longitude, Y=latitude, Z=elevation(m), M=timestamp(unix epoch)
- PostGIS GEOGRAPHY type for spherical calculations
- GIST spatial index for segment matching
- Populated during activity upload via background queue
- Track data retrieved from database, not re-parsed from GPX files

#### `activity_sensor_data`
```sql
CREATE TABLE activity_sensor_data (
    activity_id UUID PRIMARY KEY REFERENCES activities(id) ON DELETE CASCADE,
    heart_rates int[],
    cadences int[],
    powers int[],
    temperatures double precision[]
);
```

**Notes:**
- Arrays parallel to track geometry points (index 0 = point 0)
- Populated when importing FIT/TCX files (future feature)
- Single row per activity for efficient bulk reads

#### `scores`
```sql
CREATE TABLE scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    activity_id UUID NOT NULL,
    distance FLOAT NOT NULL,
    duration FLOAT NOT NULL,
    elevation_gain FLOAT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scores_user_id ON scores(user_id);
CREATE INDEX idx_scores_activity_id ON scores(activity_id);
```

**Notes:**
- Populated by background queue
- Missing elevation_loss
- Missing average/max speed
- Missing heart rate stats (if available in GPX)

---

## Proposed Schema Evolution

### Phase 1: Fix Fundamentals

```sql
-- Add foreign key constraints
ALTER TABLE activities
    ADD CONSTRAINT fk_activities_user
    FOREIGN KEY (user_id) REFERENCES users(id);

ALTER TABLE tracks
    ADD CONSTRAINT fk_tracks_user
    FOREIGN KEY (user_id) REFERENCES users(id),
    ADD CONSTRAINT fk_tracks_activity
    FOREIGN KEY (activity_id) REFERENCES activities(id);

ALTER TABLE scores
    ADD CONSTRAINT fk_scores_user
    FOREIGN KEY (user_id) REFERENCES users(id),
    ADD CONSTRAINT fk_scores_activity
    FOREIGN KEY (activity_id) REFERENCES activities(id);

-- Add spatial index for segment matching
CREATE INDEX idx_tracks_geo ON tracks USING GIST (geo);

-- Expand user model
ALTER TABLE users ADD COLUMN
    password_hash TEXT,
    avatar_url TEXT,
    bio TEXT,
    location TEXT,
    is_public BOOLEAN DEFAULT true,
    created_via TEXT,  -- 'email', 'google', 'strava'
    updated_at TIMESTAMP WITH TIME ZONE;

-- Expand activity model
ALTER TABLE activities ADD COLUMN
    description TEXT,
    visibility TEXT DEFAULT 'public',  -- 'public', 'followers', 'private'
    gear_id UUID,
    started_at TIMESTAMP WITH TIME ZONE,
    timezone TEXT;
```

### Phase 2: Segments

```sql
-- Segments are user-defined portions of trail
CREATE TABLE segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    description TEXT,
    activity_type activity_type NOT NULL,
    geo GEOGRAPHY(LineString, 4326) NOT NULL,
    distance FLOAT NOT NULL,  -- pre-calculated
    elevation_gain FLOAT,
    elevation_loss FLOAT,
    climb_category INTEGER,  -- 0-5 (HC)
    is_hazardous BOOLEAN DEFAULT false,
    is_public BOOLEAN DEFAULT true,
    star_count INTEGER DEFAULT 0,
    effort_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_segments_geo ON segments USING GIST (geo);
CREATE INDEX idx_segments_creator ON segments(creator_id);
CREATE INDEX idx_segments_activity_type ON segments(activity_type);

-- Segment efforts (user attempts on segments)
CREATE TABLE segment_efforts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    segment_id UUID NOT NULL REFERENCES segments(id),
    activity_id UUID NOT NULL REFERENCES activities(id),
    user_id UUID NOT NULL REFERENCES users(id),
    elapsed_time FLOAT NOT NULL,  -- seconds
    moving_time FLOAT,
    start_index INTEGER NOT NULL,  -- trackpoint index in activity
    end_index INTEGER NOT NULL,
    average_speed FLOAT,
    max_speed FLOAT,
    average_hr INTEGER,
    max_hr INTEGER,
    average_power INTEGER,
    pr_rank INTEGER,  -- 1, 2, 3 or NULL
    kom_rank INTEGER,  -- overall ranking at time of effort
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_efforts_segment ON segment_efforts(segment_id);
CREATE INDEX idx_efforts_activity ON segment_efforts(activity_id);
CREATE INDEX idx_efforts_user ON segment_efforts(user_id);
CREATE INDEX idx_efforts_time ON segment_efforts(segment_id, elapsed_time);

-- Segment stars (favorites)
CREATE TABLE segment_stars (
    user_id UUID NOT NULL REFERENCES users(id),
    segment_id UUID NOT NULL REFERENCES segments(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, segment_id)
);
```

### Phase 3: Trails

```sql
-- Trails are named routes that can span multiple segments
CREATE TABLE trails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id UUID REFERENCES users(id),  -- NULL for system-created
    name TEXT NOT NULL,
    description TEXT,
    region TEXT,
    country TEXT,
    difficulty TEXT,  -- 'easy', 'moderate', 'hard', 'expert'
    surface_type TEXT,  -- 'paved', 'gravel', 'dirt', 'technical'
    is_loop BOOLEAN DEFAULT false,
    geo GEOGRAPHY(LineString, 4326) NOT NULL,
    distance FLOAT NOT NULL,
    elevation_gain FLOAT,
    estimated_time INTEGER,  -- minutes
    star_count INTEGER DEFAULT 0,
    completion_count INTEGER DEFAULT 0,
    is_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_trails_geo ON trails USING GIST (geo);
CREATE INDEX idx_trails_region ON trails(region);

-- Trail segments junction
CREATE TABLE trail_segments (
    trail_id UUID NOT NULL REFERENCES trails(id),
    segment_id UUID NOT NULL REFERENCES segments(id),
    sequence INTEGER NOT NULL,  -- order in trail
    PRIMARY KEY (trail_id, segment_id)
);

-- Trail completions
CREATE TABLE trail_completions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trail_id UUID NOT NULL REFERENCES trails(id),
    user_id UUID NOT NULL REFERENCES users(id),
    activity_id UUID NOT NULL REFERENCES activities(id),
    elapsed_time FLOAT NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_completions_trail ON trail_completions(trail_id);
CREATE INDEX idx_completions_user ON trail_completions(user_id);
```

### Phase 4: Leaderboards

```sql
-- Leaderboard definitions
CREATE TABLE leaderboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    segment_id UUID REFERENCES segments(id),
    trail_id UUID REFERENCES trails(id),
    scope TEXT NOT NULL,  -- 'all_time', 'year', 'month', 'week'
    gender_filter TEXT,  -- 'all', 'male', 'female'
    age_group TEXT,  -- 'all', '18-24', '25-34', etc.
    region_filter TEXT,
    activity_type activity_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT one_target CHECK (
        (segment_id IS NOT NULL AND trail_id IS NULL) OR
        (segment_id IS NULL AND trail_id IS NOT NULL)
    )
);

-- Cached leaderboard entries (rebuilt periodically)
CREATE TABLE leaderboard_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leaderboard_id UUID NOT NULL REFERENCES leaderboards(id),
    user_id UUID NOT NULL REFERENCES users(id),
    effort_id UUID,  -- segment_effort or trail_completion
    rank INTEGER NOT NULL,
    elapsed_time FLOAT NOT NULL,
    achieved_at TIMESTAMP WITH TIME ZONE NOT NULL,
    cached_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lb_entries_leaderboard ON leaderboard_entries(leaderboard_id);
CREATE INDEX idx_lb_entries_rank ON leaderboard_entries(leaderboard_id, rank);
CREATE UNIQUE INDEX idx_lb_entries_user_lb ON leaderboard_entries(leaderboard_id, user_id);
```

### Phase 5: Social

```sql
-- User follows
CREATE TABLE follows (
    follower_id UUID NOT NULL REFERENCES users(id),
    following_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (follower_id, following_id)
);

CREATE INDEX idx_follows_following ON follows(following_id);

-- Activity kudos
CREATE TABLE kudos (
    user_id UUID NOT NULL REFERENCES users(id),
    activity_id UUID NOT NULL REFERENCES activities(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, activity_id)
);

CREATE INDEX idx_kudos_activity ON kudos(activity_id);

-- Comments
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    activity_id UUID NOT NULL REFERENCES activities(id),
    parent_id UUID REFERENCES comments(id),  -- for replies
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_comments_activity ON comments(activity_id);
```

### Phase 6: Custom Metrics

```sql
-- User-defined metrics
CREATE TABLE metric_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id UUID REFERENCES users(id),  -- NULL for system metrics
    name TEXT NOT NULL,
    description TEXT,
    formula TEXT NOT NULL,  -- e.g., "distance / duration"
    unit TEXT,
    is_public BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Metric values for activities
CREATE TABLE activity_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_id UUID NOT NULL REFERENCES activities(id),
    metric_id UUID NOT NULL REFERENCES metric_definitions(id),
    value FLOAT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (activity_id, metric_id)
);

CREATE INDEX idx_activity_metrics_activity ON activity_metrics(activity_id);
CREATE INDEX idx_activity_metrics_metric ON activity_metrics(metric_id);
```

---

## PostGIS Considerations

### Spatial Operations Needed

| Operation | PostGIS Function | Use Case |
|-----------|-----------------|----------|
| Segment matching | `ST_DWithin()` | Find segments near activity track |
| Distance calculation | `ST_Length()` | Calculate segment/trail distance |
| Point extraction | `ST_PointN()` | Get specific trackpoints |
| Line intersection | `ST_Intersection()` | Find where activity crosses segment |
| Simplification | `ST_Simplify()` | Reduce point density for display |
| Bounds | `ST_Envelope()` | Get bounding box for map display |

### Indexing Strategy

```sql
-- GIST indexes for spatial queries
CREATE INDEX idx_tracks_geo ON tracks USING GIST (geo);
CREATE INDEX idx_segments_geo ON segments USING GIST (geo);
CREATE INDEX idx_trails_geo ON trails USING GIST (geo);

-- Compound indexes for common queries
CREATE INDEX idx_efforts_segment_time ON segment_efforts(segment_id, elapsed_time);
CREATE INDEX idx_activities_user_date ON activities(user_id, submitted_at DESC);
```

### Performance Considerations

1. **Track simplification** - Store both full and simplified geometries
2. **Segment matching** - Use spatial index + bounding box pre-filter
3. **Leaderboard caching** - Materialize top-N results
4. **Distance calculations** - Pre-compute on insert, don't calculate on read

---

## Migration Strategy

1. **Backup existing data**
2. **Add columns with defaults** (non-breaking)
3. **Add new tables** (non-breaking)
4. **Backfill data** where possible
5. **Add foreign key constraints** (may require data cleanup)
6. **Add indexes** (can be done concurrently)
7. **Deploy new code** that uses new schema
8. **Remove unused columns/tables** (if any)
