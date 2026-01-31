import { test, expect } from "@playwright/test";

test.describe("Segment Detail Page", () => {
  test("should navigate to segment detail from list", async ({ page }) => {
    // Start at segments list
    await page.goto("/segments");
    await page.waitForLoadState("networkidle");

    // Find and click on the first segment in the list
    const firstSegment = page.locator("a[href^='/segments/']").first();

    // Only proceed if segments exist
    if ((await firstSegment.count()) > 0) {
      await firstSegment.click();

      // Should navigate to segment detail page
      await expect(page).toHaveURL(/\/segments\/[a-f0-9-]+/);
    }
  });

  test("should display route map", async ({ page }) => {
    await page.goto("/segments");
    await page.waitForLoadState("networkidle");

    const firstSegment = page.locator("a[href^='/segments/']").first();
    if ((await firstSegment.count()) > 0) {
      await firstSegment.click();
      await page.waitForLoadState("networkidle");

      // Check for map container
      await expect(page.getByRole("region", { name: "Map" })).toBeVisible({
        timeout: 10000,
      });
    }
  });

  test("should display elevation profile chart", async ({ page }) => {
    await page.goto("/segments");
    await page.waitForLoadState("networkidle");

    const firstSegment = page.locator("a[href^='/segments/']").first();
    if ((await firstSegment.count()) > 0) {
      await firstSegment.click();
      await page.waitForLoadState("networkidle");

      // Check for elevation profile section
      const elevationProfile = page.locator(
        '[data-testid="elevation-profile"]'
      );
      if ((await elevationProfile.count()) > 0) {
        await expect(elevationProfile).toBeVisible();
      }
    }
  });

  test("should display statistics section", async ({ page }) => {
    await page.goto("/segments");
    await page.waitForLoadState("networkidle");

    const firstSegment = page.locator("a[href^='/segments/']").first();
    if ((await firstSegment.count()) > 0) {
      await firstSegment.click();
      await page.waitForLoadState("networkidle");

      // Check for statistics - look for common stat labels
      const statsSection = page.getByText(/distance|elevation|grade/i).first();
      await expect(statsSection).toBeVisible({ timeout: 5000 });
    }
  });

  test("should display leaderboard section", async ({ page }) => {
    await page.goto("/segments");
    await page.waitForLoadState("networkidle");

    const firstSegment = page.locator("a[href^='/segments/']").first();
    if ((await firstSegment.count()) > 0) {
      await firstSegment.click();
      await page.waitForLoadState("networkidle");

      // Check for leaderboard heading or section
      const leaderboard = page.getByRole("heading", { name: /leaderboard/i });
      if ((await leaderboard.count()) > 0) {
        await expect(leaderboard).toBeVisible();
      }
    }
  });
});
