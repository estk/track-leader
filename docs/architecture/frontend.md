# Frontend Architecture

## Current Status: Non-Functional

The existing frontend **cannot run**. It imports modules (`@/lib/database`, `@/lib/gpx-parser`) that do not exist. This document serves as both an autopsy of the current state and a reference for the redesign.

## Current (Broken) Architecture

### Technology Stack

| Technology | Version | Status |
|------------|---------|--------|
| Next.js | 14.0 | Installed, App Router |
| React | 18.0 | Installed |
| Tailwind CSS | 3.3.6 | Configured |
| Leaflet | 1.9.4 | Installed |
| react-leaflet | 4.2.1 | Installed |
| sqlite3 | 5.1.6 | Installed but unused |
| xml2js | 0.6.2 | Installed |
| @heroicons/react | 2.2.0 | Installed |
| TypeScript | 5.0 | Configured |

### File Structure

```
track-leader/
├── app/
│   ├── api/
│   │   └── tracks/
│   │       ├── route.ts       # GET/POST /api/tracks (broken)
│   │       └── [id]/
│   │           └── route.ts   # GET /api/tracks/:id (broken)
│   ├── tracks/
│   │   └── [id]/
│   │       └── page.tsx       # Track detail page
│   ├── layout.tsx             # Root layout
│   ├── page.tsx               # Home page
│   └── globals.css            # Global styles
├── components/
│   ├── TrackList.tsx          # Activity list table
│   ├── TrackUpload.tsx        # File upload UI
│   ├── TrackMap.tsx           # Leaflet map
│   └── TrackDetail.tsx        # Activity detail view
├── package.json
├── tailwind.config.js
├── tsconfig.json
└── next.config.js
```

### Why It's Broken

1. **Missing lib modules:**
   ```typescript
   // These files DO NOT EXIST:
   import { db } from '@/lib/database'
   import { parseGPX } from '@/lib/gpx-parser'
   import { Track } from '@/lib/database'
   ```

2. **Wrong API architecture:**
   - Frontend expects SQLite-based local API routes
   - Backend is Rust service on separate port
   - No proxy configuration to bridge them

3. **Conflicting data sources:**
   - `tracks.db` (SQLite file) exists in project root
   - Backend uses PostgreSQL
   - Frontend has `sqlite3` in dependencies

### Component Analysis

#### TrackUpload.tsx

**What works:**
- Drag-and-drop file handling
- File type validation (.gpx only)
- Loading state management

