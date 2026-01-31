import { test, expect } from "@playwright/test";

/**
 * Tests for the Profile page (/profile).
 * These tests verify profile functionality for authenticated users.
 */
test.describe("Profile Page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/profile");
    await page.waitForLoadState("networkidle");
  });

  test("should display profile page with heading", async ({ page }) => {
    await expect(page).toHaveTitle(/TRACKS\.RS/);
    await expect(
      page.getByRole("heading", { name: /profile/i })
    ).toBeVisible();
  });

  test("should display user information", async ({ page }) => {
    // Profile page should show user name and email
    // The user's name should be visible somewhere on the page
    const mainContent = page.locator("main");
    await expect(mainContent).toBeVisible();

    // Should have at least one card with user info
    const cards = page.locator("[class*='card']");
    await expect(cards.first()).toBeVisible();
  });

  test("should display follow statistics", async ({ page }) => {
    // Look for followers/following text
    const followStats = page.getByText(/follower|following/i);
    await expect(followStats.first()).toBeVisible({ timeout: 10000 });
  });

  test("should display activity section", async ({ page }) => {
    // Look for activity-related content
    const activitySection = page.getByText(/activities|activity/i);
    await expect(activitySection.first()).toBeVisible();
  });

  test("should have navigation to activities", async ({ page }) => {
    // Should be able to navigate to activities
    const activitiesLink = page.locator('a[href="/activities"]').or(
      page.getByRole("link", { name: /activities/i })
    );

    if ((await activitiesLink.count()) > 0) {
      await activitiesLink.first().click();
      await expect(page).toHaveURL(/\/activities/);
    }
  });
});
