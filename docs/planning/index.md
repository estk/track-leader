# Track Leader - 6-Month Development Plan

## Implementation Progress

**Current Phase:** Phase 2 - Core Features
**Started:** 2026-01-26

### Phase 1 Progress (Complete except staging deploy)

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

### Phase 2 Progress

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
| Privacy controls | Pending | |
| User profile page | Pending | |
| Mobile responsive | Pending | |

**Latest Session:** 2026-01-27 - See `docs/session-notes.md` for detailed fixes and learnings.

---

## Vision

Transform Track Leader from a GPS activity tracker into an **open leaderboard platform for trail segments** - a legitimate competitor to Strava's segment feature with key differentiators:

1. **Open by default** - Public segments, transparent rankings
2. **User-defined metrics** - Compete on any measurable dimension
3. **Trail-first design** - Split activities into shareable trail routes
4. **Community-driven** - Segment creation, verification, and curation

## Current State

| Component | Status |
|-----------|--------|
| Rust Backend | Functional (activity upload/storage + auth) |
| Next.js Frontend | Basic scaffolding (landing, login, register) |
| Authentication | Backend complete, frontend pages done |
| Segments | Not implemented |
| Leaderboards | Not implemented |
| Social Features | Not implemented |

See [Current State Analysis](../current-state.md) for details.

## Development Phases

| Phase | Focus | Duration |
|-------|-------|----------|
| [Phase 1](./phase-1-foundation.md) | Foundation & Auth | Month 1 |
| [Phase 2](./phase-2-core-features.md) | Core Features | Month 2 |
| [Phase 3](./phase-3-segments.md) | Segments | Month 3 |
| [Phase 4](./phase-4-leaderboards.md) | Leaderboards | Month 4 |
| [Phase 5](./phase-5-social.md) | Social Features | Month 5 |
| [Phase 6](./phase-6-polish.md) | Polish & Launch | Month 6 |

## Phase Overview

### Phase 1: Foundation & Authentication

**Goal:** Establish solid infrastructure and user authentication.

- Delete broken frontend, initialize fresh Next.js with proper structure
- Implement authentication (OAuth + email/password)
- Complete backend fundamentals (foreign keys, proper error handling)
- Set up CI/CD pipeline
- Deploy staging environment

**Deliverables:**
- Users can register, login, logout
- Activities upload works end-to-end
- Deployed to staging

### Phase 2: Core Features

**Goal:** Build compelling activity management experience.

- Interactive activity map with elevation profile
- Activity list with search/filter/sort
- Basic user profile pages
- Activity privacy controls
- Mobile-responsive design

**Deliverables:**
- Users can upload, view, and manage activities
- Map shows route with interactive features
- Works well on mobile

### Phase 3: Segments

**Goal:** Implement the core segment system.

- Segment creation from activity portions
- Automatic segment matching on upload
- Segment detail pages
- Personal records tracking
- Segment discovery/search

**Deliverables:**
- Users can create segments
- Activities auto-match to segments
- Segment efforts tracked

### Phase 4: Leaderboards

**Goal:** Build competitive ranking system.

- Segment leaderboards (all-time, yearly, monthly, weekly)
- Demographic filters (age, gender, location)
- KOM/QOM crowns
- Personal ranking history
- Real-time updates

**Deliverables:**
- Leaderboards for every segment
- Users can filter by demographics
- Rankings update live

### Phase 5: Social Features

**Goal:** Build community engagement.

- Follow system
- Activity feed
- Kudos and comments
- Segment stars (favorites)
- Share functionality
- Notifications

**Deliverables:**
- Users can follow each other
- Feed shows followed users' activities
- Social interactions work

### Phase 6: Polish & Launch

**Goal:** Production-ready release.

- Performance optimization
- Security audit
- Documentation
- Marketing site
- Beta testing
- Public launch

**Deliverables:**
- Production deployment
- Public beta launch
- Initial user acquisition

