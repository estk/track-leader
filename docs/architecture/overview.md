# Architecture Overview

Track Leader is a web application for creating segments on trails and competing on leaderboards.

## System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Layer                              │
├─────────────────────────────────────────────────────────────────┤
│  Next.js Frontend (React)                                        │
│  - Server-side rendering                                         │
│  - Client-side interactivity                                     │
│  - MapLibre GL for maps                                          │
│  - Recharts for visualizations                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        API Layer                                 │
├─────────────────────────────────────────────────────────────────┤
│  Rust/Axum Backend                                               │
│  - REST API                                                      │
│  - JWT Authentication                                            │
│  - GPX Processing                                                │
│  - Segment Matching                                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Data Layer                                │
├─────────────────────────────────────────────────────────────────┤
│  PostgreSQL + PostGIS                                            │
│  - User data                                                     │
│  - Activities & segments                                         │
│  - Geospatial queries                                            │
│                                                                  │
│  File Storage                                                    │
│  - GPX files                                                     │
│  - Activity tracks                                               │
└─────────────────────────────────────────────────────────────────┘
```

## Frontend Architecture

### Technology Stack
- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS + shadcn/ui
- **Maps**: MapLibre GL JS
- **Charts**: Recharts
- **State**: React Context + Server Components

### Directory Structure
```
src/
├── app/                    # Next.js App Router pages
│   ├── (auth)/            # Auth group (login, register)
│   ├── activities/        # Activity pages
│   ├── segments/          # Segment pages
│   ├── feed/              # Social feed
│   └── leaderboards/      # Global leaderboards
├── components/
│   ├── ui/                # Base UI components (shadcn/ui)
│   ├── activity/          # Activity-specific components
│   ├── segments/          # Segment-specific components
│   ├── leaderboard/       # Leaderboard components
│   └── marketing/         # Landing page components
└── lib/
    ├── api.ts             # API client
    ├── auth-context.tsx   # Authentication context
    └── utils.ts           # Utility functions
```

### Key Patterns
- **Lazy Loading**: Heavy components (maps, charts) load on demand
- **Server Components**: Data fetching happens on the server
- **Loading States**: Skeleton UIs during data fetching
- **Error Boundaries**: Graceful error handling

## Backend Architecture

### Technology Stack
- **Framework**: Axum (Rust)
- **Database**: SQLx with PostgreSQL
- **Authentication**: JWT tokens
- **File Processing**: Custom GPX parser

### Module Structure
```
crates/tracks/src/
├── lib.rs              # Router and middleware setup
├── main.rs             # Application entry point
├── handlers.rs         # HTTP request handlers
├── auth.rs             # Authentication logic
├── models/             # Database models
├── activity_queue.rs   # Async activity processing
└── object_store_service.rs  # File storage
```

### Request Flow
1. Request arrives at Axum router
2. Middleware applies (CORS, compression, auth)
3. Handler processes request
4. Database queries via SQLx
5. Response with appropriate status

### Activity Processing
Upload flow:
1. GPX file received via multipart upload
2. File stored in object store
3. Background job parses GPX
4. Track points extracted and stored
5. Segment matching runs
6. Efforts recorded

## Database Schema

### Core Tables
- `users` - User accounts
- `activities` - Uploaded activities
- `activity_tracks` - GPS track data (PostGIS)
- `segments` - User-created segments
- `segment_tracks` - Segment GPS data (PostGIS)
- `segment_efforts` - Recorded attempts

### Social Tables
- `follows` - User follow relationships
- `kudos` - Activity likes
- `comments` - Activity comments
- `notifications` - User notifications

### Performance Indexes
Key indexes in `migrations/015_performance_indexes.sql`:
- User activity queries
- Leaderboard sorting
- Notification lookups

## Authentication

### JWT Flow
1. User registers/logs in
2. Server issues JWT token
3. Client stores token (localStorage)
4. Subsequent requests include `Authorization: Bearer {token}`
5. Server validates token on protected routes

### Protected Routes
Frontend uses `auth-context.tsx` to:
- Check authentication state
- Redirect unauthenticated users
- Provide user info to components

## Geospatial Features

### PostGIS Integration
- Track data stored as PostGIS LineString
- Segment matching uses ST_DWithin for proximity
- Distance calculations with ST_Length

### Map Rendering
- MapLibre GL renders vector tiles
- GeoJSON tracks displayed as layers
- OpenTopoMap tiles for terrain

## Performance Optimizations

### Frontend
- Route-level code splitting
- Lazy loading for heavy components
- Image optimization via Next.js
- Static asset caching (1 year)

### Backend
- Connection pooling (20 connections)
- Gzip compression
- Prepared SQL statements
- Database indexes for common queries

### CDN
- Cloudflare for static assets
- Edge caching for public API responses
- See `docs/architecture/cdn.md`

## Security

### Headers
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`

### CORS
- Development: Allow all origins
- Production: Restrict to frontend domain

### Input Validation
- Request payload validation
- File type verification
- Size limits on uploads

## Deployment

### Docker
- Multi-stage builds for small images
- Separate containers for frontend/backend
- PostgreSQL in container or managed service

### Environment Variables
| Variable | Description |
|----------|-------------|
| DATABASE_URL | PostgreSQL connection string |
| JWT_SECRET | Token signing key |
| OBJECT_STORE_PATH | File storage path |

See `docs/runbook.md` for operational procedures.
