# Load Tests

Performance testing suite using [k6](https://k6.io/).

## Installation

```bash
# macOS
brew install k6

# Docker
docker pull grafana/k6
```

## Test Scripts

### smoke-test.js
Light load test to verify the system is working. Run this first to validate setup.

```bash
k6 run load-tests/smoke-test.js
```

### api-load-test.js
Main load test targeting specific performance thresholds:

| Endpoint | Target RPS | p95 Latency |
|----------|-----------|-------------|
| GET /feed | 100 | < 200ms |
| GET /segments/{id}/leaderboard | 200 | < 150ms |
| GET /activities/{id} | 500 | < 100ms |

```bash
k6 run load-tests/api-load-test.js
```

### stress-test.js
Gradually increases load to find the system's breaking point. Useful for capacity planning.

```bash
k6 run load-tests/stress-test.js
```

## Configuration

Set the API URL via environment variable:

```bash
k6 run -e API_URL=https://api.trackleader.com load-tests/api-load-test.js
```

## Running with Docker

```bash
docker run --rm -i grafana/k6 run - < load-tests/smoke-test.js
```

## Interpreting Results

k6 will output metrics including:
- `http_req_duration`: Response time statistics
- `http_reqs`: Total requests made
- `errors`: Custom error rate metric

Thresholds are defined in each test file. A test fails if any threshold is not met.

## CI Integration

Add to CI pipeline for continuous performance monitoring:

```yaml
- name: Run load tests
  run: |
    k6 run --out json=results.json load-tests/smoke-test.js
```
