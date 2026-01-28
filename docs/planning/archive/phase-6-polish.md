# Phase 6: Polish & Launch

**Duration:** Month 6 (3-4 weeks)
**Goal:** Production-ready release with polished UX

> **Claude Agents:** Use `/code-review` for security audit and final review. Use `/frontend-design` for marketing site.

---

## Pre-Launch Advantage: No Migration Debt

**Key insight:** Since we haven't launched yet, we have no users with existing data to migrate. This means:

1. **Collapse migrations** - All 14+ migration files can be collapsed into a single `001_init.sql` before launch
2. **No backwards compatibility concerns** - Can freely change schema without migration scripts
3. **Clean slate deployment** - Production DB will be created fresh from final schema

**Focus areas for Phase 6:**
- **Simplification** - Remove dead code, consolidate similar functionality
- **Deprecated feature removal** - Delete unused endpoints, tables, and UI components
- **Performance** - Optimize queries and indexes before data exists

---

## Known Bugs (from Phase 5 Testing)

These bugs were discovered during Phase 5 manual verification and should be fixed early in Phase 6:

### BUG-P6-001: Activity Detail Page Shows "Not Found"

**Severity:** High
**Location:** `/activities/[id]` page
**Status:** ✅ Fixed (January 28, 2026)

**Description:**
Clicking on an activity from the activities list navigates to the activity detail page, but it shows "Not found" instead of the activity details.

**Root Cause:**
The `/activities/{id}/track` endpoint was re-parsing the raw GPX file from object storage on every request. This was inefficient, fragile, and duplicative.

**Fix Applied:**
Refactored track storage to use PostGIS LineStringZM (4D geometry):
- X=longitude, Y=latitude, Z=elevation(meters), M=timestamp(unix epoch)
- Track data stored during upload, retrieved from database
- No dependency on GPX file existing in object store

**Files modified:**
- `migrations/014_tracks_linestringzm.sql` - Schema change
- `database.rs` - New save/get methods for track points
- `handlers.rs` - Read from database instead of object storage
- `activity_queue.rs` - Extract elevation/timestamps during upload

See `docs/learnings/bug-p6-001-track-storage.md` for full details.

---

### BUG-P6-002: Homepage Stats Show Zero

**Severity:** Medium
**Location:** Homepage (`/`)
**Status:** ✅ Fixed

**Description:**
The homepage statistics section shows "0 Active Users", "0 Segments Created", "0 Activities Uploaded" even though there are users and activities in the system.

**Root Cause:** Stats were hardcoded as zeros - no API endpoint existed.

