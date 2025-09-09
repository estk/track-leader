# Tracks Service

A Rust service built with Axum for processing GPS activity files, storing them using object_store, and saving activity metadata in PostgreSQL.

## Features

- **GPX File Processing**: Parse GPX files and calculate activity metrics
- **Activity Metrics**: Calculate total distance, elevation gain/loss, and duration
- **Object Store Integration**: Store original files using the object_store crate
- **Database Storage**: Store activity metadata in PostgreSQL
- **REST API**: Upload files, retrieve activity data, and download original files
- **Multi-format Support**: Extensible design for multiple GPS file formats (GPX implemented)

## API Endpoints

### Upload GPX File
```
POST /activities/upload?user_id={uuid}&activity_type={type}
Content-Type: multipart/form-data

Form field: file (GPX file)
```

Activity types: `running`, `cycling`, `walking`, `hiking`, `other`

### Get Activity
```
GET /activities/{id}
```

### Get User Activities
```
GET /activities?user_id={uuid}
```

### Download Original GPX File
```
GET /activities/{id}/download
```

### Health Check
```
GET /health
```

## Database Schema

### Activities Table
- `id`: UUID (Primary Key)
- `user_id`: UUID
- `activity_type`: Enum (running, cycling, walking, hiking, other)
- `filename`: Text
- `object_store_path`: Text (path to original file in object store)
- `total_distance`: Double precision (meters)
- `total_ascent`: Double precision (meters)
- `total_descent`: Double precision (meters)
- `total_time`: BigInt (seconds)
- `submitted_at`: Timestamp with timezone
- `created_at`: Timestamp with timezone
- `updated_at`: Timestamp with timezone

### Track Points Table
- `id`: UUID (Primary Key)
- `activity_id`: UUID (Foreign Key to activities)
- `latitude`: Double precision
- `longitude`: Double precision
- `elevation`: Optional double precision
- `time`: Optional timestamp with timezone
- `sequence`: Integer

Note: Original GPX files are stored in the object store rather than as individual track point rows for better performance and storage efficiency.

## Setup

### Option 1: Docker Compose (Recommended)

1. Build and start the services:
   ```bash
   docker-compose up --build
   ```

2. The service will be available at `http://localhost:3000`

3. To stop the services:
   ```bash
   docker-compose down
   ```

### Option 2: Manual Setup

1. Set up PostgreSQL databasea: `docker run --name pg -e POSTGRES_PASSWORD=pg -p 5432:5432 -d postgres`
2. Set environment variable: `DATABASE_URL=postgres://postgres:pg@localhost:5432`
3. `cargo sqlx migrate run && cargo sqlx prepare`
3. Run the service: `cargo run --bin tracks`

The service will automatically run database migrations on startup.

### Docker Details

The Docker setup includes:
- **PostgreSQL 15** with health checks
- **Tracks Service** built from source
- **Persistent volumes** for database and file uploads
- **Automatic migration** on service startup

## Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (default: `postgres://postgres:password@localhost/tracks`)
- `OBJECT_STORE_PATH`: Local file system path for storing GPX files (default: `./uploads`)
- `PORT`: Server port (default: `3000`)

## Example Usage

```bash
# Upload a GPX file
curl -X POST \
  "http://localhost:3000/activities/upload?user_id=123e4567-e89b-12d3-a456-426614174000&activity_type=running" \
  -F "file=@activity.gpx"

# Get activity by ID
curl http://localhost:3000/activities/123e4567-e89b-12d3-a456-426614174000

# Get user activities
curl "http://localhost:3000/activities?user_id=123e4567-e89b-12d3-a456-426614174000"

# Download original GPX file
curl http://localhost:3000/activities/123e4567-e89b-12d3-a456-426614174000/download -o activity.gpx
```
