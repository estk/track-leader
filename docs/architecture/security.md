# Security Architecture

Authentication and authorization patterns for Track Leader.

## Authentication

### PASETO Tokens (v4.local)

We use PASETO v4.local (symmetric encryption) instead of JWT for auth tokens.

**Token claims:**
- `sub` - User ID (UUID)
- `email` - User email
- `exp` - Expiration (7 days from issue)
- `iat` - Issued at timestamp

**Key configuration:**
- `PASETO_KEY` environment variable
- Production: 64-char hex string (32 bytes) - generate with `openssl rand -hex 32`
- Development: Any string (padded/truncated to 32 bytes)

**Why PASETO over JWT:**
- Encrypted payload (not just signed)
- No algorithm confusion attacks
- Simpler, safer defaults

### Key Files

| File | Purpose |
|------|---------|
| `crates/tracks/src/auth.rs` | Token creation/verification, password hashing, extractors |
| `src/lib/auth-context.tsx` | Frontend auth state (token storage, user info) |

## Authorization Patterns

### The Golden Rule

**All authorization happens in Rust handlers.** The frontend makes UX decisions (hide/show buttons) but these are cosmetic - the backend enforces all permissions.

### User Identity

**NEVER trust user identity from request parameters.** Always extract from the auth token:

```rust
// CORRECT: User ID from auth token
pub async fn create_thing(
    AuthUser(claims): AuthUser,  // User identity from token
    Json(req): Json<CreateRequest>,
) -> Result<Json<Thing>, AppError> {
    let user_id = claims.sub;  // Authenticated user
    // ...
}

// WRONG: User ID from query/body params
pub async fn create_thing(
    Query(params): Query<CreateQuery>,  // params.user_id is UNTRUSTED
) -> Result<Json<Thing>, AppError> {
    let user_id = params.user_id;  // SECURITY VULNERABILITY
    // ...
}
```

### Extractors

| Extractor | Use Case |
|-----------|----------|
| `AuthUser(claims)` | Required authentication - returns 401 if missing/invalid |
| `OptionalAuthUser(claims)` | Optional auth - `claims` is `Option<Claims>` |

### Team Membership

Team membership is checked via database queries, NOT stored in the token. This allows immediate revocation when users are removed from teams.

```rust
// Check if user is team member
let membership = db.get_team_membership(team_id, claims.sub).await?;

// Check if user has access via team sharing
let has_access = db.user_has_activity_team_access(claims.sub, activity_id).await?;
```

### Visibility Access Control

Activities and segments have visibility levels:

| Visibility | Access |
|------------|--------|
| `public` | Anyone |
| `private` | Owner only |
| `teams_only` | Owner OR members of shared teams |

Pattern for checking access:

```rust
let has_access = match item.visibility.as_str() {
    "public" => true,
    "private" => claims.as_ref().is_some_and(|c| c.sub == item.user_id),
    "teams_only" => {
        if let Some(ref c) = claims {
            c.sub == item.user_id || db.user_has_item_team_access(c.sub, item.id).await?
        } else {
            false
        }
    }
    _ => false,
};
```

### Role-Based Access (Teams)

Team roles: `Owner` > `Admin` > `Member`

| Action | Required Role |
|--------|---------------|
| Delete team | Owner |
| Update team settings | Owner, Admin |
| Manage members | Owner, Admin |
| Remove other admins | Owner only |
| Share content | Any member |

```rust
if !membership.role.can_manage_members() {
    return Err(AppError::Forbidden);
}
```

## Security Checklist for New Endpoints

When adding mutation endpoints:

- [ ] Use `AuthUser` extractor (not `OptionalAuthUser`) for privileged operations
- [ ] Extract user ID from `claims.sub`, never from request params
- [ ] Check ownership before modifying resources
- [ ] Check team membership before team operations
- [ ] Validate visibility rules for read operations
- [ ] Use `AppError::Forbidden` for authorization failures
- [ ] Use `AppError::Unauthorized` for authentication failures

## Common Vulnerabilities to Avoid

### 1. Trusting Client-Provided User ID

```rust
// BAD: User ID from query params
Query(params): Query<UploadQuery>  // params.user_id
// GOOD: User ID from auth token
AuthUser(claims): AuthUser  // claims.sub
```

### 2. Missing Auth on Mutation Endpoints

Every endpoint that creates, updates, or deletes data MUST use `AuthUser`.

### 3. Checking Auth in Frontend Only

Frontend visibility controls are UX only. Backend must independently verify permissions.

### 4. Forgetting Team Access Checks

When checking `teams_only` visibility, must check both:
1. Is user the owner?
2. Is user a member of a team the resource is shared with?
