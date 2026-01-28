# Deployment Guide

This guide covers deploying Track Leader to production.

## Prerequisites

- Docker and Docker Compose
- Domain name with DNS access
- SSL certificates (or Let's Encrypt)

## Environment Setup

### Required Environment Variables

Create a `.env.prod` file:

```bash
# Database
POSTGRES_DB=tracks_db
POSTGRES_USER=tracks_user
POSTGRES_PASSWORD=<secure-random-password>

# Backend
DATABASE_URL=postgres://tracks_user:<password>@postgres:5432/tracks_db
JWT_SECRET=<secure-random-string>
OBJECT_STORE_PATH=/app/uploads
RUST_LOG=info

# Frontend
NEXT_PUBLIC_API_URL=https://api.yourdomain.com

# Optional
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
```

### Generate Secrets

```bash
# Generate JWT secret
openssl rand -base64 32

# Generate database password
openssl rand -base64 24
```

## Docker Deployment

### Build Images

```bash
# Build all images
docker-compose -f docker-compose.prod.yml build
```

### Start Services

```bash
# Start all services
docker-compose -f docker-compose.prod.yml up -d

# Check status
docker-compose -f docker-compose.prod.yml ps
```

### Run Migrations

```bash
# Apply database migrations
docker exec tracks_backend_prod /app/tracks migrate
```

### Verify Deployment

```bash
# Check backend health
curl http://localhost:3001/health

# Check frontend
curl http://localhost:3000
```

## Reverse Proxy Setup

### Nginx Configuration

Create `nginx/nginx.conf`:

```nginx
upstream frontend {
    server frontend:3000;
}

upstream backend {
    server backend:3001;
}

server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    ssl_certificate /etc/nginx/ssl/fullchain.pem;
    ssl_certificate_key /etc/nginx/ssl/privkey.pem;

    # Frontend
    location / {
        proxy_pass http://frontend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}

server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    ssl_certificate /etc/nginx/ssl/fullchain.pem;
    ssl_certificate_key /etc/nginx/ssl/privkey.pem;

    # Backend API
    location / {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # File upload size
        client_max_body_size 50M;
    }
}
```

### Enable Nginx

```bash
docker-compose -f docker-compose.prod.yml --profile with-nginx up -d
```

## SSL Certificates

### Let's Encrypt with Certbot

```bash
# Install certbot
apt install certbot

# Obtain certificate
certbot certonly --webroot -w /var/www/certbot \
  -d yourdomain.com \
  -d api.yourdomain.com

# Copy to nginx ssl directory
cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem nginx/ssl/
cp /etc/letsencrypt/live/yourdomain.com/privkey.pem nginx/ssl/
```

### Auto-Renewal

Add to crontab:
```bash
0 0 1 * * certbot renew --quiet && docker-compose -f docker-compose.prod.yml restart nginx
```

## Cloud Deployments

### AWS

#### EC2 Instance
1. Launch EC2 instance (t3.medium recommended)
2. Install Docker and Docker Compose
3. Clone repository
4. Configure environment
5. Run docker-compose

#### RDS for PostgreSQL
1. Create RDS PostgreSQL instance with PostGIS
2. Update DATABASE_URL to point to RDS endpoint
3. Ensure security group allows connection from EC2

### Google Cloud

#### Cloud Run
1. Build and push images to Container Registry
2. Deploy backend and frontend to Cloud Run
3. Use Cloud SQL for PostgreSQL

### DigitalOcean

#### App Platform
1. Connect GitHub repository
2. Configure environment variables
3. Deploy with managed database

## Monitoring

### Healthcheck URLs

| Service | Endpoint | Expected |
|---------|----------|----------|
| Backend | /health | 200 OK |
| Frontend | / | 200 OK |
| Database | pg_isready | 0 exit code |

### Logging

View logs:
```bash
# All services
docker-compose -f docker-compose.prod.yml logs -f

# Specific service
docker-compose -f docker-compose.prod.yml logs -f backend
```

### Resource Monitoring

```bash
# Container stats
docker stats

# Disk usage
df -h
```

## Backups

### Database

```bash
# Manual backup
docker exec tracks_postgres_prod pg_dump -U tracks_user tracks_db > backup.sql

# Scheduled backup (add to crontab)
0 2 * * * docker exec tracks_postgres_prod pg_dump -U tracks_user tracks_db | gzip > /backups/db_$(date +\%Y\%m\%d).sql.gz
```

### File Uploads

```bash
# Backup uploads volume
docker run --rm -v tracks_uploads_data:/data -v $(pwd):/backup \
  alpine tar cvf /backup/uploads.tar /data
```

## Updating

### Rolling Update

```bash
# Pull latest code
git pull origin main

# Rebuild images
docker-compose -f docker-compose.prod.yml build

# Restart services (one at a time for zero downtime)
docker-compose -f docker-compose.prod.yml up -d --no-deps backend
docker-compose -f docker-compose.prod.yml up -d --no-deps frontend

# Run any new migrations
docker exec tracks_backend_prod /app/tracks migrate
```

## Troubleshooting

### Common Issues

**Container won't start:**
```bash
# Check logs
docker-compose -f docker-compose.prod.yml logs <service>

# Check configuration
docker-compose -f docker-compose.prod.yml config
```

**Database connection failed:**
```bash
# Test connectivity
docker exec tracks_backend_prod nc -zv postgres 5432

# Check credentials
docker exec -it tracks_postgres_prod psql -U tracks_user -d tracks_db
```

**Out of disk space:**
```bash
# Clean up Docker
docker system prune -a

# Check volume usage
docker system df
```

See `docs/runbook.md` for more troubleshooting procedures.