---

## Technology Decisions

### Frontend Stack

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
| NextAuth.js v5 | Authentication |

### Backend Stack (Existing)

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

## Resource Requirements

### Single Developer Estimate

Assuming one full-time developer:

| Phase | Developer Days |
|-------|----------------|
| Phase 1 | 15-20 days |
| Phase 2 | 20-25 days |
| Phase 3 | 25-30 days |
| Phase 4 | 20-25 days |
| Phase 5 | 15-20 days |
| Phase 6 | 15-20 days |

**Total:** ~110-140 developer days (5-6 months full-time)

### Team Estimate (2-3 developers)

| Phase | Duration |
|-------|----------|
| Phase 1 | 2-3 weeks |
| Phase 2 | 3-4 weeks |
| Phase 3 | 4-5 weeks |
| Phase 4 | 3-4 weeks |
| Phase 5 | 2-3 weeks |
| Phase 6 | 2-3 weeks |

**Total:** ~4-5 months with 2-3 developers

---

## Risk Factors

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| PostGIS performance | High | Index optimization, query tuning |
| Segment matching accuracy | High | Robust algorithm, user feedback |
| Real-time leaderboards | Medium | Caching, eventual consistency |
| Mobile performance | Medium | Progressive loading, lazy hydration |

### Product Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| User adoption | High | Marketing, Strava import |
| Content bootstrapping | High | Seed with popular segments |
| Community moderation | Medium | Reporting, flagging systems |
| Competitive response | Low | Focus on differentiation |

---

## Success Metrics

### Launch Targets (End of Phase 6)

| Metric | Target |
|--------|--------|
| Registered users | 1,000 |
| Activities uploaded | 10,000 |
| Segments created | 500 |
| Daily active users | 100 |
| Page load time (p95) | < 2s |
| API response time (p95) | < 200ms |

### 12-Month Targets (Post-Launch)

| Metric | Target |
|--------|--------|
| Registered users | 50,000 |
| Monthly active users | 10,000 |
| Segments created | 10,000 |
| Premium conversions | 5% |

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

Use these Claude Code skills/plugins to accelerate development:

### Available Skills

| Skill | Command | Use Case |
|-------|---------|----------|
| **feature-dev** | `/feature-dev` | Guided feature development with architecture focus. Use for implementing major features in each phase. |
| **frontend-design** | `/frontend-design` | Create distinctive, production-grade frontend interfaces. Essential for Phase 1-2 UI work. |
| **code-review** | `/code-review` | Review pull requests before merge. Use before every PR. |
| **commit-msg** | `/commit-msg` | Generate consistent commit messages. Works with jj. |

### Phase-Specific Agent Recommendations

| Phase | Recommended Agents |
|-------|-------------------|
| Phase 1 | `/feature-dev` for auth system, `/frontend-design` for new UI scaffolding |
| Phase 2 | `/frontend-design` for activity pages, maps, charts |
| Phase 3 | `/feature-dev` for segment matching algorithm |
| Phase 4 | `/feature-dev` for leaderboard system |
| Phase 5 | `/frontend-design` for social UI components |
| Phase 6 | `/code-review` for final audit |

### Usage Tips

1. **Start features with `/feature-dev`** - It provides structured guidance and considers architecture
2. **Build UI with `/frontend-design`** - Creates polished, distinctive interfaces (not generic AI aesthetics)
3. **Review all PRs with `/code-review`** - Catches issues before merge
4. **Commit with `/commit-msg`** - Maintains consistent, descriptive commit history

### Installing Additional Plugins

```bash
# Check installed plugins
claude plugins

# Install a plugin
/plugin  # Then follow prompts
```

---

## Next Steps

1. Review and approve this plan
2. Begin Phase 1 implementation
3. Set up project tracking (Linear/GitHub Issues)
4. Establish weekly check-ins

See individual phase documents for detailed implementation plans.
