# Track Leader - 6-Month Development Plan

## Implementation Progress

**Current Phase:** Phase 6 - Polish & Launch (Complete)
**Last Updated:** 2026-01-28 (Phase 6 complete)

### Phase 1 Progress (Complete)

| Task | Status | Notes |
|------|--------|-------|
| Archive broken frontend | Done | Moved to `_archive/` |
| Initialize fresh Next.js | Done | `src/` structure, App Router |
| Set up design system | Done | Tailwind + shadcn-style components |
| Configure backend proxy | Done | Next.js rewrites to port 3001 |
| Implement backend auth | Done | JWT + argon2, /auth/register, /auth/login, /auth/me |
| Create login/register pages | Done | Frontend forms with validation |
| Create API client | Done | `src/lib/api.ts` with auth methods |
| Auth context provider | Done | React context for global auth state |
| Activities list page | Done | Shows user activities with upload button |
| Activity upload page | Done | GPX file upload form |
| Fix database constraints | Done | Migration 002 created |
| Set up CI/CD | Done | GitHub Actions workflow for Rust + Next.js |
| Deploy staging | Pending | Requires Fly.io/infrastructure setup |

### Phase 2 Progress (Complete)

| Task | Status | Notes |
|------|--------|-------|
| Install MapLibre GL | Done | maplibre-gl |
| Install Recharts | Done | For elevation profile |
| Activity track API endpoint | Done | GET /activities/{id}/track returns parsed GPX |
| Activity map component | Done | OpenTopoMap tiles with contour lines |
| Elevation profile component | Done | Syncs with map on hover |
| Activity detail page | Done | Shows map, elevation, stats |
| Activity cards clickable | Done | Navigate to detail page |
| Activity edit | Done | PATCH endpoint + edit modal |
| Activity delete | Done | DELETE endpoint + confirmation dialog |
| Fix date serialization | Done | Added rfc3339 serde to OffsetDateTime fields |
| Fix PostCSS config | Done | Created missing postcss.config.js |
| Fix activity types | Done | Frontend sends PascalCase to match backend enum |
| Privacy controls | Done | Public/private visibility toggle on upload and edit |
| User profile page | Done | Shows user info and activity summary |
| Mobile responsive | Done | Hamburger menu, responsive layouts |

### Phase 3 Progress (Complete)

| Task | Status | Notes |
|------|--------|-------|
| Segments database schema | Done | Migration 003 with PostGIS geometry |
| Segment models | Done | Segment, SegmentEffort structs |
| Segment API endpoints | Done | Create, get, list, leaderboard, track, reprocess |
| Segments list page | Done | Shows public segments with stats |
| Segment detail page | Done | Map, elevation profile, stats, leaderboard with PRs |
| Segment creation UI | Done | Click elevation profile to select start/end points |
| Automatic segment matching | Done | PostGIS ST_DWithin + ST_LineLocatePoint |
| Personal records tracking | Done | update_personal_records() + PR badges on leaderboard |
| Track geometry storage | Done | LINESTRING in tracks table with GIST index |
| Auto-reprocess on segment create | Done | Finds existing activities when segment created |
| Starred segments | Done | API endpoints and UI |

### Phase 4 Progress (Complete)

| Task | Status | Notes |
|------|--------|-------|
| Demographics migrations | Done | 008_add_demographics.sql |
| Leaderboard cache schema | Done | 009_leaderboard_cache.sql |
| Achievements schema | Done | 010_achievements.sql |
| Filtered leaderboard endpoint | Done | Scope, gender, age group filters |
| Demographics API | Done | PATCH /users/me/demographics |
| Achievements API | Done | User and segment achievement endpoints |
| Global leaderboards API | Done | Crown count, distance rankings |
| Leaderboard table component | Done | Paginated, medals, current user highlight |
| Leaderboard filters component | Done | URL state persistence |
| Crown badge component | Done | KOM, QOM, Local Legend badges |
| Full leaderboard page | Done | /segments/[id]/leaderboard |
| Profile settings page | Done | Demographics form |
| Achievements page | Done | Crown gallery with filters |
| Rankings page | Done | Personal segment rankings |
| Global leaderboards page | Done | /leaderboards with tabs |

**Deferred to Polish Phase:**
- SSE real-time leaderboard updates
- Leaderboard caching service
- Auto-achievement processing

### Phase 5 Progress (Complete)

| Task | Status | Notes |
|------|--------|-------|
| Follows database schema | Done | 011_social_follows.sql with denormalized counts |
| Notifications database schema | Done | 012_notifications.sql with actor/target pattern |
| Kudos/comments database schema | Done | 013_kudos_comments.sql with denormalized counts |
| Follow/unfollow API | Done | POST/DELETE /users/{id}/follow |
| Followers/following lists API | Done | GET with pagination |
| Notifications API | Done | GET, mark read, mark all read |
| Activity feed API | Done | GET /feed from followed users |
| Kudos API | Done | POST/DELETE /activities/{id}/kudos |
| Comments API | Done | CRUD endpoints |
| FollowButton component | Done | Toggle with optimistic updates |
| FollowStats component | Done | Clickable follower/following counts |
| NotificationBell component | Done | Dropdown with unread badge |
| Notifications page | Done | Full page with mark read |
| Feed page | Done | /feed with activity cards |
| Public profile page | Done | /profile/[userId] with follow button |
| Followers/following pages | Done | User lists with avatars |
| KudosButton component | Done | Toggle with count |
| CommentsSection component | Done | Expand/collapse, add/delete |
| FeedCard component | Done | Activity card with kudos/comments |