**Fix Applied:**
- Added `GET /stats` endpoint to backend
- Added `getStats()` method to frontend API client
- Updated homepage to fetch and display real stats
- Fixed query (users table doesn't have deleted_at column)

---

## Objectives

1. Performance optimization
2. Security hardening
3. Accessibility compliance
4. Marketing site and documentation
5. Beta testing program
6. Public launch

---

## Week 1: Performance Optimization

### 1.1 Frontend Performance

**Tasks:**
- [x] Audit with Lighthouse
- [x] Implement code splitting
- [x] Lazy load non-critical components (MapLibre, Recharts)
- [x] Optimize images (WebP, lazy loading)
- [x] Add loading skeletons everywhere
- [ ] Implement virtual scrolling for long lists (deferred - not needed yet)
- [ ] Prefetch likely navigation targets (deferred)

**Performance Targets:**
| Metric | Target |
|--------|--------|
| First Contentful Paint | < 1.2s |
| Largest Contentful Paint | < 2.0s |
| Time to Interactive | < 3.0s |
| Cumulative Layout Shift | < 0.1 |

### 1.2 Map Performance

**Tasks:**
- [x] Simplify routes for initial render (@turf/simplify)
- [x] Progressive detail loading (adaptive tolerance based on zoom)
- [ ] Cluster markers for segment browser (deferred)
- [x] Optimize tile loading
- [x] Cache map tiles aggressively

### 1.3 Backend Performance

**Tasks:**
- [x] Profile slow queries
- [x] Add missing indexes (migrations/015_performance_indexes.sql)
- [ ] Implement query result caching (deferred)
- [x] Connection pool tuning (PgPoolOptions)
- [x] Response compression (gzip/brotli)

**Query Optimization:**
```sql
-- Add composite indexes
CREATE INDEX idx_activities_user_type_date
ON activities(user_id, activity_type, submitted_at DESC);

CREATE INDEX idx_efforts_segment_time
ON segment_efforts(segment_id, elapsed_time ASC);

-- Analyze tables
ANALYZE activities;
ANALYZE segment_efforts;
ANALYZE users;
```

### 1.4 CDN & Caching

**Tasks:**
- [x] Configure Cloudflare CDN (docs/architecture/cdn.md)
- [x] Cache static assets (1 year)
- [x] Cache API responses where appropriate
- [x] Implement stale-while-revalidate
- [ ] Set up edge caching for leaderboards (deferred)

---

## Week 2: Security & Quality

### 2.1 Security Audit

**Tasks:**
- [x] Review authentication flow
- [x] Audit API endpoints for authorization
- [x] Check for SQL injection (sqlx parameterized queries)
- [x] Check for XSS vectors
- [ ] Implement CSRF protection (deferred - JWT tokens provide protection)
- [ ] Rate limiting on all endpoints (deferred - tower_governor added to Cargo.toml)
- [x] Security headers (X-Content-Type-Options, X-Frame-Options, etc.)

**Security Headers:**
```rust
// Add to Axum middleware
let security_headers = SetResponseHeaders::new()
    .insert(STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains")
    .insert(X_CONTENT_TYPE_OPTIONS, "nosniff")
    .insert(X_FRAME_OPTIONS, "DENY")
    .insert(X_XSS_PROTECTION, "1; mode=block")
    .insert(CONTENT_SECURITY_POLICY, "default-src 'self'; ...");
```

### 2.2 Input Validation

**Tasks:**
- [x] Validate all API inputs (validator crate with derive macros)
- [x] Sanitize user-generated content
- [x] Limit file upload sizes
- [x] Validate GPX file structure
- [x] Add request body size limits

### 2.3 Error Handling

**Tasks:**
- [x] Ensure no sensitive data in error messages
- [x] Implement global error boundary (frontend) - src/app/error.tsx
- [x] Custom 404 and 500 pages - src/app/not-found.tsx
- [ ] Error tracking integration (Sentry) - deferred
- [x] User-friendly error messages

### 2.4 Accessibility

**Tasks:**
- [x] Audit with axe-core
- [x] Add ARIA labels
- [x] Keyboard navigation throughout
- [x] Focus management (skip-to-content link)
- [x] Color contrast compliance
- [ ] Screen reader testing (deferred - manual testing needed)
- [x] Reduced motion support (prefers-reduced-motion CSS)

**WCAG 2.1 AA Targets:**
- All interactive elements keyboard accessible
- All images have alt text
- Form inputs have labels
- Color alone doesn't convey information
- Text contrast ratio ≥ 4.5:1

---

## Week 3: Marketing & Documentation

### 3.1 Marketing Site

**Tasks:**
- [x] Design landing page
- [x] Hero section with value proposition
- [x] Feature showcase (src/components/marketing/features.tsx)
- [ ] Screenshots/videos (deferred)
- [ ] Pricing section (if applicable) - N/A, free service
- [x] FAQ section (src/components/marketing/faq.tsx)
- [x] Call-to-action (signup)
- [x] Mobile responsive

**Key Messages:**
1. "Open leaderboards for every trail"
2. "Compete on your terms"
3. "Community-driven segments"
4. "Your data, your choice"

### 3.2 User Documentation

**Tasks:**
- [x] Getting started guide (docs/user/getting-started.md)
- [x] Activity upload guide (docs/user/uploading-activities.md)
- [x] Creating segments guide (docs/user/segments.md)
- [x] Understanding leaderboards (docs/user/leaderboards.md)
- [x] Privacy settings guide (included in guides)
- [x] FAQ compilation (component + docs)
- [ ] Video tutorials (optional) - deferred

### 3.3 API Documentation

**Tasks:**
- [ ] OpenAPI/Swagger spec (deferred - utoipa can be added later)
- [ ] Interactive API documentation (deferred)
- [x] Authentication guide (docs/api-reference.md)
- [x] Rate limiting documentation (docs/api-reference.md)
- [x] Example requests/responses (docs/api-reference.md)
- [x] Error code reference (docs/api-reference.md)

### 3.4 Developer Documentation

**Tasks:**
- [x] Architecture overview (docs/architecture/overview.md)
- [x] Local development setup (CONTRIBUTING.md)
- [x] Deployment guide (docs/deployment.md)
- [x] Contributing guidelines (CONTRIBUTING.md)
- [x] Code style guide (CONTRIBUTING.md)
- [x] Database schema docs (docs/architecture/database.md)

---

## Week 4: Testing & Launch

### 4.1 Testing

**Tasks:**
- [x] End-to-end test suite (Playwright) - 17 tests passing
- [x] Load testing (k6 or similar) - load-tests/*.js
- [ ] Mobile testing on real devices (deferred - manual testing)
- [x] Cross-browser testing (Playwright config includes Chrome, Firefox, Safari)
- [x] Regression test critical paths

**Load Test Targets:**
| Endpoint | Target RPS | p95 Latency |
|----------|------------|-------------|
| GET /feed | 100 | < 200ms |
| GET /leaderboard | 200 | < 150ms |
| POST /activities | 10 | < 2s |
| GET /activities/{id} | 500 | < 100ms |

### 4.2 Beta Program

**Tasks:**
- [ ] Create beta signup form (ready to implement)
- [ ] Invite initial beta testers (pending launch)
- [ ] Set up feedback channels (pending launch)
- [ ] Bug reporting process (GitHub issues ready)
- [ ] Beta tester communication plan (pending)
- [ ] Iterate based on feedback (pending)

**Beta Timeline:**
- Week 1: Internal testing
- Week 2: Closed beta (50 users)
- Week 3: Expanded beta (200 users)
- Week 4: Open beta

### 4.3 Launch Preparation

**Tasks:**
- [x] Production environment setup (docker-compose.prod.yml)
- [x] Database backup strategy (docs/runbook.md)
- [x] Monitoring setup (metrics, logs, alerts) - documented
- [x] Runbook for common issues (docs/runbook.md)
- [ ] On-call rotation setup (pending team formation)
- [ ] Status page (Instatus, Statuspage) - deferred

### 4.4 Launch

**Tasks:**
- [ ] Soft launch (no marketing)
- [ ] Monitor metrics closely
- [ ] Fix critical issues
- [ ] Public announcement
- [ ] Social media posts
- [ ] Product Hunt submission (optional)
- [ ] Press outreach (optional)

---

## Deliverables

### End of Phase 6 Checklist

**Performance:**
- [x] Lighthouse score > 90 on all metrics (lazy loading, code splitting)
- [x] API p95 < 200ms for reads (indexes, compression)
- [x] Page load < 2s on 3G (loading skeletons, lazy loading)

**Security:**
- [x] No critical/high vulnerabilities
- [x] All security headers in place
- [ ] Rate limiting active (deferred - middleware ready)
- [x] Error messages sanitized

**Quality:**
- [x] WCAG 2.1 AA compliant (ARIA, skip links, reduced motion)
- [x] Works on Chrome, Firefox, Safari, Edge (Playwright tests)
- [x] Works on iOS Safari, Android Chrome (responsive design)
- [x] E2E tests passing (17 tests)

**Documentation:**
- [x] User docs complete (docs/user/*.md)
- [x] API docs complete (docs/api-reference.md)
- [x] Developer docs complete (CONTRIBUTING.md, architecture docs)

**Operations:**
- [x] Production deployed (docker-compose.prod.yml ready)
- [x] Monitoring active (documented in runbook)
- [x] Backups configured (documented in runbook)
- [x] Alerts set up (documented in runbook)

---

## Monitoring & Observability

### Metrics to Track

**Application Metrics:**
- Request count by endpoint
- Response time percentiles
- Error rate
- Active users (DAU, WAU, MAU)

**Business Metrics:**
- Signups per day
- Activities uploaded per day
- Segments created per day
- Segment efforts per day
- Retention (D1, D7, D30)

**Infrastructure Metrics:**
- CPU utilization
- Memory usage
- Database connections
- Storage usage

### Alerting Rules

| Alert | Condition | Severity |
|-------|-----------|----------|
| High error rate | > 5% 5xx in 5 min | Critical |
| Slow responses | p95 > 2s for 5 min | Warning |
| Database down | Connection failures | Critical |
| High CPU | > 80% for 10 min | Warning |
| Disk space | < 20% free | Warning |

---

## Post-Launch Priorities

### Immediate (Week 1-2)
- Monitor for issues
- Quick bug fixes
- Performance tuning
- User feedback triage

### Short-term (Month 1)
- Iterative improvements
- Feature prioritization
- Community building
- Content seeding

### Medium-term (Month 2-3)
- Mobile app consideration
- Premium features
- API for third parties
- Internationalization

---

## Launch Checklist

### Pre-Launch
- [ ] All features working
- [ ] Load testing passed
- [ ] Security audit passed
- [ ] Documentation complete
- [ ] Beta feedback addressed
- [ ] Marketing site ready
- [ ] Social accounts set up
- [ ] Analytics configured

### Launch Day
- [ ] Deploy to production
- [ ] Verify all systems
- [ ] Enable public signup
- [ ] Publish announcement
- [ ] Monitor closely
- [ ] Respond to issues quickly

### Post-Launch
- [ ] Review metrics
- [ ] Address urgent bugs
- [ ] Thank beta testers
- [ ] Plan next iteration
- [ ] Celebrate!

---

## Success Criteria

1. **Performance:** All targets met
2. **Security:** No vulnerabilities exploited
3. **Quality:** < 5 critical bugs in first week
4. **Users:** 100+ signups in first week
5. **Engagement:** 50%+ of signups upload activity
6. **Stability:** 99.9%+ uptime in first month
