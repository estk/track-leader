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
 * These tests verify that viewing another user's profile works correctly.
 * Tests directly navigate to profile URLs to avoid click navigation issues.
 */
test.describe("Other User Profile Page", () => {
  // Use a helper function to get a valid user ID from the API
  async function getOtherUserId(page: any): Promise<string | null> {
    // Navigate to feed to find activities from other users
    await page.goto("/feed");
    await page.waitForLoadState("networkidle");

    // Look for profile links in the feed
    const profileLinks = page.locator('a[href^="/profile/"]');
    const count = await profileLinks.count();

    if (count > 0) {
      const href = await profileLinks.first().getAttribute("href");
      if (href) {
        // Extract user ID from /profile/{userId}
        const match = href.match(/\/profile\/([a-f0-9-]+)/);
        return match ? match[1] : null;
      }
    }
    return null;
  }

  test("should display other user profile without 'User Not Found' error", async ({
    page,
  }) => {
    const userId = await getOtherUserId(page);
    test.skip(!userId, "No other users found in feed");

    await page.goto(`/profile/${userId}`);
    await page.waitForLoadState("networkidle");

    // The profile should NOT display "User Not Found"
    const userNotFound = page.getByText("User Not Found");
    await expect(userNotFound).not.toBeVisible();

    // Profile heading should be visible
    await expect(
      page.getByRole("heading", { name: /profile/i })
    ).toBeVisible();

    // User info should be visible (avatar or name)
    const mainContent = page.locator("main");
    await expect(mainContent).toBeVisible();
  });

  test("should display public activities section on other user profile", async ({
    page,
  }) => {
    const userId = await getOtherUserId(page);
    test.skip(!userId, "No other users found in feed");

    await page.goto(`/profile/${userId}`);
    await page.waitForLoadState("networkidle");

    // Verify public activities section exists
    await expect(page.getByText(/public activities/i)).toBeVisible();

    // Should show activity count (could be 0 or more)
    const activitySection = page.locator("text=/\\d+ Public Activit/i");
    await expect(activitySection).toBeVisible();
  });

  test("should show follow button on other user profile", async ({ page }) => {
    const userId = await getOtherUserId(page);
    test.skip(!userId, "No other users found in feed");

    await page.goto(`/profile/${userId}`);
    await page.waitForLoadState("networkidle");

    // Should show a Follow or Unfollow button
    const followButton = page
      .getByRole("button", { name: /follow|unfollow/i })
      .first();
    await expect(followButton).toBeVisible();
  });

  test("should display achievements section on other user profile", async ({
    page,
  }) => {
    const userId = await getOtherUserId(page);
    test.skip(!userId, "No other users found in feed");

    await page.goto(`/profile/${userId}`);
    await page.waitForLoadState("networkidle");

    // Should show achievements section
    await expect(page.getByText(/achievements/i)).toBeVisible();
  });
});