**Bug Found & Fixed:**
- FollowButton race condition - button rendered before async follow status loaded. Fixed by adding `followStatusLoaded` state.

### Phase 6 Progress (Complete)

| Task | Status | Notes |
|------|--------|-------|
| Bundle analyzer | Done | next.config.js with @next/bundle-analyzer |
| Lazy loading components | Done | MapLibre and Recharts load on demand |
| Loading skeletons | Done | activities, segments, feed, leaderboards routes |
| Route simplification | Done | @turf/simplify for large GPS tracks |
| Performance indexes | Done | Migration 015_performance_indexes.sql |
| Connection pool tuning | Done | PgPoolOptions in main.rs |
| Gzip compression | Done | tower-http CompressionLayer |
| CDN caching headers | Done | next.config.js headers() |
| Security headers | Done | X-Content-Type-Options, X-Frame-Options, etc. |
| Input validation | Done | validator crate with derive macros |
| Error boundary | Done | src/app/error.tsx |
| Custom 404 page | Done | src/app/not-found.tsx |
| Skip-to-content link | Done | Accessibility improvement |
| ARIA labels | Done | Header and navigation |
| Reduced motion support | Done | prefers-reduced-motion in CSS |
| Marketing components | Done | Features and FAQ sections |
| User documentation | Done | docs/user/*.md |
| API documentation | Done | docs/api-reference.md updated |
| Architecture docs | Done | docs/architecture/overview.md |
| Deployment guide | Done | docs/deployment.md |
| Contributing guide | Done | CONTRIBUTING.md |
| E2E tests (Playwright) | Done | 17 tests passing |
| Load tests (k6) | Done | load-tests/*.js |
| Performance docs | Done | docs/performance.md |
| Production Docker config | Done | docker-compose.prod.yml |
| Frontend Dockerfile | Done | Dockerfile.frontend |
| Runbook | Done | docs/runbook.md |

**Bugs Fixed:**
- BUG-P6-001: Activity track storage refactored to PostGIS LineStringZM
- BUG-P6-002: Homepage stats now fetch from /stats API endpoint

---

## Vision

Transform Track Leader from a GPS activity tracker into an **open leaderboard platform for trail segments** - a legitimate competitor to Strava's segment feature with key differentiators:

1. **Open by default** - Public segments, transparent rankings
2. **User-defined metrics** - Compete on any measurable dimension
3. **Trail-first design** - Split activities into shareable trail routes
4. **Community-driven** - Segment creation, verification, and curation

## Development Phases

| Phase | Focus | Duration | Status |
|-------|-------|----------|--------|
| [Phase 1](./phase-1-foundation.md) | Foundation & Auth | Month 1 | ✅ Complete |
| [Phase 2](./phase-2-core-features.md) | Core Features | Month 2 | ✅ Complete |
| [Phase 3](./phase-3-segments.md) | Segments | Month 3 | ✅ Complete |
| [Phase 4](./phase-4-leaderboards.md) | Leaderboards | Month 4 | ✅ Complete |
| [Phase 5](./phase-5-social.md) | Social Features | Month 5 | ✅ Complete |
| [Phase 6](./phase-6-polish.md) | Polish & Launch | Month 6 | ✅ Complete |

---

## Learnings Log

### Phase 6 Learnings (2026-01-28)

1. **Lazy loading reduces initial bundle significantly** - Splitting MapLibre and Recharts into dynamic imports reduced the segments detail page bundle from 384kB to 6.4kB.

2. **Playwright selectors need specificity** - When page content has duplicate text (e.g., "Create Segments" in hero and features), use role-based selectors like `getByRole("heading", { name: /create segments/i })`.

3. **validator crate derive macros are clean** - Using `#[derive(Validate)]` with field attributes like `#[validate(email)]` is more readable than manual validation code.

4. **PostGIS LineStringZM for 4D tracks** - Storing X=lon, Y=lat, Z=elevation, M=timestamp in a single geometry column is efficient and enables spatial queries on time-series GPS data.

5. **Documentation drives architecture understanding** - Writing docs/architecture/overview.md forced clarity on component boundaries and data flow.

### Phase 5 Learnings (2026-01-27)

1. **Async state initialization race conditions** - When a React component's initial state depends on an async call, render conditional UI (loading state or delayed render) until the data is ready. Don't render components with `initialX` props until you've actually fetched X.

2. **Denormalized counts work well** - Storing `follower_count`, `kudos_count`, etc. directly on parent tables (users, activities) avoids expensive COUNT queries. Update counts atomically in the same transaction as the relationship change.

3. **Actor/target pattern for notifications** - Using `actor_id` (who did it) and `target_type`/`target_id` (what it was done to) makes notifications flexible and queryable. Easy to add new notification types without schema changes.

4. **Next.js dev server caching issues** - After modifying files, sometimes the dev server caches stale webpack chunks. Restart with `tmux respawn-pane` to clear the cache.

5. **Browser automation for verification** - Using Claude-in-Chrome MCP tools enables systematic manual testing with screenshots as evidence. Good for catching UI bugs that unit tests miss.

### Phase 4 Learnings (2026-01-27)

1. **LocalFileSystem requires existing directory** - `object_store::local::LocalFileSystem::new(path)` fails if directory doesn't exist. Create with `std::fs::create_dir_all()` first.

2. **Route ordering in Axum matters** - Specific routes must come before wildcard routes. `/segments/{id}/leaderboard` must be registered before `/segments/{id}`.

3. **tmux session management** - Use `remain-on-exit on` to keep panes alive after process exit. Restart with `tmux respawn-pane`.

4. **Port cleanup in dev scripts** - Kill stray processes with `lsof -ti :$port | xargs kill` before starting dev environment.

### Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Frontend framework | Next.js 14 | React ecosystem, App Router, good DX |
| Auth system | Email + JWT | Simple to start, OAuth can be added |
| API style | REST | Simple, cacheable |
| Real-time | SSE (planned) | Simpler than WebSockets for read-only updates |
| Maps | MapLibre GL | Open source, performant |

---

## Future Enhancements (Backlog)

Ideas collected during development that may be incorporated into future phases:

### Teams Feature (Phase 5 Extension or Phase 7)
- Create user teams
- Team visibility settings for segments, activities, routes
- Team home page with feed of published content
- Team-scoped leaderboards

### Data Quality Improvements
- Track GPS refresh rate in stats
- Segment matching tolerance based on data quality (point accuracy + refresh rate)
- Better handling of low-quality GPS data

### Developer Experience
- Protobuf or OpenAPI/Swagger for Rust-Node interface
- Auto-generate TypeScript types from backend schemas
- Synthetic test data generation

### Enhanced Leaderboard Filters
- Weight class filtering
- Equipment type (e.g., eMTB vs acoustic)
- More granular age brackets

---

## Technology Stack

### Frontend
| Technology | Purpose |
|------------|---------|
| Next.js 14 | Framework (App Router) |
| TypeScript | Type safety |
| Tailwind CSS | Styling |
| shadcn/ui | Component library |
| MapLibre GL | Maps |
| Recharts | Charts/graphs |
| TanStack Query | Data fetching |
| Zustand | State management |

### Backend
| Technology | Purpose |
|------------|---------|
| Rust + Axum | Web framework |
| PostgreSQL + PostGIS | Database |
| SQLx | Database access |
| object_store | File storage |
| Rayon | Parallel processing |

### Infrastructure
| Technology | Purpose |
|------------|---------|
| Fly.io | Hosting (recommended) |
| Cloudflare | CDN/DNS |
| S3-compatible | File storage |
| GitHub Actions | CI/CD |

---

## Open Questions

1. **Strava import** - Should we support importing activities from Strava? (Legal considerations)
2. **FIT/TCX support** - Priority for non-GPX formats?
3. **Premium features** - What goes behind paywall?
4. **Mobile apps** - Native iOS/Android or PWA?
5. **API access** - Public API for third parties?
6. **Internationalization** - Multi-language support timeline?

---

## Claude Code Agents & Skills

| Skill | Command | Use Case |
|-------|---------|----------|
| **feature-dev** | `/feature-dev` | Guided feature development with architecture focus |
| **frontend-design** | `/frontend-design` | Create distinctive, production-grade frontend interfaces |
| **code-review** | `/code-review` | Review pull requests before merge |
| **commit-msg** | `/commit-msg` | Generate consistent commit messages |

---

## Next Steps

**Phase 6 Complete - Ready for Launch**

1. **Launch Preparation**
   - Deploy to production using docker-compose.prod.yml
   - Configure domain and SSL certificates
   - Set up monitoring and alerting
   - Run production smoke tests

2. **Beta Program**
   - Create beta signup form
   - Invite initial testers (50 users)
   - Gather feedback and iterate
   - Expand to 200 users, then open beta

3. **Post-Launch Monitoring**
   - Track Lighthouse scores (target > 90)
   - Monitor API p95 latency (target < 200ms)
   - Watch error rates and user feedback
   - Quick bug fixes as needed

4. **Deferred Items (Post-Launch)**
   - SSE real-time updates for notifications and leaderboards
   - Leaderboard caching service
   - Auto-achievement processing on effort creation

5. **Future Enhancements (Phase 7+)**
   - Teams feature
   - Strava import
   - Mobile app (PWA or native)
   - Internationalization
