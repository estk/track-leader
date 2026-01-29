# Handlers Refactoring Plan - Track Leader Backend

## Executive Summary

This plan details the refactoring of `/crates/tracks/src/handlers.rs` from a single 3,926-line file with 93 handlers into a well-organized module structure with generic abstractions that eliminate code duplication. The refactoring will reduce the handler count through consolidation and create reusable patterns for toggle operations, notifications, and permission checks.

---

## 1. Current State Analysis

### Handler Counts by Domain

| Domain | Handler Count | Lines (approx) | Notes |
|--------|--------------|----------------|-------|
| Users | 3 | ~80 | `new_user`, `all_users`, `get_user_profile` |
| Activities | 7 | ~350 | CRUD + track/segments/download |
| Activity Types | 4 | ~150 | CRUD + resolve |
| Segments | 21 | ~800 | CRUD, star, track, nearby, leaderboard, preview, reprocess |
| Demographics | 2 | ~60 | get/update |
| Achievements | 3 | ~100 | user/my/segment achievements |
| Global Leaderboards | 3 | ~90 | crowns, distance, countries |
| Social/Follow | 5 | ~180 | follow/unfollow/status/followers/following |
| Notifications | 3 | ~100 | get/mark read/mark all read |
| Feed | 1 | ~30 | get_feed |
| Kudos | 4 | ~150 | give/remove/status/givers |
| Comments | 3 | ~100 | add/get/delete |
| Stats | 1 | ~20 | get_stats |
| Teams | 24 | ~900 | CRUD, members, join, invitations, sharing |

### Identified Code Duplication

1. **Toggle Pattern** (~200 lines duplicated across 3 features)
   - Segment starring: `star_segment`, `unstar_segment`, `is_segment_starred`
   - Following: `follow_user`, `unfollow_user`, `get_follow_status`
   - Kudos: `give_kudos`, `remove_kudos`, `get_kudos_status`

2. **Visibility Access Control** (~150 lines, repeated 8 times)
   - Same pattern in `get_activity`, `get_activity_track`, `download_gpx_file`, `get_activity_segments`, `get_segment`, `get_segment_track`, etc.

3. **Team Permission Checks** (~150 lines repeated)
   - `get_team_membership().ok_or(NotFound)?` + `role.can_manage_members()` pattern

4. **Notification Creation** (~50 lines across 8+ handlers)
   - `db.create_notification(user_id, type, actor, target_type, target_id, message)`

5. **Pagination Query Structs** (~30 lines each, 5 similar structs)
   - `FollowListQuery`, `NotificationsQuery`, `FeedQuery`, `DiscoverTeamsQuery`, `TeamContentQuery`

### Inline Types Count
35 request/response types defined directly in handlers.rs

---

## 2. Target Architecture

### Module Structure

```
crates/tracks/src/
├── handlers/                    # New handlers module directory
│   ├── mod.rs                   # Re-exports and trait definitions
│   ├── toggle.rs                # Generic toggle trait + implementations
│   ├── access.rs                # Visibility access control helpers
│   ├── pagination.rs            # Reusable pagination types
│   ├── notifications.rs         # Notification creation helper
│   ├── users.rs                 # User handlers (3 handlers)
│   ├── activities.rs            # Activity handlers (7 handlers)
│   ├── activity_types.rs        # Activity type handlers (4 handlers)
│   ├── segments/
│   │   ├── mod.rs               # Segment handlers re-export
│   │   ├── crud.rs              # create, get, list (3 handlers)
│   │   ├── track.rs             # track, preview, reprocess (3 handlers)
│   │   ├── star.rs              # star/unstar using toggle (3 handlers -> uses toggle)
│   │   ├── leaderboard.rs       # leaderboard, filtered, position (3 handlers)
│   │   └── nearby.rs            # nearby segments (1 handler)
│   ├── demographics.rs          # Demographics handlers (2 handlers)
│   ├── achievements.rs          # Achievement handlers (3 handlers)
│   ├── leaderboards.rs          # Global leaderboard handlers (3 handlers)
│   ├── social/
│   │   ├── mod.rs               # Social handlers re-export
│   │   ├── follow.rs            # Follow handlers using toggle (5 handlers -> uses toggle)
│   │   ├── kudos.rs             # Kudos handlers using toggle (4 handlers -> uses toggle)
│   │   ├── comments.rs          # Comment handlers (3 handlers)
│   │   └── feed.rs              # Feed handler (1 handler)
│   ├── notifications.rs         # Notification handlers (3 handlers)
│   ├── stats.rs                 # Stats handler (1 handler)
│   └── teams/
│       ├── mod.rs               # Team handlers re-export
│       ├── crud.rs              # Team CRUD (5 handlers)
│       ├── members.rs           # Membership management (5 handlers)
│       ├── join.rs              # Join/leave (4 handlers)
│       ├── invitations.rs       # Invitation management (5 handlers)
│       └── sharing.rs           # Activity/segment sharing (6 handlers)
├── types/                       # Extracted types module
│   ├── mod.rs                   # Re-exports
│   ├── requests.rs              # Request types
│   ├── responses.rs             # Response types
│   └── queries.rs               # Query parameter types
├── lib.rs                       # Router setup (updated imports)
├── models.rs                    # Domain models (unchanged)
├── database.rs                  # Database layer (unchanged)
├── errors.rs                    # Error types (unchanged)
└── ... (other existing files)
```

