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

/**
 * Tests for viewing other users' profiles (/profile/[userId]).
 * These tests verify that viewing another user's profile works correctly,
 * including graceful handling when some data fails to load.
 */
test.describe("Other User Profile Page", () => {
  test("should display other user profile without 'User Not Found' error", async ({
    page,
  }) => {
    // First, go to daily activities to find a user
    await page.goto("/activities/daily");
    await page.waitForLoadState("networkidle");

    // Navigate to previous day which should have seeded activities
    const prevButton = page.getByRole("button", { name: /previous/i });
    await prevButton.click();
    await page.waitForLoadState("networkidle");

    // Find a user link in the activity list
    const userLink = page.locator("a").filter({ hasText: /\w+ \w+/ }).first();
    const hasUserLink = (await userLink.count()) > 0;

    if (hasUserLink) {
      // Get the user name before clicking
      const userName = await userLink.textContent();

      // Click on the user to go to their profile
      await userLink.click();
      await page.waitForLoadState("networkidle");

      // Verify we're on a profile page
      await expect(page).toHaveURL(/\/profile\//);

      // The profile should display the user's name, NOT "User Not Found"
      const userNotFound = page.getByText("User Not Found");
      await expect(userNotFound).not.toBeVisible();

      // The user's name should be visible
      if (userName) {
        await expect(page.getByText(userName.trim())).toBeVisible();
      }

      // Profile heading should be visible
      await expect(
        page.getByRole("heading", { name: /profile/i })
      ).toBeVisible();
    }
  });

  test("should display public activities section on other user profile", async ({
    page,
  }) => {
    // Go to daily activities and find a user
    await page.goto("/activities/daily");
    await page.waitForLoadState("networkidle");

    // Navigate to a previous day
    const prevButton = page.getByRole("button", { name: /previous/i });
    await prevButton.click();
    await page.waitForLoadState("networkidle");

    // Find and click a user link
    const userLink = page.locator("a").filter({ hasText: /\w+ \w+/ }).first();

    if ((await userLink.count()) > 0) {
      await userLink.click();
      await page.waitForLoadState("networkidle");

      // Verify public activities section exists
      await expect(page.getByText(/public activities/i)).toBeVisible();

      // Should show a count (number) for public activities
      const activityCount = page.locator("text=/\\d+/").first();
      await expect(activityCount).toBeVisible();
    }
  });

  test("should show follow button on other user profile", async ({ page }) => {
    // Navigate to daily activities and find a user
    await page.goto("/activities/daily");
    await page.waitForLoadState("networkidle");

    const prevButton = page.getByRole("button", { name: /previous/i });
    await prevButton.click();
    await page.waitForLoadState("networkidle");

    const userLink = page.locator("a").filter({ hasText: /\w+ \w+/ }).first();

    if ((await userLink.count()) > 0) {
      await userLink.click();
      await page.waitForLoadState("networkidle");

      // Should show a Follow or Unfollow button
      const followButton = page
        .getByRole("button", { name: /follow|unfollow/i })
        .first();
      await expect(followButton).toBeVisible();
    }
  });

  test("should display achievements section on other user profile", async ({
    page,
  }) => {
    await page.goto("/activities/daily");
    await page.waitForLoadState("networkidle");

    const prevButton = page.getByRole("button", { name: /previous/i });
    await prevButton.click();
    await page.waitForLoadState("networkidle");

    const userLink = page.locator("a").filter({ hasText: /\w+ \w+/ }).first();

    if ((await userLink.count()) > 0) {
      await userLink.click();
      await page.waitForLoadState("networkidle");

      // Should show achievements section
      await expect(page.getByText(/achievements/i)).toBeVisible();
    }
  });
});
