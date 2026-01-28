# API Reference

Track Leader REST API documentation.

## Base URL

- Development: `http://localhost:3001`
- Production: `https://api.trackleader.com`

## Authentication

Most endpoints require authentication via JWT token.

### Register

```http
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword",
  "name": "John Doe"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe"
  }
}
```

### Login

```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe"
  }
}
```

### Get Current User

```http
GET /auth/me
Authorization: Bearer {token}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "name": "John Doe"
}
```

---

## Health & Stats

### Health Check

```http
GET /health
```

**Response:** `200 OK`

### Platform Stats

```http
GET /stats
```

**Response:**
```json
{
  "active_users": 1234,
  "segments_created": 567,
  "activities_uploaded": 8901
}
```

---

## Activities

### Upload Activity

```http
POST /activities/new
Authorization: Bearer {token}
Content-Type: multipart/form-data

user_id: {uuid}
activity_type: Running
name: Morning Run
file: [GPX file]
```

**Activity Types:**
- `Walking`
- `Running`
- `Hiking`
- `RoadCycling`
- `MountainBiking`
- `Unknown`

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "activity_type": "Running",
  "name": "Morning Run",
  "submitted_at": "2026-01-26T12:00:00Z"
}
```

### Get Activity

```http
GET /activities/{id}
Authorization: Bearer {token}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "activity_type": "Running",
  "name": "Morning Run",
  "submitted_at": "2026-01-26T12:00:00Z",
  "scores": {
    "distance": 5234.5,
    "duration": 1845.0,
    "elevation_gain": 125.3
  }
}
```

### Delete Activity

```http
DELETE /activities/{id}
Authorization: Bearer {token}
```

**Response:** `204 No Content`

### Get Activity Track

```http
GET /activities/{id}/track
```

**Response:**
```json
{
  "type": "LineString",
  "coordinates": [[lon1, lat1, ele1], [lon2, lat2, ele2], ...]
}
```

### Get Activity Segments

```http
GET /activities/{id}/segments
```

Returns all segment efforts recorded for this activity.

### Download GPX File

```http
GET /activities/{id}/download
```

**Response:**
```
Content-Type: application/gpx+xml
Content-Disposition: attachment; filename="activity.gpx"
```

### Get User Activities

```http
GET /users/{id}/activities
Authorization: Bearer {token}
```

---

## Segments

### List Segments

```http
GET /segments
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| activity_type | string | Filter by activity type |
| limit | number | Results per page (default 50) |
| offset | number | Pagination offset |

### Get Nearby Segments

```http
GET /segments/nearby?lat={lat}&lon={lon}&radius={meters}
```

### Create Segment

```http
POST /segments
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Summit Push",
  "description": "Final climb to the peak",
  "activity_type": "Hiking",
  "activity_id": "550e8400-e29b-41d4-a716-446655440001",
  "start_index": 100,
  "end_index": 250
}
```

### Get Segment

```http
GET /segments/{id}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "name": "Summit Push",
  "description": "Final climb to the peak",
  "activity_type": "Hiking",
  "distance": 1234.5,
  "elevation_gain": 150.0,
  "created_by": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2026-01-26T12:00:00Z"
}
```

### Get Segment Track

```http
GET /segments/{id}/track
```

### Get Segment Leaderboard

