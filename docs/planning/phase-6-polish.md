# Phase 6: Polish & Launch

**Duration:** Month 6 (3-4 weeks)
**Goal:** Production-ready release with polished UX

> **Claude Agents:** Use `/code-review` for security audit and final review. Use `/frontend-design` for marketing site.

---

## Known Bugs (from Phase 5 Testing)

These bugs were discovered during Phase 5 manual verification and should be fixed early in Phase 6:

### BUG-P6-001: Activity Detail Page Shows "Not Found"

**Severity:** High
**Location:** `/activities/[id]` page
**Status:** Open - Requires Tracks Table Refactoring

**Description:**
Clicking on an activity (e.g., "reno tour") from the activities list navigates to the activity detail page, but it shows "Not found" instead of the activity details.

**Root Cause (Investigated):**
The `/activities/{id}/track` endpoint returns 404 because it tries to read the raw GPX file from object storage, but the file doesn't exist (activity metadata was created without uploading actual GPX).

**Architectural Problem:**
Currently `get_activity_track` re-parses the GPX file from object storage on every request. This is:
1. Inefficient (parsing on every request)
2. Fragile (depends on GPX file existing in object store)
3. Duplicative (we already parse GPX during upload)

**Fix Required - Tracks Table Refactoring:**
Extend the existing `tracks` table to store elevation and timestamp data alongside the geometry:

1. **Migration** (`014_tracks_elevation_time.sql` - already created):
   ```sql
   ALTER TABLE tracks
   ADD COLUMN elevations double precision[],
   ADD COLUMN recorded_times timestamptz[];
   ```

2. **Update `save_track_geometry`** to also save elevation/time arrays when processing GPX

3. **Update `get_activity_track`** to read from `tracks` table instead of re-parsing GPX:
   - Extract points from `geo` LineString using `ST_DumpPoints`
   - Combine with `elevations` and `recorded_times` arrays
   - Return TrackData response

4. **Backfill existing activities** - Re-process any activities that have GPX files to populate the new columns

**Files to modify:**
- `crates/tracks/src/database.rs` - Update save_track_geometry, add get_track_data
- `crates/tracks/src/handlers.rs` - Update get_activity_track to use DB
- `crates/tracks/src/activity_queue.rs` - Save elevation/time during processing

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
- [ ] Audit with Lighthouse
- [ ] Implement code splitting
- [ ] Lazy load non-critical components
- [ ] Optimize images (WebP, lazy loading)
- [ ] Add loading skeletons everywhere
- [ ] Implement virtual scrolling for long lists
- [ ] Prefetch likely navigation targets

**Performance Targets:**
| Metric | Target |
|--------|--------|
| First Contentful Paint | < 1.2s |
| Largest Contentful Paint | < 2.0s |
| Time to Interactive | < 3.0s |
| Cumulative Layout Shift | < 0.1 |

### 1.2 Map Performance

**Tasks:**
- [ ] Simplify routes for initial render
- [ ] Progressive detail loading
- [ ] Cluster markers for segment browser
- [ ] Optimize tile loading
- [ ] Cache map tiles aggressively

### 1.3 Backend Performance

**Tasks:**
- [ ] Profile slow queries
- [ ] Add missing indexes
- [ ] Implement query result caching
- [ ] Connection pool tuning
- [ ] Response compression (gzip/brotli)

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
- [ ] Configure Cloudflare CDN
- [ ] Cache static assets (1 year)
- [ ] Cache API responses where appropriate
- [ ] Implement stale-while-revalidate
- [ ] Set up edge caching for leaderboards

---

## Week 2: Security & Quality

### 2.1 Security Audit

**Tasks:**
- [ ] Review authentication flow
- [ ] Audit API endpoints for authorization
- [ ] Check for SQL injection (sqlx should be safe)
- [ ] Check for XSS vectors
- [ ] Implement CSRF protection
- [ ] Rate limiting on all endpoints
- [ ] Security headers (CSP, HSTS, etc.)

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
- [ ] Validate all API inputs
- [ ] Sanitize user-generated content
- [ ] Limit file upload sizes
- [ ] Validate GPX file structure
- [ ] Add request body size limits