### Estimated Line Counts (Post-Refactor)

| Module | Lines | Handlers |
|--------|-------|----------|
| `handlers/mod.rs` | ~50 | 0 |
| `handlers/toggle.rs` | ~150 | 0 (trait + impls) |
| `handlers/access.rs` | ~80 | 0 (helpers) |
| `handlers/pagination.rs` | ~40 | 0 (types) |
| `handlers/users.rs` | ~80 | 3 |
| `handlers/activities.rs` | ~300 | 7 |
| `handlers/activity_types.rs` | ~150 | 4 |
| `handlers/segments/` | ~600 | 12 |
| `handlers/demographics.rs` | ~60 | 2 |
| `handlers/achievements.rs` | ~100 | 3 |
| `handlers/leaderboards.rs` | ~90 | 3 |
| `handlers/social/` | ~350 | 13 |
| `handlers/notifications.rs` | ~100 | 3 |
| `handlers/stats.rs` | ~30 | 1 |
| `handlers/teams/` | ~700 | 25 |
| `types/` | ~200 | 0 |
| **Total** | ~3,080 | 76 |

**Net Reduction**: ~850 lines (22% reduction) through generic abstractions

---

## 3. Generic Abstractions

### 3.1 Toggle Trait

**Location**: `crates/tracks/src/handlers/toggle.rs`

**Purpose**: Eliminate duplicated toggle operation pattern for star/follow/kudos

```rust
// Conceptual interface (not actual implementation)
#[async_trait]
pub trait ToggleOperation {
    type TargetId;
    type StatusResponse;

    /// Check if the toggle is currently enabled
    async fn is_enabled(db: &Database, user_id: Uuid, target_id: Self::TargetId) -> Result<bool, AppError>;

    /// Enable the toggle (create relationship)
    async fn enable(db: &Database, user_id: Uuid, target_id: Self::TargetId) -> Result<bool, AppError>;

    /// Disable the toggle (remove relationship)
    async fn disable(db: &Database, user_id: Uuid, target_id: Self::TargetId) -> Result<bool, AppError>;

    /// Optional: create notification when enabled
    fn notification_config() -> Option<NotificationConfig>;

    /// Optional: validation before enabling (e.g., "can't follow yourself")
    async fn validate_enable(db: &Database, user_id: Uuid, target_id: Self::TargetId) -> Result<(), AppError> {
        Ok(()) // Default: no validation
    }
}

// Implementations for:
pub struct SegmentStar;      // star_segment, unstar_segment, is_segment_starred
pub struct UserFollow;       // follow_user, unfollow_user, get_follow_status
pub struct ActivityKudos;    // give_kudos, remove_kudos, get_kudos_status
```

**Handler Generation**: Generic handler functions that work with any `ToggleOperation`:

