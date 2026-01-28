# Performance Targets

This document defines performance targets for Track Leader's API endpoints and frontend.

## API Performance Targets

| Endpoint | Target RPS | p95 Latency | p99 Latency |
|----------|-----------|-------------|-------------|
| GET /health | 1000 | < 10ms | < 50ms |
| GET /stats | 500 | < 50ms | < 100ms |
| GET /feed | 100 | < 200ms | < 500ms |
| GET /segments | 200 | < 100ms | < 200ms |
| GET /segments/{id} | 300 | < 100ms | < 200ms |
| GET /segments/{id}/leaderboard | 200 | < 150ms | < 300ms |
| GET /activities/{id} | 500 | < 100ms | < 200ms |
| POST /activities/new | 10 | < 2000ms | < 5000ms |
| POST /auth/login | 50 | < 300ms | < 500ms |

## Frontend Performance Targets

Based on Lighthouse metrics:

| Metric | Target |
|--------|--------|
| Performance Score | > 90 |
| First Contentful Paint | < 1.5s |
| Largest Contentful Paint | < 2.5s |
| Total Blocking Time | < 200ms |
| Cumulative Layout Shift | < 0.1 |

## Database Performance

### Query Targets
- Simple queries (single table, indexed): < 10ms
- Join queries (2-3 tables): < 50ms
- Complex aggregations: < 200ms

### Index Coverage
Critical paths have dedicated indexes:
- `idx_activities_user_type_date` - Activity list by user
- `idx_efforts_segment_time` - Leaderboard queries
- `idx_notifications_user_unread` - Notification badge counts

See `migrations/015_performance_indexes.sql` for full index list.

## Load Testing

### Running Tests

```bash
# Smoke test (quick validation)
k6 run load-tests/smoke-test.js

# Full load test
k6 run load-tests/api-load-test.js

# Stress test (find limits)
k6 run load-tests/stress-test.js
```

### Test Scenarios

1. **Smoke Test**: 1 VU for 10s - validates basic functionality
2. **Load Test**: Targets specific RPS for each endpoint
3. **Stress Test**: Ramps from 50 to 600 concurrent users

## Optimization Strategies

### Backend
- Connection pool sizing (see `main.rs`)
- Gzip compression enabled
- Prepared statements for frequent queries
- Response caching headers

### Frontend
- Lazy loading for MapLibre and Recharts
- Route-level code splitting
- Image optimization via Next.js
- Static asset caching (1 year for hashed assets)

### CDN
- Cloudflare caching for static assets
- Edge caching for public API responses
- See `docs/architecture/cdn.md` for configuration

## Monitoring

### Key Metrics to Track
- API response time (p50, p95, p99)
- Error rate by endpoint
- Database query time
- Cache hit ratio
- Memory usage
- CPU utilization

### Alerting Thresholds
- p95 latency > 500ms for any endpoint
- Error rate > 1%
- CPU > 80% sustained
- Memory > 90%
