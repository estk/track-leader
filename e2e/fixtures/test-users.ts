/**
 * Known test user credentials for E2E tests.
 *
 * These users are created by the seed script (cargo run -p test-data --bin seed)
 * with predictable credentials. All test users have the password "tracks.rs".
 */

/** All test users share this password (set in test-data generator) */
export const TEST_PASSWORD = "tracks.rs";

/** Known E2E test user configuration */
export interface TestUser {
  id: string;
  email: string;
  password: string;
  name: string;
}

/**
 * Primary test user for authenticated E2E tests.
 * This user is created by the seed script with a known ID and credentials.
 */
export const TEST_USER_1: TestUser = {
  id: "00000000-0000-0000-0000-000000000e01",
  email: "test.user1@example.com",
  password: TEST_PASSWORD,
  name: "Test User One",
};

/**
 * Secondary test user for multi-user scenarios (e.g., following, teams).
 */
export const TEST_USER_2: TestUser = {
  id: "00000000-0000-0000-0000-000000000e02",
  email: "test.user2@example.com",
  password: TEST_PASSWORD,
  name: "Test User Two",
};

/**
 * Third test user for complex multi-user scenarios.
 */
export const TEST_USER_3: TestUser = {
  id: "00000000-0000-0000-0000-000000000e03",
  email: "test.user3@example.com",
  password: TEST_PASSWORD,
  name: "Test User Three",
};

/** All available test users */
export const TEST_USERS = [TEST_USER_1, TEST_USER_2, TEST_USER_3] as const;

/**
 * Base URL for E2E tests.
 * Defaults to localhost:3000 but can be overridden via E2E_BASE_URL.
 */
export function getBaseUrl(): string {
  return process.env.E2E_BASE_URL || "http://localhost:3000";
}

/**
 * API base URL for direct API calls during setup.
 */
export function getApiBaseUrl(): string {
  return process.env.E2E_API_URL || `${getBaseUrl()}/api`;
}
