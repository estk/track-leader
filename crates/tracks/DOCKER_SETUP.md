# Docker Setup for Tracks Service

## Prerequisites

1. Docker and Docker Compose must be installed
2. Docker daemon must be running

## Quick Start

1. Start the services:
   ```bash
   docker-compose up --build
   ```

2. The service will be available at `http://localhost:3000`

3. To run in detached mode:
   ```bash
   docker-compose up -d --build
   ```

4. To stop the services:
   ```bash
   docker-compose down
   ```

## Service Details

### PostgreSQL Service
- **Container**: `tracks_postgres`
- **Port**: `5432:5432`
- **Database**: `tracks_db`
- **User**: `tracks_user`
- **Password**: `tracks_password`
- **Health Check**: Ensures database is ready before starting the tracks service

### Tracks Service
- **Container**: `tracks_service`
- **Port**: `3000:3000`
- **Build Context**: Current directory
- **Dependencies**: Waits for PostgreSQL to be healthy

## Persistent Storage

The setup includes two Docker volumes:
- `postgres_data`: PostgreSQL database files
- `uploads_data`: GPX file uploads

## Environment Variables

The following environment variables are configured in docker-compose.yml:

- `DATABASE_URL`: PostgreSQL connection string
- `OBJECT_STORE_PATH`: Path for storing uploaded files (`/app/uploads`)
- `PORT`: Service port (`3000`)
- `RUST_LOG`: Log level (`info`)

## Testing the Setup

1. Check service health:
   ```bash
   curl http://localhost:3000/health
   ```

2. Upload a GPX file:
   ```bash
   curl -X POST \
     "http://localhost:3000/activities/upload?user_id=123e4567-e89b-12d3-a456-426614174000&activity_type=running" \
     -F "file=@your-activity.gpx"
   ```

## Troubleshooting

### Docker Daemon Not Running
If you see "Cannot connect to the Docker daemon", start Docker Desktop or run:
```bash
sudo systemctl start docker  # Linux
```

### Port Already in Use
If port 3000 or 5432 is in use, modify the ports in `docker-compose.yml`:
```yaml
ports:
  - "3001:3000"  # Use port 3001 instead
```

### Database Connection Issues
Check PostgreSQL logs:
```bash
docker-compose logs postgres
```

Check service logs:
```bash
docker-compose logs tracks-service
```

## Development

To rebuild only the tracks service:
```bash
docker-compose build tracks-service
docker-compose up tracks-service
```

To view logs in real-time:
```bash
docker-compose logs -f tracks-service
```