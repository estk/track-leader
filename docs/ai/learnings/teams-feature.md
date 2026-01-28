# Teams Feature Implementation

Reference for the teams feature implementation, added to enable group-based access control for activities and segments.

## Purpose

Teams allow users to share activities and segments with specific groups rather than making them fully public or private. This is useful for:
- Private property access (e.g., ranch trails)
- Club/team training data
- Exclusive segment access

## Key Concepts

### Visibility Model

Three visibility levels for activities and segments:

| Visibility | Who Can See |
|------------|-------------|
| `public` | Everyone |
| `private` | Owner only |
| `teams_only` | Owner + members of shared teams |

### Team Roles

| Role | Can Manage Members | Can Modify Team | Can Delete Team |
|------|-------------------|-----------------|-----------------|
| `owner` | Yes | Yes | Yes |
| `admin` | Yes | Yes | No |
| `member` | No | No | No |

Role methods in `TeamRole`:
- `can_manage_members()` - owner or admin
- `can_modify_team()` - owner or admin
- `can_delete_team()` - owner only

### Join Policies

| Policy | Behavior |
|--------|----------|
| `invitation` | Must be invited by admin |
| `request` | User requests, admin approves |
| `open` | Anyone can join directly |

## Implementation Details

### Database Schema

Migrations:
- `006_teams.sql` - Core tables (teams, memberships, invitations, requests, sharing)
- `007_teams_indexes.sql` - Performance indexes
- `008_teams_only_visibility.sql` - Documents visibility column extension

### Backend (Rust)

Key files:
- `crates/tracks/src/models.rs` - Team structs and enums
- `crates/tracks/src/database.rs` - Team queries (~500 lines)
- `crates/tracks/src/handlers.rs` - Team handlers (~400 lines)
- `crates/tracks/src/auth.rs` - Added `OptionalAuthUser` extractor

Access control pattern in handlers:
```rust
pub async fn get_activity(
    Extension(db): Extension<Database>,
    OptionalAuthUser(claims): OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Activity>, AppError> {
    let activity = db.get_activity(id).await?.ok_or(AppError::NotFound)?;

    let has_access = match activity.visibility.as_str() {
        "public" => true,
        "private" => claims.as_ref().map_or(false, |c| c.sub == activity.user_id),
        "teams_only" => {
            if let Some(ref c) = claims {
                c.sub == activity.user_id ||
                    db.user_has_activity_team_access(c.sub, id).await?
            } else {
                false
            }
        }
        _ => false,
    };

    if has_access {
        Ok(Json(activity))
    } else {
        Err(AppError::NotFound) // 404, not 403!
    }
}
```

**Security Rule:** Always return 404 (not 403) for unauthorized access to avoid leaking resource existence.

### Frontend (React/Next.js)

Key files:
- `src/lib/api.ts` - Team API methods and types
- `src/components/teams/team-card.tsx` - Team card display
- `src/components/teams/team-selector.tsx` - Multi-select for team sharing
- `src/components/teams/role-badge.tsx` - Role badge (amber/blue/gray)
- `src/app/teams/page.tsx` - Teams list (My Teams + Discover tabs)
- `src/app/teams/new/page.tsx` - Create team form
- `src/app/teams/[id]/page.tsx` - Team detail with tabs
- `src/app/teams/[id]/settings/page.tsx` - Team settings (admin only)
- `src/app/teams/[id]/invite/page.tsx` - Invitation management
- `src/app/activities/upload/page.tsx` - Updated with team selection

Team selection in upload:
1. User selects "Teams Only" visibility
2. `TeamSelector` component appears
3. User selects one or more teams
4. `teamIds` passed to upload API
5. Backend creates `activity_teams` junction records

## Gotchas

### OptionalAuthUser Extractor

Created `OptionalAuthUser` for endpoints that need optional auth:
- Returns `Option<Claims>` instead of failing
- Used for public endpoints with conditional access (get_activity, get_segment, etc.)
- Returns `Infallible` as rejection type (never fails)

### ActivityVisibility Type

Frontend uses `ActivityVisibility` and `SegmentVisibility` types:
```typescript
export type ActivityVisibility = 'public' | 'private' | 'teams_only';
```

When adding teams_only support to existing pages, update state types from `"public" | "private"` to `ActivityVisibility`.

### Denormalized Counts

Teams table has denormalized counts:
- `member_count` - Updated on add/remove member
- `activity_count` - Updated on share/unshare
- `segment_count` - Updated on share/unshare

These are updated atomically in the respective database methods.

## Testing Checklist

- [ ] Create team with each visibility/join_policy combination
- [ ] Invite user via email, accept invitation
- [ ] Request to join, approve/reject
- [ ] Upload activity as teams_only, verify non-member gets 404
- [ ] Create segment as teams_only, verify track data protected
- [ ] Leave team, verify loss of access
- [ ] Delete team, verify cascade deletes memberships
