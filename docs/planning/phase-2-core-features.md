# Phase 2: Core Features

**Duration:** Month 2 (4 weeks)
**Goal:** Build compelling activity management experience

> **Claude Agents:** Use `/frontend-design` for activity maps, elevation charts, and list UI. Use `/feature-dev` for API integrations.

---

## Objectives

1. Interactive activity map with full route visualization
2. Elevation profile and performance charts
3. Activity management (edit, delete, privacy)
4. User profile pages
5. Mobile-responsive design throughout

---

## Week 1: Activity Detail Page

### 1.1 Map Component

**Tasks:**
- [ ] Install MapLibre GL JS and React wrapper
- [ ] Create map container component
- [ ] Render activity route as line layer
- [ ] Add start/end markers with icons
- [ ] Implement fit-to-bounds on load
- [ ] Add zoom/pan controls
- [ ] Add fullscreen mode

**Map Features:**
```typescript
interface ActivityMapProps {
  coordinates: [number, number][];
  startPoint: [number, number];
  endPoint: [number, number];
  highlightRange?: [number, number]; // For segment preview
}
```

### 1.2 Elevation Profile

**Tasks:**
- [ ] Install Recharts
- [ ] Create elevation profile component
- [ ] Plot elevation vs distance
- [ ] Show grade percentages
- [ ] Highlight climbs/descents
- [ ] Sync hover with map position

**Component:**
```typescript
interface ElevationProfileProps {
  points: { distance: number; elevation: number }[];
  onHover?: (distance: number) => void;
}
```

### 1.3 Activity Stats Display

**Tasks:**
- [ ] Create stats grid component
- [ ] Display core metrics:
  - Distance
  - Duration (elapsed vs moving)
  - Elevation gain/loss
  - Average speed
  - Max speed
  - Pace
- [ ] Format units appropriately (km/mi toggle)
- [ ] Add metric tooltips with explanations

### 1.4 Splits Table

**Tasks:**
- [ ] Calculate kilometer/mile splits from trackpoints
- [ ] Display splits in sortable table
- [ ] Show pace, elevation change per split
- [ ] Highlight fastest/slowest splits
- [ ] Color-code by relative performance

---

## Week 2: Activity Management

### 2.1 Activity Edit

**Tasks:**
- [ ] Create edit modal/page
- [ ] Allow name/description edit
- [ ] Allow activity type change
- [ ] Allow gear selection (future)
- [ ] Implement backend `PATCH /activities/{id}`
- [ ] Optimistic UI updates

### 2.2 Activity Delete

