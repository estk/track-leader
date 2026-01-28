import { test, expect } from "@playwright/test";

test.describe("Segments", () => {
  test("should display segments page", async ({ page }) => {
    await page.goto("/segments");
    await expect(page).toHaveTitle(/Track Leader/);
    await expect(page.getByRole("heading", { name: /segments/i })).toBeVisible();
  });

  test("should have search functionality", async ({ page }) => {
    await page.goto("/segments");
    await expect(page.getByPlaceholder(/search/i)).toBeVisible();
  });

  test("should have activity type filters", async ({ page }) => {
    await page.goto("/segments");
    // Look for filter buttons
    await expect(page.getByRole("button", { name: /all types/i })).toBeVisible();
  });

  test("should have sort options", async ({ page }) => {
    await page.goto("/segments");
    // Look for sort select
    const sortSelect = page.locator("select").first();
    await expect(sortSelect).toBeVisible();
  });

  test("should toggle between list and map views", async ({ page }) => {
    await page.goto("/segments");
    const listButton = page.getByRole("button", { name: /list/i });
    const mapButton = page.getByRole("button", { name: /map/i });

    await expect(listButton).toBeVisible();
    await expect(mapButton).toBeVisible();

    // Click map view
    await mapButton.click();
    // Map should be visible (check for map container - use role to be specific)
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible({ timeout: 10000 });
  });
});