```http
GET /segments/{id}/leaderboard
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| gender | string | Filter: `male`, `female`, `all` |
| age_group | string | Filter: `18-29`, `30-39`, etc. |
| limit | number | Results per page |

**Response:**
```json
{
  "entries": [
    {
      "rank": 1,
      "user_id": "...",
      "user_name": "Jane Doe",
      "elapsed_time": 245.5,
      "recorded_at": "2026-01-26T12:00:00Z",
      "activity_id": "..."
    }
  ],
  "kom": {...},
  "qom": {...}
}
```

### Star/Unstar Segment

```http
POST /segments/{id}/star
DELETE /segments/{id}/star
Authorization: Bearer {token}
```

### Get Starred Segments

```http
GET /segments/starred
Authorization: Bearer {token}
```

### Get My Efforts on Segment

```http
GET /segments/{id}/my-efforts
Authorization: Bearer {token}
```

### Get Segment Achievements

```http
GET /segments/{id}/achievements
```

---

## Leaderboards

### Crown Leaderboard

```http
GET /leaderboards/crowns
```

Returns users ranked by total crowns (KOMs + QOMs).

### Distance Leaderboard

```http
GET /leaderboards/distance
```

Returns users ranked by total distance.

---

## User Profiles

### Get User Profile

```http
GET /users/{id}/profile
```

### Update Profile

```http
PUT /users/me/profile
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "John Doe",
  "gender": "male",
  "birth_year": 1990
}
```

### Get User Achievements

```http
GET /users/{id}/achievements
GET /users/me/achievements
Authorization: Bearer {token}
```

---

## Social Features

### Follow/Unfollow User

```http
POST /users/{id}/follow
DELETE /users/{id}/follow
Authorization: Bearer {token}
```

### Get Followers/Following

```http
GET /users/{id}/followers
GET /users/{id}/following
```

### Activity Feed

```http
GET /feed
Authorization: Bearer {token}
```

Returns activities from followed users.

### Kudos

```http
POST /activities/{id}/kudos
DELETE /activities/{id}/kudos
GET /activities/{id}/kudos/givers
Authorization: Bearer {token}
```

### Comments

```http
GET /activities/{id}/comments
POST /activities/{id}/comments
Authorization: Bearer {token}
Content-Type: application/json

{
  "body": "Great run!"
}
```

```http
DELETE /comments/{id}
Authorization: Bearer {token}
```

---

## Notifications

### Get Notifications

```http
GET /notifications
Authorization: Bearer {token}
```

### Mark as Read

```http
POST /notifications/{id}/read
POST /notifications/read-all
Authorization: Bearer {token}
```

---

## Teams

Teams enable group-based sharing for activities and segments.

### Create Team

```http
POST /teams
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Trail Runners Club",
  "description": "Local trail running group",
  "visibility": "private",
  "join_policy": "invitation"
}
```

**Visibility:** `public` (discoverable) or `private`
**Join Policy:** `open`, `request`, or `invitation`

### List My Teams

```http
GET /teams
Authorization: Bearer {token}
```

### Discover Teams

```http
GET /teams/discover
```

Returns teams with `visibility: public`.

### Get Team

```http
GET /teams/{id}
Authorization: Bearer {token}
```

### Update Team

```http
PATCH /teams/{id}
Authorization: Bearer {token}
```

Requires `admin` or `owner` role.

### Delete Team

```http
DELETE /teams/{id}
Authorization: Bearer {token}
```

Requires `owner` role.

### Team Members

```http
GET /teams/{id}/members
DELETE /teams/{id}/members/{user_id}
PATCH /teams/{id}/members/{user_id}
Authorization: Bearer {token}
```

### Join/Leave Team

```http
POST /teams/{id}/join
POST /teams/{id}/leave
Authorization: Bearer {token}
```

Join behavior depends on team's `join_policy`.

### Team Invitations

```http
POST /teams/{id}/invitations
GET /teams/{id}/invitations
DELETE /teams/{id}/invitations/{invitation_id}
Authorization: Bearer {token}
```

```http
GET /invitations/{token}
POST /invitations/{token}/accept
Authorization: Bearer {token}
```

### Join Requests

```http
GET /teams/{id}/join-requests
POST /teams/{id}/join-requests/{request_id}
Authorization: Bearer {token}
```

### Activity-Team Sharing

```http
GET /activities/{id}/teams
POST /activities/{id}/teams
DELETE /activities/{id}/teams/{team_id}
Authorization: Bearer {token}
```

### Segment-Team Sharing

```http
GET /segments/{id}/teams
POST /segments/{id}/teams
DELETE /segments/{id}/teams/{team_id}
Authorization: Bearer {token}
```

### Team Content

```http
GET /teams/{id}/activities
GET /teams/{id}/segments
Authorization: Bearer {token}
```

---

## Error Responses

All errors return consistent JSON:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Activity not found"
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

## Rate Limiting

| Tier | Limit |
|------|-------|
| Anonymous | 60 requests/minute |
| Authenticated | 300 requests/minute |
| Uploads | 10 files/hour |

Response headers:
```
X-RateLimit-Limit: 300
X-RateLimit-Remaining: 299
X-RateLimit-Reset: 1706284800
```

---

## Pagination

List endpoints support pagination:

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
