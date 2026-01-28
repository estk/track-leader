# Phase 5: Social Features

**Duration:** Month 5 (3-4 weeks)
**Goal:** Build community engagement and social interactions

> **Claude Agents:** Use `/frontend-design` for feed, kudos, and comment UI. Use `/feature-dev` for notification system.

---

## Objectives

### Core (Weeks 1-3)
1. Follow system for users
2. Activity feed from followed users
3. Kudos and comments on activities
4. Notifications system

### Stretch Goals (Week 4+)
5. Teams - Create teams, publish to teams, team feeds
6. Share functionality with social previews

---

## Week 1: Follow System

### 1.1 Database Schema

**Tasks:**
- [ ] Create follows table
- [ ] Create notifications table
- [ ] Add follower counts to users
- [ ] Run migrations

**Schema:**
```sql
-- Migration 011_social_follows.sql
CREATE TABLE follows (
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (follower_id, following_id)
);

CREATE INDEX idx_follows_following ON follows(following_id);
CREATE INDEX idx_follows_follower ON follows(follower_id);

-- Denormalized counts for performance
ALTER TABLE users ADD COLUMN follower_count INTEGER DEFAULT 0;
ALTER TABLE users ADD COLUMN following_count INTEGER DEFAULT 0;

-- Migration 012_notifications.sql
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type TEXT NOT NULL,  -- 'follow', 'kudos', 'comment', 'crown', 'mention'
    actor_id UUID REFERENCES users(id),  -- Who triggered it
    target_type TEXT,  -- 'activity', 'segment', 'comment'
    target_id UUID,
    message TEXT,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_notifications_user ON notifications(user_id, created_at DESC);
CREATE INDEX idx_notifications_unread ON notifications(user_id) WHERE read_at IS NULL;
```

### 1.2 Follow API

**Tasks:**
- [ ] Implement `POST /users/{id}/follow`
- [ ] Implement `DELETE /users/{id}/follow`
- [ ] Implement `GET /users/{id}/followers`
- [ ] Implement `GET /users/{id}/following`
- [ ] Update follower counts on follow/unfollow
- [ ] Create follow notification

### 1.3 Follow UI

**Tasks:**
- [ ] Add follow button to profile pages
- [ ] Show follower/following counts
- [ ] Followers list page
- [ ] Following list page

---

## Week 2: Activity Feed

### 2.1 Feed Algorithm

**Tasks:**
- [ ] Design feed query
- [ ] Include followed users' activities
- [ ] Include segment efforts from followed users
- [ ] Chronological ordering (v1)

**Feed Query:**
```sql
SELECT
    a.id,
    a.user_id,
    a.name,
    a.activity_type,
    a.submitted_at,
    u.name as user_name,
    u.avatar_url,
    s.distance,
    s.duration,
    s.elevation_gain,
    (SELECT COUNT(*) FROM kudos WHERE activity_id = a.id) as kudos_count,
    (SELECT COUNT(*) FROM comments WHERE activity_id = a.id) as comment_count,
    EXISTS(SELECT 1 FROM kudos WHERE activity_id = a.id AND user_id = $1) as user_kudos
FROM activities a
JOIN users u ON a.user_id = u.id
LEFT JOIN scores s ON a.id = s.activity_id
WHERE a.user_id IN (
    SELECT following_id FROM follows WHERE follower_id = $1
)
AND a.visibility = 'public'
AND a.deleted_at IS NULL
ORDER BY a.submitted_at DESC
LIMIT $2 OFFSET $3;
```

### 2.2 Feed UI

**Tasks:**
- [ ] Create feed component
- [ ] Activity cards with:
  - User avatar and name
  - Activity title and stats
  - Map thumbnail
  - Kudos/comment counts
  - Matched segments preview
- [ ] Infinite scroll
- [ ] Pull-to-refresh on mobile

---

## Week 3: Kudos & Comments

### 3.1 Kudos System

