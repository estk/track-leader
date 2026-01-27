# API Reference

## Current API (Rust Backend)

Base URL: `http://localhost:3000`

### Health Check

```http
GET /health
```

**Response:** `200 OK` (empty body)

---

### Users

#### Create User

```http
GET /users/new?name={name}&email={email}
```

**Note:** Should be POST, but currently implemented as GET.

**Query Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| name | string | Yes | User's display name |
| email | string | Yes | User's email (must be unique) |

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "name": "John Doe",
  "created_at": "2026-01-26T12:00:00Z"
}
```

**Errors:**
- `500` - Database error (e.g., duplicate email)

---

#### List Users

```http
GET /users
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "created_at": "2026-01-26T12:00:00Z"
  }
]
```

---

### Activities

#### Upload Activity

```http
POST /activities/new?user_id={uuid}&activity_type={type}&name={name}
Content-Type: multipart/form-data
```

**Query Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| user_id | UUID | Yes | User who owns this activity |
| activity_type | enum | Yes | Type of activity |
| name | string | Yes | Activity name/title |

**Activity Types:**
- `Walking`
- `Running`
- `Hiking`
- `RoadCycling`
- `MountainBiking`
- `Unknown`

**Form Data:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| file | file | Yes | GPX file |

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "activity_type": "Running",
  "name": "Morning Run",
  "object_store_path": "activities/550e8400.../550e8400...",
  "submitted_at": "2026-01-26T12:00:00Z"
}
```

**Errors:**
- `400` - No file provided
- `400` - Failed to process multipart data
- `500` - Failed to store file

**Note:** Scores are calculated asynchronously and not returned in this response.

---

#### Get Activity

```http
GET /activities/{id}
```

**Path Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| id | UUID | Activity ID |

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "activity_type": "Running",
  "name": "Morning Run",
  "object_store_path": "activities/550e8400.../550e8400...",
  "submitted_at": "2026-01-26T12:00:00Z"
}
```

**Errors:**
- `404` - Activity not found

**Note:** Scores are not included (should be joined).

---

#### Download GPX File

```http
GET /activities/{id}/download
```

**Path Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| id | UUID | Activity ID |

**Response:**
```
Content-Type: application/gpx+xml
Content-Disposition: attachment; filename="activity-name"
```

Body contains the original GPX file bytes.

**Errors:**
- `404` - Activity not found

---

#### Get User Activities

```http
GET /users/{id}/activities
```

**Path Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| id | UUID | User ID |

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "activity_type": "Running",
    "name": "Morning Run",
    "object_store_path": "activities/550e8400.../550e8400...",
    "submitted_at": "2026-01-26T12:00:00Z"
  }
]
```

**Note:** No pagination implemented.

---

## Proposed API Extensions

### Authentication

```http
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword",
  "name": "John Doe"
}
```

```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword"
}
```

Response includes JWT token or sets session cookie.

```http
POST /auth/logout
Authorization: Bearer {token}
```

```http
GET /auth/me
Authorization: Bearer {token}
```

### Activities (Enhanced)

```http
GET /activities/{id}
Authorization: Bearer {token}
```

Response includes scores:
```json
{
  "id": "...",
  "user_id": "...",
  "activity_type": "Running",
  "name": "Morning Run",
  "submitted_at": "2026-01-26T12:00:00Z",
  "scores": {
    "distance": 5234.5,
    "duration": 1845.0,
    "elevation_gain": 125.3
  },
  "segments": [
    {
      "segment_id": "...",
      "segment_name": "Hill Climb",
      "elapsed_time": 245.0,
      "rank": 15
    }
  ]
}
```

### Segments

```http
GET /segments
Authorization: Bearer {token}
```

Query parameters:
- `lat`, `lon`, `radius` - Search by location
- `activity_type` - Filter by activity type
- `starred` - Only starred segments
- `created_by` - Filter by creator

```http
POST /segments
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Summit Push",
  "description": "Final climb to the peak",
  "activity_type": "Hiking",
  "geo": {
    "type": "LineString",
    "coordinates": [[lon1, lat1], [lon2, lat2], ...]
  }
}
```

```http
GET /segments/{id}
```

```http
GET /segments/{id}/leaderboard
```

Query parameters:
- `scope` - `all_time`, `year`, `month`, `week`
- `gender` - `all`, `male`, `female`
- `age_group` - `all`, `18-24`, `25-34`, etc.
- `limit` - Number of results (default 10)

```http
POST /segments/{id}/star
DELETE /segments/{id}/star
```

### Trails

```http
GET /trails
POST /trails
GET /trails/{id}
GET /trails/{id}/leaderboard
POST /trails/{id}/star
DELETE /trails/{id}/star
```

### Leaderboards

```http
GET /leaderboards/segments/{segment_id}
GET /leaderboards/trails/{trail_id}
GET /leaderboards/global
```

### Social

```http
POST /users/{id}/follow
DELETE /users/{id}/follow
GET /users/{id}/followers
GET /users/{id}/following
```

```http
POST /activities/{id}/kudos
DELETE /activities/{id}/kudos
GET /activities/{id}/kudos
```

```http
POST /activities/{id}/comments
GET /activities/{id}/comments
DELETE /comments/{id}
```

### Metrics

```http
GET /metrics
POST /metrics
GET /activities/{id}/metrics
```

---

## Error Response Format

All errors should return consistent JSON:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Activity not found",
    "details": {}
  }
}
```

**Error Codes:**
| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Invalid input |
| `UNAUTHORIZED` | 401 | Missing/invalid auth |
| `FORBIDDEN` | 403 | No permission |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource already exists |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

---

## Pagination

All list endpoints should support:

```http
GET /activities?limit=20&offset=0
```

Response includes pagination metadata:

```json
{
  "data": [...],
  "pagination": {
    "total": 150,
    "limit": 20,
    "offset": 0,
    "has_more": true
  }
}
```

---

## Rate Limiting

Recommended limits:
- Anonymous: 60 requests/minute
- Authenticated: 300 requests/minute
- Uploads: 10 files/hour

Response headers:
```
X-RateLimit-Limit: 300
X-RateLimit-Remaining: 299
X-RateLimit-Reset: 1706284800
```

---

## CORS Configuration

Current: All origins allowed (development mode)

Production should restrict to:
- Frontend domain
- Mobile app origins (if applicable)
