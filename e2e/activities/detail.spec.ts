import { test, expect } from "@playwright/test";

/**
 * E2E tests for the Activity Detail page (/activities/[id]).
 * Tests activity header, map, elevation profile, stats, and actions.
 *
 * Note: The test user may not have activities, so these tests navigate
 * to view activities from the Daily Activities page (which shows all activities).
 */
test.describe("Activity Detail Page", () => {
  test.beforeEach(async ({ page }) => {
    // Try to find an activity from multiple sources
    let foundActivity = false;

    // Strategy 1: Try the feed which shows recent activities
    await page.goto("/feed");
    await page.waitForLoadState("networkidle");

    // Look for activity links in the feed (exclude navigation links)
    let activityLinks = page.locator('a[href^="/activities/"]:not([href*="upload"]):not([href*="daily"]):not([href="/activities"])');
    let count = await activityLinks.count();

    if (count > 0) {
      await activityLinks.first().click();
      await page.waitForLoadState("networkidle");
      foundActivity = true;
    }

    // Strategy 2: If no activities in feed, try segments leaderboard
    if (!foundActivity) {
      await page.goto("/segments");
      await page.waitForLoadState("networkidle");

      const segmentLink = page.locator('a[href^="/segments/"]:not([href="/segments"])').first();
      if ((await segmentLink.count()) > 0) {
        await segmentLink.click();
        await page.waitForLoadState("networkidle");

        // Try to find an activity link in the segment leaderboard
        activityLinks = page.locator('a[href^="/activities/"]:not([href*="upload"]):not([href*="daily"]):not([href="/activities"])');
        count = await activityLinks.count();
        if (count > 0) {
          await activityLinks.first().click();
          await page.waitForLoadState("networkidle");
          foundActivity = true;
        }
      }
    }

    // If we still can't find an activity, skip all tests in this describe block
    if (!foundActivity) {
      test.skip(true, "No activities available in test data");
    }
  });

  test("should display activity name in header", async ({ page }) => {
    // Look for a heading with the activity name
    const header = page.getByRole("heading", { level: 1 }).or(page.locator("h1"));
    await expect(header.first()).toBeVisible();
  });

  test("should display activity type badge", async ({ page }) => {
    // Look for activity type badges (Run, Ride, Hike, etc.)
    const typeBadge = page
      .locator(
        "text=/Run|Ride|Hike|Walk|Mountain Bike|Road Cycling|Gravel|E-Mountain Biking|Trail Work|Other/i"
      )
      .first();
    await expect(typeBadge).toBeVisible();
  });

  test("should display visibility badge", async ({ page }) => {
    // Look for visibility badge
    const visibilityBadge = page
      .locator("text=/Public|Private|Teams/i")
      .first();
    await expect(visibilityBadge).toBeVisible();
  });

  test("should display activity date", async ({ page }) => {
    // Look for a date - various formats possible
    const dateText = page
      .locator(
        "text=/\\w+\\s+\\d{1,2},?\\s*\\d{0,4}|\\d{1,2}\\/\\d{1,2}\\/\\d{2,4}/"
      )
      .first();
    await expect(dateText).toBeVisible();
  });

  test("should display route map", async ({ page }) => {
    // Look for map container
    const mapContainer = page
      .getByRole("region", { name: "Map" })
      .or(page.locator('[class*="map"], [data-testid="map"]'));
    await expect(mapContainer.first()).toBeVisible({ timeout: 10000 });
  });

  test("should display Elevation Profile", async ({ page }) => {
    // Look for elevation profile section by card title
    const elevationHeading = page.getByText(/elevation profile/i);
    await expect(elevationHeading).toBeVisible({ timeout: 10000 });
  });

  test("should display statistics", async ({ page }) => {
    // Look for any stats like Distance, Elevation, km, etc. in any card
    const stats = page
      .locator(
        "text=/\\d+(\\.\\d+)?\\s*(km|m|min|sec|s)|elevation|distance/i"
      )
      .first();
    await expect(stats).toBeVisible({ timeout: 10000 });
  });
});
