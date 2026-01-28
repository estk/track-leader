# Phase 1: Foundation & Authentication

**Duration:** Month 1 (4 weeks)
**Goal:** Establish solid infrastructure and user authentication

> **Claude Agents:** Use `/feature-dev` for auth implementation, `/frontend-design` for new UI scaffolding

---

## Objectives

1. Replace broken frontend with functional Next.js application
2. Implement user authentication (OAuth + email/password)
3. Fix backend fundamentals
4. Set up CI/CD and staging environment
5. Establish design system foundation

---

## Week 1: Frontend Reboot

### 1.1 Clean Slate

**Tasks:**
- [ ] Archive current frontend code for reference
- [ ] Remove broken components, API routes, lib imports
- [ ] Keep `package.json` dependencies worth keeping
- [ ] Update `.gitignore` to ignore new directories

### 1.2 Initialize New Frontend

**Tasks:**
- [ ] Run `npx create-next-app@latest` with:
  - TypeScript
  - Tailwind CSS
  - ESLint
  - App Router
  - `src/` directory
- [ ] Configure path aliases (`@/` → `src/`)
- [ ] Set up Prettier with Tailwind plugin
- [ ] Configure TypeScript strict mode

### 1.3 Install Core Dependencies

```bash
npm install @tanstack/react-query zustand
npm install next-auth@beta
npm install lucide-react
npm install clsx tailwind-merge
npx shadcn-ui@latest init
```

### 1.4 Set Up Design System

**Tasks:**
- [ ] Configure Tailwind theme (colors, fonts, spacing)
- [ ] Install shadcn/ui base components:
  - Button, Input, Card, Dialog
  - Form, Label, Select
  - Toast, Avatar, Badge
- [ ] Create base layout component
- [ ] Create navigation component (placeholder)

### 1.5 Configure Backend Proxy

**Tasks:**
- [ ] Add `next.config.js` rewrites to proxy API calls:
  ```javascript
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:3001/:path*',
      },
    ];
  }
  ```
- [ ] Update Rust backend to run on port 3001
- [ ] Test proxy with health check endpoint

---

## Week 2: Authentication

### 2.1 Backend Auth Infrastructure

**Tasks:**
- [ ] Add password_hash column to users table
- [ ] Install `argon2` crate for password hashing
- [ ] Create `POST /auth/register` endpoint
- [ ] Create `POST /auth/login` endpoint (returns JWT)
- [ ] Create auth middleware for protected routes
- [ ] Add JWT validation with `jsonwebtoken` crate

**Schema Migration:**
```sql
ALTER TABLE users ADD COLUMN password_hash TEXT;
ALTER TABLE users ADD COLUMN auth_provider TEXT DEFAULT 'email';
ALTER TABLE users ADD COLUMN external_id TEXT;
ALTER TABLE users ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE;
```

### 2.2 Frontend Auth with NextAuth

**Tasks:**
- [ ] Configure NextAuth with credentials provider
- [ ] Add Google OAuth provider
- [ ] Create login page (`/login`)
- [ ] Create registration page (`/register`)
- [ ] Create protected route wrapper
- [ ] Add session provider to layout
- [ ] Create auth API routes

### 2.3 Auth UI Components

**Tasks:**
- [ ] Login form with email/password
- [ ] Registration form with validation
- [ ] "Continue with Google" button
- [ ] Password strength indicator
- [ ] Error state handling
- [ ] Loading states

### 2.4 Session Management

**Tasks:**
- [ ] Store JWT in httpOnly cookie
- [ ] Implement token refresh logic
- [ ] Add logout functionality
- [ ] Create `useAuth` hook
- [ ] Protect dashboard routes

---

## Week 3: Backend Improvements

### 3.1 Database Integrity

**Tasks:**
- [ ] Add foreign key constraints:
  ```sql
  ALTER TABLE activities
      ADD CONSTRAINT fk_activities_user
      FOREIGN KEY (user_id) REFERENCES users(id);
  ```
- [ ] Add cascade delete rules
- [ ] Create missing indexes
- [ ] Validate existing data integrity

### 3.2 API Improvements

**Tasks:**
- [ ] Change `GET /users/new` to `POST /users`
- [ ] Add pagination to list endpoints:
  ```rust
  pub struct PaginationParams {
      pub limit: Option<i64>,
      pub offset: Option<i64>,
  }
  ```
- [ ] Return scores with activities
- [ ] Add `updated_at` timestamps
- [ ] Implement proper CORS configuration

