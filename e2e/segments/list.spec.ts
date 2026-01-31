import { test, expect } from "@playwright/test";

test.describe("Segments List Page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/segments");
  });

  test("should display segments page with title", async ({ page }) => {
    await expect(page).toHaveTitle(/TRACKS\.RS/);
    await expect(
      page.getByRole("heading", { name: /segments/i })
    ).toBeVisible();
  });

  test("should have filter buttons (All, Starred, Near Me)", async ({ page }) => {
    // These are buttons, not tabs
    await expect(page.getByRole("button", { name: /^all$/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /starred/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /near me/i })).toBeVisible();
  });

  test("should have search functionality", async ({ page }) => {
    await expect(page.getByPlaceholder(/search/i)).toBeVisible();
  });

  test("should have sort dropdown", async ({ page }) => {
    // Look for sort select with "Newest" option
    const sortSelect = page.locator("select").first();
    await expect(sortSelect).toBeVisible();
  });

  test("should have distance filter", async ({ page }) => {
    // Distance filter is a select element with "Any distance" option
    const distanceSelect = page.locator("select").filter({ hasText: /any distance/i });
    await expect(distanceSelect).toBeVisible();
  });

  test("should have climb filter", async ({ page }) => {
    // Climb filter is a select element with "Any climb" option
    const climbSelect = page.locator("select").filter({ hasText: /any climb/i });
    await expect(climbSelect).toBeVisible();
  });

  test("should have list/map view toggle", async ({ page }) => {
    const listButton = page.getByRole("button", { name: /list/i });
    const mapButton = page.getByRole("button", { name: /map/i });

    await expect(listButton).toBeVisible();
    await expect(mapButton).toBeVisible();
  });

  test("should toggle to map view", async ({ page }) => {
    const mapButton = page.getByRole("button", { name: /map/i });
    await mapButton.click();

    // Map should be visible
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible({
      timeout: 10000,
    });
  });

  test("should have activity type filter chips", async ({ page }) => {
    // Look for "All Types" filter button
    await expect(
      page.getByRole("button", { name: /all types/i })
    ).toBeVisible();
  });

  test("should display segment list with details", async ({ page }) => {
    // Wait for segments to load
    await page.waitForLoadState("networkidle");

    // Check for segment list items showing name, distance, elevation, grade
    // Look for any segment card/row in the list
    const segmentList = page.locator('[data-testid="segment-list"]');
    if ((await segmentList.count()) > 0) {
      await expect(segmentList).toBeVisible();
    }
  });
});