```rust
pub async fn toggle_enable<T: ToggleOperation>(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(target_id): Path<T::TargetId>,
) -> Result<impl IntoResponse, AppError> {
    T::validate_enable(&db, claims.sub, target_id).await?;

    let was_already_enabled = T::is_enabled(&db, claims.sub, target_id).await?;
    if was_already_enabled {
        return Ok(StatusCode::OK); // Idempotent
    }

    T::enable(&db, claims.sub, target_id).await?;

    if let Some(config) = T::notification_config() {
        // Create notification...
    }

    Ok(StatusCode::CREATED)
}
```

### 3.2 Visibility Access Control

**Location**: `crates/tracks/src/handlers/access.rs`

**Purpose**: Eliminate repeated visibility check pattern

```rust
pub enum ResourceType {
    Activity { owner_id: Uuid },
    Segment { creator_id: Uuid },
}

pub async fn check_visibility_access(
    db: &Database,
    claims: Option<&Claims>,
    visibility: &str,
    resource: ResourceType,
    resource_id: Uuid,
) -> Result<bool, AppError> {
    match visibility {
        "public" => Ok(true),
        "private" => {
            let owner_id = match resource {
                ResourceType::Activity { owner_id } => owner_id,
                ResourceType::Segment { creator_id } => creator_id,
            };
            Ok(claims.map_or(false, |c| c.sub == owner_id))
        }
        "teams_only" => {
            if let Some(c) = claims {
                let owner_id = match resource {
                    ResourceType::Activity { owner_id } => owner_id,
                    ResourceType::Segment { creator_id } => creator_id,
                };
                if c.sub == owner_id {
                    return Ok(true);
                }
                match resource {
                    ResourceType::Activity { .. } => {
                        db.user_has_activity_team_access(c.sub, resource_id).await
                    }
                    ResourceType::Segment { .. } => {
                        db.user_has_segment_team_access(c.sub, resource_id).await
                    }
                }
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}
```

### 3.3 Team Permission Helper

**Location**: `crates/tracks/src/handlers/teams/mod.rs`

```rust
pub async fn require_team_permission(
    db: &Database,
    team_id: Uuid,
    user_id: Uuid,
    permission: TeamPermission,
) -> Result<TeamMembership, AppError> {
    let membership = db
        .get_team_membership(team_id, user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let has_permission = match permission {
        TeamPermission::ManageMembers => membership.role.can_manage_members(),
        TeamPermission::ModifyTeam => membership.role.can_modify_team(),
        TeamPermission::DeleteTeam => membership.role.can_delete_team(),
        TeamPermission::ViewContent => true, // Any member
    };

    if !has_permission {
        return Err(AppError::Forbidden);
    }

    Ok(membership)
}

pub enum TeamPermission {
    ManageMembers,
    ModifyTeam,
    DeleteTeam,
    ViewContent,
}
```

### 3.4 Pagination Types

**Location**: `crates/tracks/src/handlers/pagination.rs`

```rust
/// Standard pagination query parameters
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// Paginated response wrapper
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}
```

### 3.5 Notification Helper

**Location**: `crates/tracks/src/handlers/notifications.rs` (helper section)

```rust
pub struct NotificationBuilder {
    recipient_id: Uuid,
    notification_type: &'static str,
    actor_id: Option<Uuid>,
    target_type: Option<&'static str>,
    target_id: Option<Uuid>,
    message: Option<String>,
}

impl NotificationBuilder {
    pub fn new(recipient_id: Uuid, notification_type: &'static str) -> Self { ... }
    pub fn actor(mut self, actor_id: Uuid) -> Self { ... }
    pub fn target(mut self, target_type: &'static str, target_id: Uuid) -> Self { ... }
    pub fn message(mut self, message: impl Into<String>) -> Self { ... }

    pub async fn send(self, db: &Database) -> Result<(), AppError> {
        db.create_notification(
            self.recipient_id,
            self.notification_type,
            self.actor_id,
            self.target_type,
            self.target_id,
            self.message.as_deref(),
        ).await
    }
}

// Usage:
NotificationBuilder::new(activity.user_id, "kudos")
    .actor(claims.sub)
    .target("activity", activity_id)
    .send(&db)
    .await?;
```

---

## 4. API Surface Consolidation Opportunities

### Endpoints That Could Be Merged

