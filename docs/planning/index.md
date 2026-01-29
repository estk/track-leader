# Track Leader - Project Status

**Last Updated:** 2026-01-28
**Status:** All Phases Complete - Ready for Launch

## Completed Phases

| Phase | Focus | Status |
|-------|-------|--------|
| Phase 1 | Foundation & Auth | ✅ Complete |
| Phase 2 | Core Features | ✅ Complete |
| Phase 3 | Segments | ✅ Complete |
| Phase 4 | Leaderboards | ✅ Complete |
| Phase 5 | Social Features | ✅ Complete |
| Phase 6 | Polish & Launch | ✅ Complete |

Phase documentation has been archived.

---

## Current Capabilities

### Backend
- User authentication (JWT + argon2)
- GPX file upload and processing
- Track storage with PostGIS LineStringZM (4D geometry)
- Segment creation and matching
- Filtered leaderboards (scope, gender, age group)
- Social features (follows, kudos, comments, notifications)
- Activity feed
- Global leaderboards (crowns, distance)

### Frontend
- Full authentication flow
- Activity upload, view, edit, delete
- Interactive maps with elevation profiles
- Segment creation from activities
- Leaderboard filtering and pagination
- User profiles with demographics
- Notification system
- Mobile responsive design

### Infrastructure
- E2E tests (Playwright, 17 tests)
- Load tests (k6)
- Production Docker config
- Operations runbook

---

## Launch Checklist

### Pre-Launch
- [x] All features working
- [x] E2E tests passing
- [x] Load testing scripts ready
- [x] Documentation complete
- [x] Production Docker config
- [ ] Deploy to production
- [ ] Configure domain and SSL
- [ ] Set up monitoring

### Launch Day
- [ ] Deploy to production
- [ ] Verify all systems
- [ ] Enable public signup
- [ ] Publish announcement

### Post-Launch
- [ ] Monitor metrics
- [ ] Address urgent bugs
- [ ] Plan next iteration

---

## Deferred Features

These items were planned but deferred for post-launch:

### Leaderboard features
- Much richer leaderboard filters
  - Equipment type (e-bike vs acoustic)
  - Weight class 

### Refactor tasks
- Refactor any api endpoints in the node application's api.ts into the rust api
- Openapi spec generation from tracks crate and then generate a client for node rather than have a custom built one, maybe use utopia

### General
- Strava import
- Mobile app (PWA or native)

### Need mobile app
- Live tracking, based on your privacy prefs (teams, followers etc) you can show your live location to logged in users with the right permissions
- Allow manual logging of events such as shuttle entry/exit, dig start/end

### Real-Time Updates
- SSE for leaderboard updates
- SSE for notification updates

### Perf
- Virtual scrolling for long lists
- Leaderboard caching service
- Rate limiting integration (tower_governor in Cargo.toml)

### Low priority
- User defined activity metrics that allow them to create and share a custom leaderboard
- Screen reader testing
- Sentry error tracking integration
- Internationalization

---

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Frontend | Next.js 14 | React ecosystem, App Router, good DX |
| Backend | Rust + Axum | Performance, type safety |
| Database | PostgreSQL + PostGIS | Spatial queries, reliability |
| Auth | JWT | Simple, stateless |
| Maps | MapLibre GL | Open source, performant |
| Track Storage | PostGIS LineStringZM | 4D geometry for elevation + time |

---

## Learnings Log

### Phase 6
- Lazy loading reduces bundle size significantly (384kB → 6.4kB for segments page)
- Playwright selectors need specificity when page has duplicate text
- validator crate derive macros are cleaner than manual validation
- PostGIS LineStringZM efficiently stores 4D track data

### Key Technical Learnings
- Denormalized counts avoid expensive COUNT queries
- Actor/target pattern for notifications is flexible
- Route ordering in Axum: specific routes before wildcards
- Auth state may not be set when useEffect runs

See [../ai/index.md](../ai/index.md) for full AI context documentation.

---

## Open Questions

1. **Strava import** - Legal considerations for activity import?
2. **Premium features** - What goes behind paywall?
3. **Mobile apps** - Native iOS/Android or PWA?
4. **API access** - Public API for third parties?