**Tasks:**
- [ ] Add delete confirmation dialog
- [ ] Implement soft delete (mark deleted, don't remove)
- [ ] Backend `DELETE /activities/{id}`
- [ ] Remove from object store
- [ ] Update UI immediately

### 2.3 Privacy Controls

**Tasks:**
- [ ] Add visibility column to activities
- [ ] Options: public, followers, private
- [ ] Privacy selector in upload flow
- [ ] Privacy editor in activity detail
- [ ] Enforce privacy in list queries

**Schema:**
```sql
ALTER TABLE activities ADD COLUMN visibility TEXT DEFAULT 'public';
```

### 2.4 Activity Actions

**Tasks:**
- [ ] Download original GPX button
- [ ] Share activity (generate share link)
- [ ] Export to different formats (GPX, TCX)
- [ ] Copy link to clipboard
- [ ] Social share buttons (Twitter, Facebook)

---

## Week 3: Activity List & Search

### 3.1 Activity List Page

**Tasks:**
- [ ] Create activity list component
- [ ] Show activity cards with:
  - Map thumbnail (static image)
  - Name, date, type
  - Key stats (distance, duration, elevation)
- [ ] Implement infinite scroll
- [ ] Add loading skeletons

### 3.2 Filtering

**Tasks:**
- [ ] Filter by activity type
- [ ] Filter by date range
- [ ] Filter by distance range
- [ ] Filter by duration range
- [ ] Combine filters with AND logic
- [ ] Persist filters in URL

### 3.3 Sorting

**Tasks:**
- [ ] Sort by date (default, desc)
- [ ] Sort by distance
- [ ] Sort by duration
- [ ] Sort by elevation
- [ ] Toggle ascending/descending

### 3.4 Search

**Tasks:**
- [ ] Full-text search on activity name
- [ ] Search by location (future - needs geocoding)
- [ ] Debounced search input
- [ ] Highlight matching text

**Backend:**
```rust
#[derive(Deserialize)]
pub struct ActivityQuery {
    pub activity_type: Option<ActivityType>,
    pub from_date: Option<OffsetDateTime>,
    pub to_date: Option<OffsetDateTime>,
    pub min_distance: Option<f64>,
    pub max_distance: Option<f64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
```

---

## Week 4: User Profiles & Mobile

### 4.1 User Profile Page

**Tasks:**
- [ ] Create `/profile/[username]` route
- [ ] Display user info (name, avatar, bio)
- [ ] Show activity statistics:
  - Total activities
  - Total distance
  - Total elevation
  - Longest activity
  - Fastest activity
- [ ] List recent activities
- [ ] Edit profile button (if own profile)

### 4.2 Profile Edit

**Tasks:**
- [ ] Create profile edit modal
- [ ] Allow name, bio, location edit
- [ ] Avatar upload
- [ ] Backend `PATCH /users/{id}`
- [ ] Image processing/resize

### 4.3 Settings Page

**Tasks:**
- [ ] Create `/settings` route
- [ ] Account settings section
- [ ] Privacy settings section
- [ ] Unit preferences (metric/imperial)
- [ ] Email preferences
- [ ] Delete account option

### 4.4 Mobile Responsiveness

**Tasks:**
- [ ] Audit all pages for mobile
- [ ] Implement mobile navigation (bottom tabs or hamburger)
- [ ] Optimize map for touch
- [ ] Touch-friendly filter controls
- [ ] Swipe gestures where appropriate
- [ ] Test on various screen sizes

---

## Deliverables

### End of Phase 2 Checklist

- [ ] Activity detail page with interactive map
- [ ] Elevation profile chart
- [ ] Splits table
- [ ] Activity edit/delete working
- [ ] Privacy controls implemented
- [ ] Activity list with filters/search/sort
- [ ] Infinite scroll pagination
- [ ] User profile pages
- [ ] Profile editing
- [ ] Settings page
- [ ] Mobile responsive on all pages

### Components Created

| Component | Purpose |
|-----------|---------|
| `ActivityMap` | Interactive route map |
| `ElevationProfile` | Elevation chart |
| `StatsGrid` | Activity statistics |
| `SplitsTable` | Kilometer/mile splits |
| `ActivityCard` | List item component |
| `FilterPanel` | Activity filters |
| `ProfileHeader` | User profile header |
| `ProfileStats` | User statistics |
| `SettingsForm` | Settings sections |

---

## API Endpoints After Phase 2

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| PATCH | `/activities/{id}` | Yes | Update activity |
| DELETE | `/activities/{id}` | Yes | Delete activity |
| GET | `/activities` | Yes | List with filters |
| GET | `/users/{id}` | Mixed | Get user profile |
| PATCH | `/users/{id}` | Yes | Update profile |
| POST | `/users/{id}/avatar` | Yes | Upload avatar |
| GET | `/users/{id}/stats` | Mixed | Get user stats |

---

## Database Changes

```sql
-- Add visibility to activities
ALTER TABLE activities ADD COLUMN visibility TEXT DEFAULT 'public';
ALTER TABLE activities ADD COLUMN description TEXT;
ALTER TABLE activities ADD COLUMN deleted_at TIMESTAMPTZ;

-- Add profile fields to users
ALTER TABLE users ADD COLUMN bio TEXT;
ALTER TABLE users ADD COLUMN location TEXT;
ALTER TABLE users ADD COLUMN preferences JSONB DEFAULT '{}';

-- Add index for activity queries
CREATE INDEX idx_activities_visibility ON activities(visibility);
CREATE INDEX idx_activities_deleted ON activities(deleted_at) WHERE deleted_at IS NULL;
```

---

## Static Map Generation

For activity list thumbnails, generate static map images:

**Option A: Server-side rendering**
- Use MapLibre Native or similar
- Generate on activity upload
- Store in object store
- Regenerate on edit

**Option B: Third-party service**
- Use Mapbox Static API
- Generate URL with encoded polyline
- Cache-friendly URLs

**Recommended:** Start with Option B for simplicity, migrate to A if costs become issue.

---

## Performance Considerations

### Map Performance
- Simplify routes for display (Douglas-Peucker)
- Progressive loading for long routes
- Tile-based rendering

### List Performance
- Virtual scrolling for long lists
- Lazy load map thumbnails
- Skeleton loading states

### Backend Performance
- Index activity queries properly
- Cache user stats
- Pagination required

---

## Success Criteria

1. **Map works:** Interactive, responsive, fits route
2. **Elevation works:** Chart syncs with map
3. **Edit works:** Changes persist correctly
4. **Delete works:** Activity removed from list
5. **List works:** Filters, search, sort functional
6. **Profile works:** Stats display correctly
7. **Mobile works:** All features usable on phone