1. **Achievements**: `get_user_achievements` and `get_my_achievements` could use a single endpoint with `/users/{id}/achievements` where `id` can be "me" or a UUID

2. **Demographics**: Similarly, `/users/{id}/demographics` with "me" support

3. **Starred Segments**: `get_starred_segments` and `get_starred_segment_efforts` could be a single endpoint with `?include_efforts=true`

### Endpoints That Should Remain Separate

1. **Toggle operations** (star/follow/kudos) - different HTTP methods on same path is idiomatic REST
2. **Team operations** - separation by concern is clearer
3. **Activity vs Segment sharing** - different resource types, different validation

---

## 5. Implementation Checklist

### Phase 1: Setup and Infrastructure ✅

- [x] **1.1** Create `handlers/` directory structure
- [x] **1.2** Create `handlers/mod.rs` with placeholder module declarations
- [x] **1.3** Create `handlers/pagination.rs` with `PaginationQuery` and `PaginatedResponse`
- [x] **1.4** Create `handlers/access.rs` with visibility helpers
- [x] **1.5** Create `types/` directory with `mod.rs`, `requests.rs`, `responses.rs`, `queries.rs`
- [ ] **1.6** Move 35 inline request/response types to `types/` modules (types migrated with handlers)

### Phase 2: Generic Abstractions ✅

- [x] **2.1** Create `handlers/toggle.rs` with `ToggleOperation` trait
- [x] **2.2** Implement `SegmentStar` toggle
- [x] **2.3** Implement `UserFollow` toggle
- [x] **2.4** Implement `ActivityKudos` toggle
- [x] **2.5** Create generic toggle handler functions
- [ ] **2.6** Create team permission helper in `handlers/teams/mod.rs` (pending teams refactor)
- [x] **2.7** Create `NotificationBuilder` helper (`handlers/notify.rs`)

### Phase 3: Simple Handler Modules (Parallel - 3 agents)

**Agent A: Core Resources ✅**
- [x] **3.1a** Move user handlers to `handlers/users.rs`
- [x] **3.2a** Move activity type handlers to `handlers/activity_types.rs`
- [x] **3.3a** Move stats handler to `handlers/stats.rs`
- [x] **3.4a** Move demographics handlers to `handlers/demographics.rs`
- [x] **3.5a** Move achievement handlers to `handlers/achievements.rs`
- [x] **3.6a** Move global leaderboard handlers to `handlers/leaderboards.rs`

**Agent B: Activities and Segments**
- [ ] **3.1b** Create `handlers/activities.rs` with activity CRUD handlers
- [ ] **3.2b** Create `handlers/segments/mod.rs` structure
- [ ] **3.3b** Move segment CRUD to `handlers/segments/crud.rs`
- [ ] **3.4b** Move segment track/preview/reprocess to `handlers/segments/track.rs`
- [ ] **3.5b** Move segment leaderboard handlers to `handlers/segments/leaderboard.rs`
- [ ] **3.6b** Move nearby segments to `handlers/segments/nearby.rs`
- [ ] **3.7b** Convert segment star handlers to use toggle in `handlers/segments/star.rs`

**Agent C: Social Features**
- [ ] **3.1c** Create `handlers/social/mod.rs` structure
- [ ] **3.2c** Convert follow handlers to use toggle in `handlers/social/follow.rs`
- [ ] **3.3c** Convert kudos handlers to use toggle in `handlers/social/kudos.rs`
- [ ] **3.4c** Move comment handlers to `handlers/social/comments.rs`
- [ ] **3.5c** Move feed handler to `handlers/social/feed.rs`
- [ ] **3.6c** Move notification handlers to `handlers/notifications.rs`

### Phase 4: Team Handlers

- [ ] **4.1** Create `handlers/teams/mod.rs` with permission helper
- [ ] **4.2** Move team CRUD to `handlers/teams/crud.rs`
- [ ] **4.3** Move membership handlers to `handlers/teams/members.rs`
- [ ] **4.4** Move join/leave handlers to `handlers/teams/join.rs`
- [ ] **4.5** Move invitation handlers to `handlers/teams/invitations.rs`
- [ ] **4.6** Move sharing handlers to `handlers/teams/sharing.rs`

