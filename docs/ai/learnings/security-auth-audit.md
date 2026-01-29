# Security Audit: Authentication & Authorization

Date: 2025-01-28

## Summary

Migrated from JWT to PASETO and discovered a critical auth vulnerability in the activity upload endpoint.

## Changes Made

### 1. PASETO Migration

Replaced `jsonwebtoken` crate with `pasetors` for token-based authentication.

**Why PASETO:**
- v4.local uses symmetric encryption (not just signing)
- No algorithm confusion attacks possible
- Encrypted payload protects claims from exposure

**Token claims:**
- `sub` (user ID), `email`, `exp`, `iat`
- Team membership intentionally NOT in token (checked via DB for immediate revocation)

**Key handling:**
- `PASETO_KEY` env var
- 64-char hex for production, any string for dev (padded to 32 bytes)

### 2. Security Vulnerability Fixed

**Issue:** `new_activity` handler accepted `user_id` as query parameter without authentication.

```rust
// BEFORE (vulnerable)
pub struct UploadQuery {
    pub user_id: Uuid,  // Attacker-controlled!
    // ...
}

pub async fn new_activity(
    Query(params): Query<UploadQuery>,  // No auth check
) {
    // Used params.user_id to create activity
}
```

**Impact:** Anyone could upload activities as any user by passing their UUID.

**Fix:** Added `AuthUser` extractor, removed `user_id` from query params:

```rust
// AFTER (secure)
pub async fn new_activity(
    AuthUser(claims): AuthUser,  // Requires valid token
    Query(params): Query<UploadQuery>,  // No user_id
) {
    let user_id = claims.sub;  // From verified token
}
```

## Audit Findings

### Secure Patterns Found

1. **Team permission handlers** - Use `AuthUser` and check membership via DB
2. **Follow endpoints** - Use `AuthUser`, path param is target (not identity)
3. **Read endpoints** - Use `OptionalAuthUser` for visibility checks
4. **Visibility checks** - Properly check owner OR team membership

### Patterns to Watch

Path parameters with user IDs are OK when they represent the TARGET of an action (e.g., "follow this user"), but user IDENTITY must always come from `AuthUser`.

## Testing Approach

Used browser automation to verify:
1. Unauthenticated upload → 401
2. Authenticated upload → proceeds
3. Token roundtrip (create → verify)
4. Invalid/tampered token rejection

## Recommendations Applied

1. Added security documentation: `docs/architecture/security.md`
2. Updated CLAUDE.md with security rules
3. Updated context.md with auth patterns
4. All mutation endpoints should be audited for `AuthUser` usage
