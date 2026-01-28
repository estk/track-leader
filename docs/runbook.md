# Track Leader Operations Runbook

This document provides operational procedures for running Track Leader in production.

## Quick Reference

| Service | Port | Health Check |
|---------|------|-------------|
| Frontend | 3000 | GET / |
| Backend | 3001 | GET /health |
| PostgreSQL | 5432 | pg_isready |

## Deployment

### Prerequisites
- Docker and Docker Compose
- Domain with DNS configured
- SSL certificates (or use Let's Encrypt)

### Initial Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/track-leader.git
   cd track-leader
   ```

2. Create environment file:
   ```bash
   cp .env.example .env.prod
   # Edit .env.prod with production values
   ```

3. Required environment variables:
   ```bash
   POSTGRES_PASSWORD=<secure-password>
   NEXT_PUBLIC_API_URL=https://api.yourdomian.com
   ```

4. Start services:
   ```bash
   docker-compose -f docker-compose.prod.yml up -d
   ```

5. Run database migrations:
   ```bash
   docker exec tracks_backend_prod /app/tracks migrate
   ```

### Updating

1. Pull latest changes:
   ```bash
   git pull origin main
   ```

2. Rebuild and restart:
   ```bash
   docker-compose -f docker-compose.prod.yml build
   docker-compose -f docker-compose.prod.yml up -d
   ```

3. Run migrations if needed:
   ```bash
   docker exec tracks_backend_prod /app/tracks migrate
   ```

## Monitoring

### Health Checks

```bash
# Backend health
curl http://localhost:3001/health

# Frontend health
curl http://localhost:3000

# Database health
docker exec tracks_postgres_prod pg_isready -U tracks_user
```

### Logs

```bash
# All services
docker-compose -f docker-compose.prod.yml logs -f

# Specific service
docker-compose -f docker-compose.prod.yml logs -f backend

# Last 100 lines
docker-compose -f docker-compose.prod.yml logs --tail=100 backend
```

### Resource Usage

```bash
docker stats
```

## Troubleshooting

### Backend Not Starting

1. Check logs:
   ```bash
   docker-compose -f docker-compose.prod.yml logs backend
   ```

2. Verify database connection:
   ```bash
   docker exec tracks_backend_prod curl -s localhost:3001/health
   ```

3. Check environment variables:
   ```bash
   docker exec tracks_backend_prod env | grep DATABASE
   ```

### Database Connection Issues

1. Verify PostgreSQL is running:
   ```bash
   docker exec tracks_postgres_prod pg_isready -U tracks_user
   ```

2. Check connection from backend:
   ```bash
   docker exec tracks_backend_prod nc -zv postgres 5432
   ```

3. Verify credentials:
   ```bash
   docker exec -it tracks_postgres_prod psql -U tracks_user -d tracks_db -c '\l'
   ```

### High Memory Usage

1. Check container stats:
   ```bash
   docker stats --no-stream
   ```

2. Restart the affected service:
   ```bash
   docker-compose -f docker-compose.prod.yml restart backend
   ```

3. If persistent, adjust resource limits in docker-compose.prod.yml

### Slow Queries

1. Enable query logging (temporarily):
   ```bash
   docker exec -it tracks_postgres_prod psql -U tracks_user -d tracks_db -c "SET log_min_duration_statement = 100;"
   ```

2. Check slow query log:
   ```bash
   docker exec tracks_postgres_prod tail -f /var/log/postgresql/postgresql.log
   ```

3. Analyze query plans:
   ```sql
   EXPLAIN ANALYZE SELECT ...;
   ```

## Backup & Recovery

### Database Backup

```bash
# Create backup
docker exec tracks_postgres_prod pg_dump -U tracks_user tracks_db > backup_$(date +%Y%m%d).sql

# Automated daily backup (add to crontab)
0 2 * * * docker exec tracks_postgres_prod pg_dump -U tracks_user tracks_db | gzip > /backups/tracks_$(date +\%Y\%m\%d).sql.gz
```

### Database Restore

```bash
# Stop backend first
docker-compose -f docker-compose.prod.yml stop backend

# Restore
cat backup.sql | docker exec -i tracks_postgres_prod psql -U tracks_user -d tracks_db

# Restart backend
docker-compose -f docker-compose.prod.yml start backend
```

### Uploads Backup

```bash
# Backup uploads volume
docker run --rm -v tracks_uploads_data:/data -v $(pwd):/backup alpine tar cvf /backup/uploads_backup.tar /data
```

## Scaling

### Horizontal Scaling

For high traffic, deploy multiple backend instances behind a load balancer:

```bash
docker-compose -f docker-compose.prod.yml up -d --scale backend=3
```

Update nginx configuration to load balance across instances.

### Database Scaling

For read-heavy workloads, consider:
1. Read replicas
2. Connection pooling with PgBouncer
3. Caching layer (Redis)

## Security

### SSL Certificates

Using Let's Encrypt with certbot:
```bash
certbot certonly --webroot -w /var/www/certbot -d trackleader.com -d api.trackleader.com
```

### Firewall Rules

```bash
# Allow only necessary ports
ufw allow 80/tcp
ufw allow 443/tcp
ufw deny 5432/tcp  # Block direct DB access
ufw deny 3001/tcp  # Block direct API access (use nginx)
```

### Environment Variables

Never commit secrets to git. Use:
- Environment files (not committed)
- Docker secrets
- Cloud provider secret management

## Incident Response

### Service Down

1. Check container status:
   ```bash
   docker-compose -f docker-compose.prod.yml ps
   ```

2. Restart failed services:
   ```bash
   docker-compose -f docker-compose.prod.yml restart <service>
   ```

3. Check logs for root cause:
   ```bash
   docker-compose -f docker-compose.prod.yml logs --tail=500 <service>
   ```

### Data Corruption

1. Stop all services:
   ```bash
   docker-compose -f docker-compose.prod.yml down
   ```

2. Restore from backup (see Backup & Recovery)

3. Verify data integrity:
   ```sql
   -- Check for inconsistencies
   SELECT COUNT(*) FROM activities WHERE user_id NOT IN (SELECT id FROM users);
   ```

4. Restart services

## Contact

- On-call: #ops-alerts Slack channel
- Escalation: Platform team lead
