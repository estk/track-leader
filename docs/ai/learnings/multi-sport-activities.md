# Multi-Sport Activities Implementation

## Overview

Multi-sport activities allow a single GPX file to contain multiple activity types (e.g., Ride+Hike, Run+Walk). This enables tracking activities like bike-to-hike adventures, duathlons, or any activity that transitions between types.

## Architecture

### Activity Types System

Replaced the PostgreSQL enum with a UUID-based table system:

```
activity_types (table)
├── id: UUID (fixed UUIDs for built-in types)
├── name: TEXT (canonical short name: "run", "mtb", "road")
├── is_builtin: BOOLEAN
├── created_by: UUID (NULL for built-ins)
└── created_at: TIMESTAMPTZ

activity_aliases (table)
├── id: UUID
├── alias: TEXT ("biking", "cycling", "running")
├── activity_type_id: UUID → activity_types(id)
└── created_at: TIMESTAMPTZ
```

**Built-in Types:**
| UUID | Name | Description |
|------|------|-------------|
| `00000000-0000-0000-0000-000000000001` | walk | Walking |
| `00000000-0000-0000-0000-000000000002` | run | Running |
| `00000000-0000-0000-0000-000000000003` | hike | Hiking |
| `00000000-0000-0000-0000-000000000004` | road | Road cycling |
| `00000000-0000-0000-0000-000000000005` | mtb | Mountain biking |
| `00000000-0000-0000-0000-000000000006` | emtb | E-mountain biking |
| `00000000-0000-0000-0000-000000000007` | gravel | Gravel cycling |
| `00000000-0000-0000-0000-000000000008` | unknown | Unknown/other |

### Alias Resolution

Aliases can map to multiple types (1:many):
- `"biking"` → road, mtb, emtb, gravel (user picks)
- `"running"` → run (exact match)

Resolution logic returns:
- `Exact(Uuid)` - single match, use directly
- `Ambiguous(Vec<Uuid>)` - multiple matches, UI shows picker
- `NotFound` - no match

### Multi-Sport Storage

Activities table has two new columns:

```sql
type_boundaries TIMESTAMPTZ[]  -- ['2024-01-15T10:00:00Z', '2024-01-15T10:30:00Z', '2024-01-15T11:00:00Z']
segment_types UUID[]           -- [uuid1, uuid2] references activity_types(id)
```

**Invariant:** `length(segment_types) = length(type_boundaries) - 1`

For a ride→hike→ride activity:
```
type_boundaries: [start_time, hike_start, hike_end, end_time]
segment_types:   [mtb_id,     hike_id,    mtb_id]
```

Single-sport activities use `NULL` for both arrays and rely on `activity_type_id`.

## Segment Matching for Multi-Sport

When processing a multi-sport activity, segment matching must respect activity type boundaries:

1. **Find all geometric matches** - Segments where the track passes through start and end points
2. **For each match, determine position on track** - Calculate the midpoint fraction
3. **Convert fraction to timestamp** - Interpolate based on track point timestamps
4. **Look up activity type at timestamp** - Find which boundary window contains the timestamp
5. **Filter by type match** - Only keep segments where types match

```rust
// Helper functions in activity_queue.rs

fn fraction_to_timestamp(track_points: &[TrackPointData], fraction: f64) -> Option<OffsetDateTime>
fn get_activity_type_at_timestamp(
    type_boundaries: &[OffsetDateTime],
    segment_types: &[Uuid],
    timestamp: OffsetDateTime,
) -> Option<Uuid>
```

## API Endpoints

### Activity Types

```
GET /activity-types          - List all types (built-in + custom)
POST /activity-types         - Create custom type
GET /activity-types/resolve  - Resolve name/alias to type(s)
GET /activity-types/{id}     - Get type details
```

### Upload with Multi-Sport