**Database:**
```sql
-- Migration 013_kudos_comments.sql
CREATE TABLE kudos (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, activity_id)
);

CREATE INDEX idx_kudos_activity ON kudos(activity_id);

-- Denormalized count
ALTER TABLE activities ADD COLUMN kudos_count INTEGER DEFAULT 0;
```

**Tasks:**
- [ ] Implement `POST /activities/{id}/kudos`
- [ ] Implement `DELETE /activities/{id}/kudos`
- [ ] Implement `GET /activities/{id}/kudos` (list who gave kudos)
- [ ] Update kudos_count on activity
- [ ] Create kudos notification

### 3.2 Kudos UI

**Tasks:**
- [ ] Kudos button on activity cards
- [ ] Kudos button on activity detail
- [ ] Animate on kudos given
- [ ] Show who gave kudos (avatars)
- [ ] Prevent self-kudos

### 3.3 Comments System

**Database:**
```sql
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_comments_activity ON comments(activity_id, created_at);

-- Denormalized count
ALTER TABLE activities ADD COLUMN comment_count INTEGER DEFAULT 0;
```

**Tasks:**
- [ ] Implement `POST /activities/{id}/comments`
- [ ] Implement `GET /activities/{id}/comments`
- [ ] Implement `DELETE /comments/{id}` (author only)
- [ ] Support threaded replies (parent_id)
- [ ] Create comment notification

### 3.4 Comments UI

**Tasks:**
- [ ] Comment section on activity detail
- [ ] Comment input with submit button
- [ ] Display comments threaded
- [ ] Reply button
- [ ] Delete own comments

---

## Week 4: Notifications & Sharing

### 4.1 Notifications System

**Tasks:**
- [ ] Implement `GET /notifications`
- [ ] Implement `POST /notifications/{id}/read`
- [ ] Implement `POST /notifications/read-all`
- [ ] Notification types:
  - New follower
  - Kudos received
  - Comment on activity
  - Reply to comment
  - Crown achieved
  - Crown lost
  - PR achieved

### 4.2 Notifications UI

**Tasks:**
- [ ] Notification bell icon in header
- [ ] Unread count badge
- [ ] Notification dropdown/panel
- [ ] Mark as read on view
- [ ] Notification preferences page

### 4.3 Share Functionality

**Tasks:**
- [ ] Generate shareable activity links
- [ ] Generate shareable segment links
- [ ] Copy link button
- [ ] Open Graph meta tags for previews

---

## Stretch Goals

### Teams (Phase 5 Extension)

Teams allow users to form groups and share content within the team. This is a significant feature that could be Week 4+ or deferred to Phase 7.

**Database:**
```sql
-- Migration 014_teams.sql
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    avatar_url TEXT,
    visibility TEXT DEFAULT 'public',  -- 'public', 'private', 'invite_only'
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE team_members (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT DEFAULT 'member',  -- 'owner', 'admin', 'member'
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (team_id, user_id)
);

CREATE TABLE team_publications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    content_type TEXT NOT NULL,  -- 'activity', 'segment', 'route'
    content_id UUID NOT NULL,
    published_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_team_publications_team ON team_publications(team_id, published_at DESC);
```

**API:**
- `POST /teams` - Create team
- `GET /teams/{id}` - Get team details
- `GET /teams/{id}/members` - List members
- `POST /teams/{id}/members` - Invite/add member
- `DELETE /teams/{id}/members/{user_id}` - Remove member
- `POST /teams/{id}/publish` - Publish content to team
- `GET /teams/{id}/feed` - Team activity feed

**UI:**
- Team creation page
- Team home page with feed
- Team members list
- Publish to team option on activities/segments
- Team selector in navigation

### Synthetic Test Data

For testing with realistic data volumes:

**Options:**
1. **GPX repository** - Download public GPX files from sources like:
   - OpenStreetMap GPS traces
   - Wikiloc public routes
   - Strava's public segment data (careful with ToS)

