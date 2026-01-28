# Track Leader

A GPS activity tracking application with segment competition features, similar to Strava.

## Features

- **Activity Upload** - Upload GPX files with activity type and visibility settings
- **Interactive Maps** - View activities on OpenTopoMap with contour lines and hill shading
- **Elevation Profiles** - Interactive charts with hover sync to map
- **Segments** - Create segments from activity portions, compete on leaderboards
- **Personal Records** - Track PRs on segments automatically
- **Segment Discovery** - Search, filter, and explore segments on a map

## Tech Stack

- **Frontend:** Next.js 14, React, TypeScript, Tailwind CSS, Leaflet, Recharts
- **Backend:** Rust, Axum, SQLx
- **Database:** PostgreSQL 15 with PostGIS
- **Auth:** JWT with argon2 password hashing

## Development

### Manual Start 

If you prefer to run components separately:

```bash
# Terminal 1 - Database
cd crates/tracks
docker-compose up postgres

# Terminal 2 - Backend
cd crates/tracks
RUST_LOG=info DATABASE_URL="postgres://tracks_user:tracks_password@localhost:5432/tracks_db" cargo run

# Terminal 3 - Frontend
npm run dev
```

## Project Structure

```
track-leader/
├── src/                    # Next.js frontend
│   ├── app/               # App router pages
│   ├── components/        # React components
│   └── lib/               # Utilities, API client
├── crates/tracks/         # Rust backend
│   ├── src/               # Backend source
│   ├── migrations/        # SQL migrations
│   └── uploads/           # GPX file storage
├── scripts/               # Development scripts
├── logs/                  # Development logs (gitignored)
└── docs/                  # Documentation
```

## Documentation

- [API Reference](docs/api-reference.md) - REST API documentation
- [Architecture](docs/architecture/overview.md) - System design
- [Development](docs/ai/development.md) - Development environment setup