```
POST /activities/new
Query params:
  - activity_type_id: UUID (required, primary type)
  - name, visibility, team_ids
  - type_boundaries: ["2024-01-15T10:00:00Z", ...] (optional)
  - segment_types: ["uuid1", "uuid2", ...] (optional)
```

## Frontend Integration

### GPX Parser (`src/lib/gpx-parser.ts`)

Client-side GPX parsing for preview and multi-sport editing:

```typescript
parseGpxFile(file: File): Promise<ParsedGpx>
parseGpxString(xml: string): ParsedGpx
findPointIndexByTimestamp(points: TrackPoint[], timestamp: string): number
getTimestampAtIndex(points: TrackPoint[], index: number): string | null
getTrackTimeRange(points: TrackPoint[]): { start: string | null, end: string | null }
```

### Elevation Profile Multi-Range Mode

The `ElevationProfile` component (`src/components/activity/elevation-profile.tsx`) supports:

```typescript
interface MultiRangeSegment {
  startIndex: number;
  endIndex: number;
  activityTypeId: string;
}

// Props
multiRangeMode?: boolean;       // Enable boundary selection mode
segments?: MultiRangeSegment[]; // Colored segments to display
onBoundaryClick?: (index: number) => void;  // Click handler for adding boundaries
selectedBoundaryIndex?: number; // Visual feedback for selected boundary
```

**Activity Type Colors:**
| Type | Color |
|------|-------|
| Walk | Green (#22c55e) |
| Run | Red (#ef4444) |
| Hike | Lime (#84cc16) |
| Road | Blue (#3b82f6) |
| MTB | Orange (#f97316) |
| E-MTB | Yellow (#eab308) |
| Gravel | Purple (#a855f7) |
| Unknown | Gray (#6b7280) |

### Upload Page Multi-Sport Editor

The upload page (`src/app/activities/upload/page.tsx`) provides:

1. **GPX Preview** - Parses file client-side, shows elevation profile
2. **Multi-sport Toggle** - Checkbox to enable (requires timestamps in GPX)
3. **Boundary Selection** - Click on chart to add/remove segment boundaries
4. **Type Selectors** - Dropdown for each segment with colored indicator
5. **Boundary Management** - Trash button to remove boundaries

**State Flow:**
```
User selects GPX → Parse & preview → Enable multi-sport → Click to add boundaries
                                                       → Select types per segment
                                                       → Submit with type_boundaries & segment_types
```

**Invariant Maintained:** `length(segmentTypes) = length(typeBoundaries) - 1`

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Boundary system | Timestamps | Matches GPX data, intuitive for users |
| Storage | Arrays on activities table | Compact, no extra joins |
| Type names | Short canonical names | "mtb" not "mountain_biking" |
| Aliases | Separate table with 1:many | Enables disambiguation UI |
| Built-in UUIDs | Fixed values | Consistent across environments |

## Migration Strategy

The migration was split for safety:
1. **009_activity_types.sql** - Creates type system, migrates data, adds new columns
2. **010_drop_activity_type_enum.sql** - Drops legacy enum after verification

This allows rollback if issues are found after the first migration.

## Rust Type Reference

```rust
// models.rs

pub mod builtin_types {
    use uuid::Uuid;
    pub const WALK: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000001);
    pub const RUN: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000002);
    pub const HIKE: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000003);
    pub const ROAD: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000004);
    pub const MTB: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000005);
    pub const EMTB: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000006);
    pub const GRAVEL: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000007);
    pub const UNKNOWN: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000008);
}

pub struct ActivityTypeRow {
    pub id: Uuid,
    pub name: String,
    pub is_builtin: bool,
    pub created_by: Option<Uuid>,
}

pub enum ResolvedActivityType {
    Exact(Uuid),
    Ambiguous(Vec<Uuid>),
    NotFound,
}
```

## Testing Considerations

- Verify single-sport activities still work (NULL boundaries)
- Test segment matching at type boundaries (edge cases)
- Test alias resolution with ambiguous aliases
- Verify custom type creation and usage
