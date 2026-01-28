import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Stress test - gradually increase load to find breaking point
const BASE_URL = __ENV.API_URL || 'http://localhost:3001';
const errorRate = new Rate('errors');

export const options = {
  stages: [
    // Ramp up
    { duration: '30s', target: 50 },
    { duration: '1m', target: 100 },
    { duration: '1m', target: 200 },
    { duration: '1m', target: 400 },
    { duration: '1m', target: 600 },
    // Ramp down
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    errors: ['rate<0.1'], // Accept up to 10% error rate during stress
    http_req_duration: ['p(95)<2000'], // 2s is acceptable under stress
  },
};

export default function () {
  const endpoints = [
    { url: `${BASE_URL}/segments`, name: 'segments' },
    { url: `${BASE_URL}/stats`, name: 'stats' },
    { url: `${BASE_URL}/feed`, name: 'feed' },
  ];

  // Randomly pick an endpoint
  const endpoint = endpoints[Math.floor(Math.random() * endpoints.length)];

  const response = http.get(endpoint.url, {
    tags: { name: endpoint.name },
  });

  const success = check(response, {
    'status is acceptable': (r) => r.status === 200 || r.status === 401 || r.status === 404,
  });

  errorRate.add(!success);
  sleep(0.1);
}