### 3.3 Error Handling

**Tasks:**
- [ ] Replace `.unwrap()` with proper error handling in activity queue
- [ ] Add structured logging with request IDs
- [ ] Create consistent error response format
- [ ] Add validation for all inputs
- [ ] Log errors with context

### 3.4 Activity Upload Enhancement

**Tasks:**
- [ ] Validate GPX file structure before storing
- [ ] Extract start time from GPX
- [ ] Store track points in tracks table
- [ ] Add spatial index on tracks.geo
- [ ] Return scores in activity response

---

## Week 4: Infrastructure & Integration

### 4.1 CI/CD Pipeline

**Tasks:**
- [ ] Create GitHub Actions workflow:
  - Rust: `cargo fmt --check`, `cargo clippy`, `cargo test`
  - Node: `npm run lint`, `npm run build`
- [ ] Add database setup for integration tests
- [ ] Create Docker Compose for local dev
- [ ] Set up deployment workflow (staging)

### 4.2 Staging Environment

**Tasks:**
- [ ] Provision Fly.io application
- [ ] Set up PostgreSQL on Fly.io
- [ ] Configure environment variables
- [ ] Set up S3-compatible storage (Tigris)
- [ ] Deploy initial version
- [ ] Set up custom domain (staging.trackleader.app)

### 4.3 Frontend-Backend Integration

**Tasks:**
- [ ] Create API client module with TanStack Query
- [ ] Define TypeScript types matching Rust models
- [ ] Implement activity upload flow
- [ ] Implement activity list page
- [ ] Test complete user journey

### 4.4 Basic Pages

**Tasks:**
- [ ] Home page (marketing/landing for logged out)
- [ ] Dashboard (activity list for logged in)
- [ ] Activity upload component
- [ ] Basic activity detail page (placeholder)
- [ ] User settings page (placeholder)

---

## Deliverables

### End of Phase 1 Checklist

- [ ] Fresh Next.js frontend running
- [ ] User registration working
- [ ] User login working (email + Google)
- [ ] Protected routes implemented
- [ ] Activity upload functional
- [ ] Activity list displays
- [ ] Staging environment deployed
- [ ] CI/CD pipeline running
- [ ] Backend has proper error handling

### Technical Debt Addressed

- [ ] Foreign key constraints added
- [ ] Pagination on list endpoints
- [ ] Scores joined with activities
- [ ] Proper CORS configuration
- [ ] No `.unwrap()` in production code

---

## API Endpoints After Phase 1

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/register` | No | Register new user |
| POST | `/auth/login` | No | Login, returns JWT |
| POST | `/auth/logout` | Yes | Invalidate session |
| GET | `/auth/me` | Yes | Get current user |
| GET | `/users/{id}` | Yes | Get user profile |
| POST | `/activities` | Yes | Upload activity |
| GET | `/activities` | Yes | List user's activities |
| GET | `/activities/{id}` | Yes | Get activity with scores |
| GET | `/activities/{id}/download` | Yes | Download GPX |
| GET | `/health` | No | Health check |

---

## Database Schema After Phase 1

```sql
-- Updated users table
users (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    password_hash TEXT,
    auth_provider TEXT DEFAULT 'email',
    external_id TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
)

-- Activities with FK
activities (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    activity_type activity_type NOT NULL,
    name TEXT NOT NULL,
    object_store_path TEXT NOT NULL,
    started_at TIMESTAMPTZ,
    submitted_at TIMESTAMPTZ DEFAULT NOW()
)

-- Tracks table populated
tracks (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    activity_id UUID NOT NULL REFERENCES activities(id),
    geo GEOGRAPHY(LineString, 4326) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
)
-- With spatial index:
CREATE INDEX idx_tracks_geo ON tracks USING GIST (geo);

-- Scores with FK
scores (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    activity_id UUID NOT NULL REFERENCES activities(id),
    distance FLOAT NOT NULL,
    duration FLOAT NOT NULL,
    elevation_gain FLOAT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
)
```

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| OAuth configuration complexity | Start with credentials, add OAuth second |
| PostGIS on Fly.io | Use their managed Postgres with PostGIS |
| JWT security | Use short expiry, implement refresh tokens |
| Frontend/backend type drift | Generate types from Rust models |

---

## Success Criteria

1. **Authentication works:** Users can register, login, logout
2. **Upload works:** GPX files upload and process correctly
3. **List works:** Activities display with scores
4. **Deploy works:** Staging environment accessible
5. **CI/CD works:** PRs run tests automatically