2. **Synthetic generation** - Script to generate fake GPX with:
   - Realistic coordinate sequences
   - Varied activity types
   - Multiple users with demographics
   - Segment efforts with realistic times

3. **Data import tool** - CLI command to:
   - Load GPX files from a directory
   - Create users with varied demographics
   - Auto-create segments from activities
   - Generate effort history

**Implementation:**
```bash
# Proposed CLI commands
cargo run --bin generate-test-data -- --users 100 --activities 1000 --segments 50
cargo run --bin import-gpx -- --dir ./test-data/gpx --user test@example.com
```

### API Type Generation

For type-safe Rust-Node interface:

**Option 1: OpenAPI/Swagger**
- Generate OpenAPI spec from Rust handlers (utoipa crate)
- Generate TypeScript types from OpenAPI (openapi-typescript)
- Pros: Standard, good tooling
- Cons: Runtime overhead, manual sync

**Option 2: Protobuf**
- Define .proto files for API messages
- Generate Rust and TypeScript from protos
- Pros: Efficient, strongly typed
- Cons: Adds complexity, changes API format

**Recommendation:** Start with OpenAPI/Swagger since we already have REST API. Add utoipa to Rust handlers, generate TypeScript types.

### Enhanced Leaderboard Filters

Add more athlete-based filters to leaderboards:

- **Weight class** - Light (<60kg), Medium (60-80kg), Heavy (>80kg)
- **Equipment type** - Acoustic bike, eMTB, gravel, etc.
- **Power-to-weight** - If users add FTP

### GPS Data Quality

Track and use GPS data quality for better segment matching:

**Track metadata:**
```sql
ALTER TABLE tracks ADD COLUMN gps_sample_rate_hz FLOAT;  -- Points per second
ALTER TABLE tracks ADD COLUMN gps_accuracy_meters FLOAT;  -- Average HDOP
ALTER TABLE tracks ADD COLUMN point_count INTEGER;
```

**Segment matching tolerance:**
- High quality (1Hz+, <5m accuracy): 10m tolerance
- Medium quality (0.5Hz, 5-15m accuracy): 25m tolerance
- Low quality (<0.5Hz, >15m accuracy): 50m tolerance or reject

---

## Deliverables

### End of Phase 5 Checklist

**Core (Required):**
- [ ] Follow/unfollow users
- [ ] View followers/following lists
- [ ] Activity feed from followed users
- [ ] Give/remove kudos
- [ ] Post/delete comments
- [ ] Notification center
- [ ] Unread notification count
- [ ] Share links with OG previews

**Stretch (If Time Permits):**
- [ ] Teams creation and management
- [ ] Team feeds
- [ ] Publish to team functionality

### API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/users/{id}/follow` | Yes | Follow user |
| DELETE | `/users/{id}/follow` | Yes | Unfollow user |
| GET | `/users/{id}/followers` | Mixed | Get followers |
| GET | `/users/{id}/following` | Mixed | Get following |
| GET | `/feed` | Yes | Get activity feed |
| POST | `/activities/{id}/kudos` | Yes | Give kudos |
| DELETE | `/activities/{id}/kudos` | Yes | Remove kudos |
| GET | `/activities/{id}/kudos` | No | List kudos |
| POST | `/activities/{id}/comments` | Yes | Add comment |
| GET | `/activities/{id}/comments` | No | List comments |
| DELETE | `/comments/{id}` | Yes | Delete comment |
| GET | `/notifications` | Yes | Get notifications |
| POST | `/notifications/{id}/read` | Yes | Mark as read |
| POST | `/notifications/read-all` | Yes | Mark all read |

---

## Success Criteria

1. **Follow works:** Can follow/unfollow users
2. **Feed works:** Shows activities from followed users
3. **Kudos works:** Can give/remove kudos
4. **Comments work:** Can comment and reply
5. **Notifications work:** Receive and manage notifications
6. **Sharing works:** Can share with social previews
