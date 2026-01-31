import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for E2E tests.
 *
 * Projects:
 * - setup: Runs auth.setup.ts to create authenticated storage state
 * - chromium: Authenticated tests using saved auth state
 * - chromium-unauth: Tests that don't require authentication
 *
 * Storage state is saved to playwright/.auth/user.json
 */
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? "github" : "html",

  use: {
    baseURL: process.env.E2E_BASE_URL || "http://localhost:3000",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  projects: [
    // Setup project: runs authentication and saves state
    {
      name: "setup",
      testMatch: /.*\.setup\.ts/,
    },

    // Authenticated tests (require login)
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        storageState: "playwright/.auth/user.json",
      },
      dependencies: ["setup"],
      // Exclude unauth directory from authenticated tests
      testIgnore: ["**/unauth/**"],
    },

    // Unauthenticated tests (login page, registration, redirects)
    {
      name: "chromium-unauth",
      use: { ...devices["Desktop Chrome"] },
      testMatch: ["**/unauth/**"],
    },

    // Optional: Additional browsers for broader coverage
    // Uncomment these for full browser matrix in CI
    // {
    //   name: "firefox",
    //   use: {
    //     ...devices["Desktop Firefox"],
    //     storageState: "playwright/.auth/user.json",
    //   },
    //   dependencies: ["setup"],
    //   testIgnore: ["**/unauth/**"],
    // },
    // {
    //   name: "webkit",
    //   use: {
    //     ...devices["Desktop Safari"],
    //     storageState: "playwright/.auth/user.json",
    //   },
    //   dependencies: ["setup"],
    //   testIgnore: ["**/unauth/**"],
    // },
  ],

  // Web server configuration for local development
  // In CI, we start the servers separately
  webServer: process.env.CI
    ? undefined
    : {
        command: "npm run dev",
        url: "http://localhost:3000",
        reuseExistingServer: true,
        timeout: 120000,
      },
});
