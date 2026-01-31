import { test as setup, expect } from "@playwright/test";
import { TEST_USER_1 } from "./fixtures/test-users";

const authFile = "playwright/.auth/user.json";

/**
 * Authentication setup that runs before authenticated tests.
 * Logs in via the UI and saves the storage state for reuse.
 */
setup("authenticate", async ({ page }) => {
  // Navigate to login page
  await page.goto("/login");

  // Fill in credentials
  await page.getByLabel(/email/i).fill(TEST_USER_1.email);
  await page.getByLabel(/password/i).fill(TEST_USER_1.password);

  // Submit login form
  await page.getByRole("button", { name: /sign in/i }).click();

  // Wait for redirect to authenticated area (e.g., /activities or /feed)
  // The app redirects to /activities after login
  await expect(page).toHaveURL(/\/(activities|feed|profile)/, {
    timeout: 10000,
  });

  // Verify we're logged in by checking for user-specific UI elements
  // Wait for any loading to complete
  await page.waitForLoadState("networkidle");

  // Save signed-in state to file
  await page.context().storageState({ path: authFile });
});