### Phase 5: Integration (Sequential)

- [ ] **5.1** Update `lib.rs` router imports
- [ ] **5.2** Update `lib.rs` OpenAPI path declarations
- [ ] **5.3** Delete old `handlers.rs` file
- [ ] **5.4** Run `cargo +nightly fmt`
- [ ] **5.5** Run `cargo clippy --workspace`
- [ ] **5.6** Run `cargo nextest run`
- [ ] **5.7** Run E2E tests: `cd e2e && npm test`

### Phase 6: Documentation

- [ ] **6.1** Update `docs/ai/index.md` with new structure
- [ ] **6.2** Add brief comments to each handler module explaining its purpose

---

## 6. Agent Parallelization Strategy

### Recommended Agent Assignment

```
Main Agent (Orchestrator)
├── Infrastructure Agent (Phase 1)
│   └── Creates directory structure, pagination types, access helpers
├── Abstractions Agent (Phase 2)
│   └── Creates toggle trait, implementations, notification builder
├── Core Resources Agent (Phase 3A)
│   └── Moves simple handlers: users, activity_types, stats, demographics, achievements, leaderboards
├── Activities/Segments Agent (Phase 3B)
│   └── Restructures activity and segment handlers
├── Social Agent (Phase 3C)
│   └── Restructures social features: follow, kudos, comments, feed, notifications
└── Teams Agent (Phase 4)
    └── Restructures all team-related handlers
```

### Dependency Graph

```
Phase 1 (Infrastructure)
    │
    v
Phase 2 (Abstractions)
    │
    ├────────────────────┬────────────────────┬────────────────────┐
    v                    v                    v                    v
Phase 3A             Phase 3B             Phase 3C             Phase 4
(Core)               (Activities)         (Social)             (Teams)
    │                    │                    │                    │
    └────────────────────┴────────────────────┴────────────────────┘
                                   │
                                   v
                             Phase 5 (Integration)
                                   │
                                   v
                             Phase 6 (Documentation)
```

**Phases 3A, 3B, 3C, and 4 can run in parallel** once Phases 1 and 2 are complete.

---

## 7. Testing Strategy

### During Refactoring

1. **After each module move**: Run `cargo check` to verify compilation
2. **After completing each phase**: Run `cargo nextest run` for unit/integration tests
3. **After Phase 5**: Run full E2E test suite

### Maintaining Backwards Compatibility

- All public API endpoints remain unchanged (same paths, methods, request/response shapes)
- Only internal code organization changes
- OpenAPI spec should produce identical output

### Test Commands

```bash
# Quick compile check
cargo check --workspace

# Full test suite (filtered output for token efficiency)
cargo nextest run --package tracks 2>&1 | grep -v -E "^\s*(Compiling|Fresh|Blocking)"

# E2E tests
cd e2e && npm test

# Verify OpenAPI spec unchanged
diff <(curl -s localhost:8000/api-docs/openapi.json | jq -S) <(curl -s localhost:8000/api-docs/openapi.json | jq -S)
```

---

## 8. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking API compatibility | OpenAPI spec diff check, E2E tests |
| Missing import during move | `cargo check` after each module |
| Circular dependencies | Careful module organization, types in separate module |
| Toggle abstraction too complex | Start simple, add complexity only if needed |
| Merge conflicts during parallel work | Clear module boundaries, coordinate on shared files |

---

## 9. Success Metrics

1. **Line count reduction**: Target 20-25% reduction through deduplication
2. **Compilation time**: Should remain similar or improve (more parallel compilation)
3. **Test pass rate**: 100% (no behavioral changes)
4. **Max file size**: No single file > 500 lines
5. **Cyclomatic complexity**: Reduced through extracted helpers

---

## Critical Files

- `crates/tracks/src/handlers.rs` - Source file to refactor (3,926 lines)
- `crates/tracks/src/lib.rs` - Router setup requiring import updates
- `crates/tracks/src/models.rs` - Domain types that handlers depend on
- `crates/tracks/src/database.rs` - Database layer the handlers call into
- `crates/tracks/src/errors.rs` - Error types used by handlers