**Issues:**
- Posts to `/api/tracks` (doesn't exist/wrong endpoint)
- Uses `alert()` for success/error feedback
- Icon sizing broken: `<CloudArrowUpIcon width={25} className="h-1 w-1" />`

#### TrackList.tsx

**What works:**
- Clean table layout
- Formatting helpers (distance, duration, date)
- Loading skeleton animation
- Empty state handling

**Issues:**
- No pagination
- No sorting controls
- No filtering
- Click handler assumes `track.id` is integer (backend uses UUID)

#### TrackMap.tsx

**What works:**
- Basic Leaflet integration
- Polyline rendering
- Empty state handling

**Issues:**
- No dynamic bounds fitting
- Static zoom level (13)
- No start/end markers
- No elevation profile
- Point count shown but not useful

#### TrackDetail.tsx

**What works:**
- Dynamic import for Leaflet (SSR avoidance)
- Stats grid layout
- Additional statistics section

**Issues:**
- Back button icon sizing: `className="w-44 h-44"` (massive)
- Naive calorie calculation: `distance/1000 * 65`
- No actual performance metrics
- No segment information

---

## Recommended New Architecture

### Option A: Next.js App Router + Server Actions (Recommended)

```
┌─────────────────────────────────────────────────────────┐
│                    Next.js Frontend                      │
├─────────────────────────────────────────────────────────┤
│  app/                                                    │
│  ├── (auth)/            # Auth routes                   │
│  │   ├── login/                                         │
│  │   └── register/                                      │
│  ├── (dashboard)/       # Protected routes              │
│  │   ├── activities/                                    │
│  │   ├── segments/                                      │
│  │   └── leaderboards/                                  │
│  ├── api/               # API routes (proxy to Rust)    │
│  └── actions/           # Server actions                │
├─────────────────────────────────────────────────────────┤
│                         ▼                               │
│              HTTP calls to Rust backend                 │
└─────────────────────────────────────────────────────────┘
```

**Pros:**
- Keep existing Tailwind/Leaflet setup
- Server components for initial load
- Server actions for mutations
- Built-in auth options (NextAuth)

**Cons:**
- Two Node.js processes (Next.js + potential API routes)
- Complexity of server/client boundary

### Option B: SvelteKit

```
┌─────────────────────────────────────────────────────────┐
│                   SvelteKit Frontend                     │
├─────────────────────────────────────────────────────────┤
│  src/                                                    │
│  ├── routes/                                            │
│  │   ├── (auth)/                                        │
│  │   ├── activities/                                    │
│  │   ├── segments/                                      │
│  │   └── leaderboards/                                  │
│  ├── lib/               # Shared utilities              │
│  └── components/        # Svelte components             │
├─────────────────────────────────────────────────────────┤
│                         ▼                               │
│              HTTP calls to Rust backend                 │
└─────────────────────────────────────────────────────────┘
```

**Pros:**
- Smaller bundle size
- Better performance
- Simpler mental model
- Built-in transitions

**Cons:**
- Rewrite from scratch
- Team needs to learn Svelte
- Fewer component libraries

### Option C: HTMX + Server-Rendered (from Rust)

```
┌─────────────────────────────────────────────────────────┐
│              Rust Backend (Axum + Templates)             │
├─────────────────────────────────────────────────────────┤
│  templates/                                              │
│  ├── base.html                                          │
│  ├── activities/                                        │
│  │   ├── list.html                                      │
│  │   └── detail.html                                    │
│  ├── segments/                                          │
│  └── leaderboards/                                      │
├─────────────────────────────────────────────────────────┤
│  HTMX for interactivity                                 │
│  Alpine.js for client-side state                        │
│  Tailwind CSS for styling                               │
└─────────────────────────────────────────────────────────┘
```

**Pros:**
- Single server process
- Fast initial load
- Simple deployment
- No JavaScript framework

**Cons:**
- Complex map interactivity harder
- Less rich UX possible
- Template debugging harder

---

## Recommended Technology Stack (Next.js Approach)

| Category | Technology | Rationale |
|----------|------------|-----------|
| Framework | Next.js 14 | Already installed, App Router mature |
| Styling | Tailwind CSS 4 | Already configured, upgrade to v4 |
| Maps | MapLibre GL | Better than Leaflet for complex interactions |
| Charts | Recharts | Elevation profiles, performance graphs |
| State | Zustand | Simple, TypeScript-friendly |
| Forms | React Hook Form + Zod | Type-safe validation |
| Auth | NextAuth.js v5 | OAuth + credentials support |
| API Client | TanStack Query | Caching, revalidation |
| Icons | Lucide React | Consistent, well-maintained |
| UI Components | shadcn/ui | Copy-paste, customizable |

---

## Key Pages to Build

### 1. Dashboard (/)
- Recent activities feed
- Personal stats summary
- Trending segments
- Leaderboard highlights

### 2. Activities (/activities)
- Activity list with filters
- Calendar view option
- Bulk actions
- Export functionality

### 3. Activity Detail (/activities/[id])
- Interactive map with route
- Elevation profile
- Splits table
- Matched segments
- Share/download options

### 4. Segments (/segments)
- Segment browser/search
- Map-based discovery
- Popularity rankings
- Create segment flow

### 5. Segment Detail (/segments/[id])
- Segment map visualization
- Leaderboard table
- Personal efforts
- Segment stats

### 6. Leaderboards (/leaderboards)
- Global/regional filters
- Time period filters
- Demographic filters
- Multi-segment rankings

### 7. Profile (/profile/[username])
- User stats
- Activity history
- Achievements/badges
- Following/followers

### 8. Settings (/settings)
- Account management
- Privacy controls
- Notification preferences
- Connected services

---

## Design System Requirements

### Visual Identity
- **Primary color:** Trail green (#2D5A27 or similar)
- **Accent color:** Summit orange (#E67E22)
- **Background:** Off-white with subtle topographic pattern
- **Typography:** Clean sans-serif (Inter or similar)

### Interaction Patterns
- Smooth page transitions
- Skeleton loading states
- Optimistic updates
- Pull-to-refresh on mobile
- Infinite scroll for feeds

### Mobile-First
- Touch-friendly controls
- Bottom navigation on mobile
- Swipe gestures for map
- Offline capability for viewing

### Accessibility
- WCAG 2.1 AA compliance
- Keyboard navigation
- Screen reader support
- High contrast mode
- Reduced motion support

---

## Migration Path

1. **Delete current frontend** (except `components/` for reference)
2. **Initialize fresh Next.js 14** with new structure
3. **Set up design system** with Tailwind + shadcn/ui
4. **Build authentication** with NextAuth
5. **Implement API client** connecting to Rust backend
6. **Port and improve components** one by one
7. **Add new features** (segments, leaderboards)
