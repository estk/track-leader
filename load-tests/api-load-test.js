import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const feedLatency = new Trend('feed_latency');
const leaderboardLatency = new Trend('leaderboard_latency');
const activityLatency = new Trend('activity_latency');

// Configuration
const BASE_URL = __ENV.API_URL || 'http://localhost:3001';

// Test scenarios
export const options = {
  scenarios: {
    // GET /feed - 100 RPS target
    feed_load: {
      executor: 'constant-arrival-rate',
      rate: 100,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 50,
      maxVUs: 100,
      exec: 'testFeed',
    },
    // GET /segments/{id}/leaderboard - 200 RPS target
    leaderboard_load: {
      executor: 'constant-arrival-rate',
      rate: 200,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 100,
      maxVUs: 200,
      exec: 'testLeaderboard',
      startTime: '35s',
    },
    // GET /activities/{id} - 500 RPS target
    activity_load: {
      executor: 'constant-arrival-rate',
      rate: 500,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 200,
      maxVUs: 500,
      exec: 'testActivity',
      startTime: '70s',
    },
  },
  thresholds: {
    // Overall error rate
    errors: ['rate<0.01'], // Less than 1% errors
    // Feed endpoint: p95 < 200ms
    feed_latency: ['p(95)<200'],
    // Leaderboard endpoint: p95 < 150ms
    leaderboard_latency: ['p(95)<150'],
    // Activity endpoint: p95 < 100ms
    activity_latency: ['p(95)<100'],
  },
};

// Helper to get a random segment ID (adjust range based on your data)
function getRandomSegmentId() {
  return Math.floor(Math.random() * 100) + 1;
}

// Helper to get a random activity ID (adjust range based on your data)
function getRandomActivityId() {
  return Math.floor(Math.random() * 1000) + 1;
}

// Test feed endpoint
export function testFeed() {
  const url = `${BASE_URL}/feed`;
  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'feed' },
  };

  const startTime = Date.now();
  const response = http.get(url, params);
  const duration = Date.now() - startTime;

  feedLatency.add(duration);

  const success = check(response, {
    'feed: status is 200 or 401': (r) => r.status === 200 || r.status === 401,
  });

  errorRate.add(!success);
  sleep(0.01);
}

// Test leaderboard endpoint
export function testLeaderboard() {
  const segmentId = getRandomSegmentId();
  const url = `${BASE_URL}/segments/${segmentId}/leaderboard`;
  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'leaderboard' },
  };

  const startTime = Date.now();
  const response = http.get(url, params);
  const duration = Date.now() - startTime;

  leaderboardLatency.add(duration);

  const success = check(response, {
    'leaderboard: status is 200 or 404': (r) => r.status === 200 || r.status === 404,
  });

  errorRate.add(!success);
  sleep(0.01);
}

// Test activity endpoint
export function testActivity() {
  const activityId = getRandomActivityId();
  const url = `${BASE_URL}/activities/${activityId}`;
  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'activity' },
  };

  const startTime = Date.now();
  const response = http.get(url, params);
  const duration = Date.now() - startTime;

  activityLatency.add(duration);

  const success = check(response, {
    'activity: status is 200 or 404': (r) => r.status === 200 || r.status === 404,
  });

  errorRate.add(!success);
  sleep(0.01);
}

// Summary handler
export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'load-tests/results.json': JSON.stringify(data, null, 2),
  };
}

function textSummary(data, options) {
  const { metrics } = data;

  let output = '\n========== Load Test Summary ==========\n\n';

  output += 'Feed Endpoint (Target: 100 RPS, p95 < 200ms)\n';
  if (metrics.feed_latency) {
    output += `  p95: ${metrics.feed_latency.values['p(95)']?.toFixed(2) || 'N/A'}ms\n`;
    output += `  p99: ${metrics.feed_latency.values['p(99)']?.toFixed(2) || 'N/A'}ms\n`;
    output += `  avg: ${metrics.feed_latency.values.avg?.toFixed(2) || 'N/A'}ms\n`;
  }

  output += '\nLeaderboard Endpoint (Target: 200 RPS, p95 < 150ms)\n';
  if (metrics.leaderboard_latency) {
    output += `  p95: ${metrics.leaderboard_latency.values['p(95)']?.toFixed(2) || 'N/A'}ms\n`;
    output += `  p99: ${metrics.leaderboard_latency.values['p(99)']?.toFixed(2) || 'N/A'}ms\n`;
    output += `  avg: ${metrics.leaderboard_latency.values.avg?.toFixed(2) || 'N/A'}ms\n`;
  }

  output += '\nActivity Endpoint (Target: 500 RPS, p95 < 100ms)\n';
  if (metrics.activity_latency) {
    output += `  p95: ${metrics.activity_latency.values['p(95)']?.toFixed(2) || 'N/A'}ms\n`;
    output += `  p99: ${metrics.activity_latency.values['p(99)']?.toFixed(2) || 'N/A'}ms\n`;
    output += `  avg: ${metrics.activity_latency.values.avg?.toFixed(2) || 'N/A'}ms\n`;
  }

  output += '\nError Rate\n';
  if (metrics.errors) {
    output += `  Rate: ${(metrics.errors.values.rate * 100).toFixed(2)}%\n`;
  }

  output += '\n=========================================\n';

  return output;
}
