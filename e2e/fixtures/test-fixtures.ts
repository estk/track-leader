import { test as base, expect } from "@playwright/test";
import { TEST_USER_1, type TestUser } from "./test-users";

/**
 * Extended test fixtures for E2E tests.
 */
export const test = base.extend<{
  /** The currently logged-in test user */
  testUser: TestUser;
}>({
  testUser: async ({}, use) => {
    await use(TEST_USER_1);
  },
});

export { expect };

/**
 * Helper to wait for page to be ready after navigation.
 * Waits for network to be idle and any loading spinners to disappear.
 */
export async function waitForPageReady(
  page: import("@playwright/test").Page
): Promise<void> {
  await page.waitForLoadState("networkidle");
  // Wait for any loading overlays to disappear
  const loadingIndicator = page.locator('[data-testid="loading"]');
  if ((await loadingIndicator.count()) > 0) {
    await loadingIndicator.waitFor({ state: "hidden", timeout: 10000 });
  }
}

/**
 * Helper to format a date as YYYY-MM-DD.
 */
export function formatDate(date: Date): string {
  return date.toISOString().split("T")[0];
}

/**
 * Gets today's date formatted as YYYY-MM-DD.
 */
export function getToday(): string {
  return formatDate(new Date());
}

/**
 * Helper to check if an element exists without failing the test.
 */
export async function elementExists(
  page: import("@playwright/test").Page,
  selector: string
): Promise<boolean> {
  return (await page.locator(selector).count()) > 0;
}
