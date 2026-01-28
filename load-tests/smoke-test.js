import http from 'k6/http';
import { check, sleep } from 'k6';

// Smoke test - light load to verify the system is working
const BASE_URL = __ENV.API_URL || 'http://localhost:3001';

export const options = {
  vus: 1,
  duration: '10s',
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<500'],
  },
};

export default function () {
  // Test health endpoint
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'health: status is 200': (r) => r.status === 200,
  });

  // Test stats endpoint
  const statsRes = http.get(`${BASE_URL}/stats`);
  check(statsRes, {
    'stats: status is 200': (r) => r.status === 200,
    'stats: has expected fields': (r) => {
      const body = JSON.parse(r.body);
      return body.active_users !== undefined &&
             body.segments_created !== undefined &&
             body.activities_uploaded !== undefined;
    },
  });

  // Test segments list
  const segmentsRes = http.get(`${BASE_URL}/segments`);
  check(segmentsRes, {
    'segments: status is 200': (r) => r.status === 200,
  });

  sleep(1);
}
