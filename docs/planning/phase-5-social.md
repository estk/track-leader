# Phase 5: Social Features

**Duration:** Month 5 (3-4 weeks)
**Goal:** Build community engagement and social interactions

> **Claude Agents:** Use `/frontend-design` for feed, kudos, and comment UI. Use `/feature-dev` for notification system.

---

## Objectives

1. Follow system for users
2. Activity feed from followed users
3. Kudos and comments on activities
4. Share functionality
5. Notifications system

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
- [ ] Follow suggestions (based on activity overlap)

### 1.4 Privacy Considerations

**Tasks:**
- [ ] Respect user privacy settings
- [ ] Option to require follow approval
- [ ] Block user functionality
- [ ] Hide from search option

---

## Week 2: Activity Feed

### 2.1 Feed Algorithm

**Tasks:**
- [ ] Design feed query
- [ ] Include followed users' activities
- [ ] Include segment efforts from followed users
- [ ] Include crowned segments
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

### 2.3 Feed Personalization (Future)

**Placeholder for future:**
- Relevance scoring
- Popular activities boost
- PR/achievement highlighting
- "Suggested for you" section

---

## Week 3: Kudos & Comments

### 3.1 Kudos System

**Database:**
```sql
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
- [ ] Mention parsing (@username)

### 3.4 Comments UI

**Tasks:**
- [ ] Comment section on activity detail
- [ ] Comment input with submit button
- [ ] Display comments threaded
- [ ] Reply button
- [ ] Delete own comments
- [ ] @mention autocomplete

---

## Week 4: Sharing & Notifications

### 4.1 Share Functionality

**Tasks:**
- [ ] Generate shareable activity links
- [ ] Generate shareable segment links
- [ ] Copy link button
- [ ] Social share buttons:
  - Twitter/X
  - Facebook
  - LinkedIn
  - Email
- [ ] Open Graph meta tags for previews
- [ ] Activity embed code (iframe)

### 4.2 Notifications System

**Tasks:**
- [ ] Implement `GET /notifications`
- [ ] Implement `POST /notifications/{id}/read`
- [ ] Implement `POST /notifications/read-all`
- [ ] Notification types:
  - New follower
  - Kudos received
  - Comment on activity
  - Reply to comment
  - Mention
  - Crown achieved
  - Crown lost
  - PR achieved

### 4.3 Notifications UI

**Tasks:**
- [ ] Notification bell icon in header
- [ ] Unread count badge
- [ ] Notification dropdown/panel
- [ ] Mark as read on view
- [ ] Notification preferences page
- [ ] Email notification option (future)

### 4.4 Push Notifications (PWA)

**Tasks:**
- [ ] Register service worker
- [ ] Request notification permission
- [ ] Store push subscription
- [ ] Backend push service
- [ ] Notification categories

---

## Deliverables

### End of Phase 5 Checklist

- [ ] Follow/unfollow users
- [ ] View followers/following lists
- [ ] Activity feed from followed users
- [ ] Give/remove kudos
- [ ] Post/delete comments
- [ ] Threaded comment replies
- [ ] @mentions working
- [ ] Share activities/segments
- [ ] Notification center
- [ ] Notification preferences
- [ ] Unread notification count

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

## Social Graph Considerations

### Spam Prevention

- Rate limit follows (50/day)
- Rate limit comments (100/day)
- Report button on comments
- Block functionality
- Auto-flag suspicious activity

### Content Moderation

- Comment content filtering (profanity, spam)
- Report comment flow
- Admin review queue (future)
- Shadow banning capability

### Privacy

- Respect activity visibility
- Don't leak private activity in notifications
- Allow disabling notifications
- GDPR: export/delete social data

---

## Feed Performance

### Caching Strategy

- Cache feed per user (5 minute TTL)
- Invalidate on new activity from followed
- Paginate with cursor, not offset
- Pre-compute feed for active users

### Query Optimization

- Index on (user_id, submitted_at) for feed
- Denormalize counts (kudos, comments)
- Batch fetch related data
- Limit feed to 7 days for inactive users

---

## Success Criteria

1. **Follow works:** Can follow/unfollow users
2. **Feed works:** Shows activities from followed users
3. **Kudos works:** Can give/remove kudos
4. **Comments work:** Can comment and reply
5. **Sharing works:** Can share to social platforms
6. **Notifications work:** Receive and manage notifications