### 2.3 Error Handling

**Tasks:**
- [ ] Ensure no sensitive data in error messages
- [ ] Implement global error boundary (frontend)
- [ ] Custom 404 and 500 pages
- [ ] Error tracking integration (Sentry)
- [ ] User-friendly error messages

### 2.4 Accessibility

**Tasks:**
- [ ] Audit with axe-core
- [ ] Add ARIA labels
- [ ] Keyboard navigation throughout
- [ ] Focus management
- [ ] Color contrast compliance
- [ ] Screen reader testing
- [ ] Reduced motion support

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
- [ ] Design landing page
- [ ] Hero section with value proposition
- [ ] Feature showcase
- [ ] Screenshots/videos
- [ ] Pricing section (if applicable)
- [ ] FAQ section
- [ ] Call-to-action (signup)
- [ ] Mobile responsive

**Key Messages:**
1. "Open leaderboards for every trail"
2. "Compete on your terms"
3. "Community-driven segments"
4. "Your data, your choice"

### 3.2 User Documentation

**Tasks:**
- [ ] Getting started guide
- [ ] Activity upload guide
- [ ] Creating segments guide
- [ ] Understanding leaderboards
- [ ] Privacy settings guide
- [ ] FAQ compilation
- [ ] Video tutorials (optional)

### 3.3 API Documentation

**Tasks:**
- [ ] OpenAPI/Swagger spec
- [ ] Interactive API documentation
- [ ] Authentication guide
- [ ] Rate limiting documentation
- [ ] Example requests/responses
- [ ] Error code reference

### 3.4 Developer Documentation

**Tasks:**
- [ ] Architecture overview
- [ ] Local development setup
- [ ] Deployment guide
- [ ] Contributing guidelines
- [ ] Code style guide
- [ ] Database schema docs

---

## Week 4: Testing & Launch

### 4.1 Testing

**Tasks:**
- [ ] End-to-end test suite (Playwright)
- [ ] Load testing (k6 or similar)
- [ ] Mobile testing on real devices
- [ ] Cross-browser testing
- [ ] Regression test critical paths

**Load Test Targets:**
| Endpoint | Target RPS | p95 Latency |
|----------|------------|-------------|
| GET /feed | 100 | < 200ms |
| GET /leaderboard | 200 | < 150ms |
| POST /activities | 10 | < 2s |
| GET /activities/{id} | 500 | < 100ms |

### 4.2 Beta Program

**Tasks:**
- [ ] Create beta signup form
- [ ] Invite initial beta testers
- [ ] Set up feedback channels
- [ ] Bug reporting process
- [ ] Beta tester communication plan
- [ ] Iterate based on feedback

**Beta Timeline:**
- Week 1: Internal testing
- Week 2: Closed beta (50 users)
- Week 3: Expanded beta (200 users)
- Week 4: Open beta

### 4.3 Launch Preparation

**Tasks:**
- [ ] Production environment setup
- [ ] Database backup strategy
- [ ] Monitoring setup (metrics, logs, alerts)
- [ ] Runbook for common issues
- [ ] On-call rotation setup
- [ ] Status page (Instatus, Statuspage)

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
- [ ] Lighthouse score > 90 on all metrics
- [ ] API p95 < 200ms for reads
- [ ] Page load < 2s on 3G

**Security:**
- [ ] No critical/high vulnerabilities
- [ ] All security headers in place
- [ ] Rate limiting active
- [ ] Error messages sanitized

**Quality:**
- [ ] WCAG 2.1 AA compliant
- [ ] Works on Chrome, Firefox, Safari, Edge
- [ ] Works on iOS Safari, Android Chrome
- [ ] E2E tests passing

**Documentation:**
- [ ] User docs complete
- [ ] API docs complete
- [ ] Developer docs complete

**Operations:**
- [ ] Production deployed
- [ ] Monitoring active
- [ ] Backups configured
- [ ] Alerts set up

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
