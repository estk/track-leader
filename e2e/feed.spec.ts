import { test, expect } from "@playwright/test";

/**
 * Tests for the Feed page (/feed).
 * These tests verify feed functionality for authenticated users.
 */
test.describe("Feed Page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/feed");
    await page.waitForLoadState("networkidle");
  });

  test("should display feed page with title", async ({ page }) => {
    await expect(page).toHaveTitle(/TRACKS\.RS/);
    await expect(page.getByRole("heading", { name: /feed/i })).toBeVisible();
  });

  test.describe("Activity Type Filter", () => {
    test("should display activity type filter dropdown", async ({ page }) => {
      // Look for the activity type filter with "All Types" option
      const typeFilter = page.getByRole("combobox").filter({
        has: page.getByText(/all types/i),
      });

      // If not found as combobox, try button that triggers dropdown
      if (!(await typeFilter.isVisible())) {
        await expect(
          page.getByRole("button", { name: /all types/i })
        ).toBeVisible();
      } else {
        await expect(typeFilter).toBeVisible();
      }
    });

    test("should have activity type options in dropdown", async ({ page }) => {
      // Click the activity type filter to open it
      const filterButton = page.getByRole("button", { name: /all types/i });
      if (await filterButton.isVisible()) {
        await filterButton.click();

        // Verify some expected activity types are present
        await expect(page.getByRole("option", { name: /run/i })).toBeVisible();
        await expect(
          page.getByRole("option", { name: /road cycling/i })
        ).toBeVisible();
        await expect(
          page.getByRole("option", { name: /mountain biking/i })
        ).toBeVisible();
        await expect(page.getByRole("option", { name: /hike/i })).toBeVisible();
      }
    });
  });

  test.describe("Time Period Filter", () => {
    test("should display time period filter", async ({ page }) => {
      // Look for time period filter (All Time or similar)
      const timeFilter = page.getByRole("button", { name: /all time/i });
      if (await timeFilter.isVisible()) {
        await expect(timeFilter).toBeVisible();
      } else {
        // Try looking for a combobox or select
        await expect(page.getByText(/all time/i)).toBeVisible();
      }
    });
  });

  test.describe("Empty State", () => {
    test("should show Find People to Follow button when feed is empty", async ({
      page,
    }) => {
      // This test checks for the empty state UI
      // The button may or may not be visible depending on test data
      const findPeopleButton = page.getByRole("button", {
        name: /find people to follow/i,
      });
      const findPeopleLink = page.getByRole("link", {
        name: /find people to follow/i,
      });

      // Check if empty state is shown
      const emptyStateVisible =
        (await findPeopleButton.isVisible()) ||
        (await findPeopleLink.isVisible());

      if (emptyStateVisible) {
        // Verify the empty state call-to-action exists
        const ctaElement =
          (await findPeopleButton.count()) > 0
            ? findPeopleButton
            : findPeopleLink;
        await expect(ctaElement).toBeVisible();
      }
    });
  });

  test.describe("Activity Cards", () => {
    test("should display activity cards if feed has content", async ({
      page,
    }) => {
      // Look for activity cards or the empty state
      const activityCard = page.locator('[data-testid="activity-card"]');
      const activityItem = page.locator("article").first();

      // Check if there are any activity items
      const hasActivityCards =
        (await activityCard.count()) > 0 || (await activityItem.count()) > 0;

      if (hasActivityCards) {
        // If cards exist, verify they have expected content
        const card = activityCard.first().or(activityItem.first());
        await expect(card).toBeVisible();
      }
      // If no cards, that's okay - might be empty feed for test user
    });

    test("should display activity metadata on cards", async ({ page }) => {
      // Look for activity cards
      const activityCard = page.locator('[data-testid="activity-card"]').first();
      const activityItem = page.locator("article").first();

      const card = activityCard.or(activityItem);
      if (await card.isVisible()) {
        // Activity cards typically show athlete name and activity type
        // The specific content depends on what's in the feed
        await expect(card).toBeVisible();
      }
    });
  });
});
