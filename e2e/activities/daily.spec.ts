import { test, expect } from "@playwright/test";

/**
 * E2E tests for the Daily Activities page (/activities/daily).
 * Tests date navigation, filtering, and map display.
 */
test.describe("Daily Activities Page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/activities/daily");
  });

  test("should display page title", async ({ page }) => {
    await expect(page).toHaveTitle(/TRACKS\.RS/);
  });

  test("should have date picker", async ({ page }) => {
    // Look for the date input field
    const datePicker = page.locator('input[type="date"]');
    await expect(datePicker).toBeVisible();
  });

  test("should have previous and next navigation buttons", async ({ page }) => {
    // Look for navigation buttons (typically chevron icons or prev/next labels)
    const prevButton = page.getByRole("button", { name: /previous|prev|back/i }).or(
      page.locator('button:has(svg[class*="chevron-left"]), button[aria-label*="previous"]')
    );
    const nextButton = page.getByRole("button", { name: /next|forward/i }).or(
      page.locator('button:has(svg[class*="chevron-right"]), button[aria-label*="next"]')
    );

    await expect(prevButton.first()).toBeVisible();
    await expect(nextButton.first()).toBeVisible();
  });

  test("should have my activities only checkbox", async ({ page }) => {
    // The checkbox uses id="mine-only" with label "My activities only"
    const checkbox = page.locator("#mine-only");
    await expect(checkbox).toBeVisible();
  });

  test("should display map container or empty state", async ({ page }) => {
    // Wait for page to load
    await page.waitForLoadState("networkidle");

    // The page shows either a map with activities OR a "No Activities" state
    const mapContainer = page.getByRole("region", { name: "Map" });
    const noActivitiesHeading = page.getByRole("heading", { name: /no activities/i });

    // Either the map should be visible OR the empty state should be visible
    const mapVisible = await mapContainer.isVisible().catch(() => false);
    const emptyVisible = await noActivitiesHeading.isVisible().catch(() => false);

    expect(mapVisible || emptyVisible).toBeTruthy();
  });

  test("should display formatted date", async ({ page }) => {
    // Look for a date display in format like "Saturday, January 31, 2026"
    // The date text should contain day name, month, and year
    const dateDisplay = page.locator("text=/\\w+,\\s+\\w+\\s+\\d{1,2},\\s+\\d{4}/");
    await expect(dateDisplay.first()).toBeVisible();
  });

  test("should navigate to previous day when clicking prev button", async ({ page }) => {
    // Wait for page to load
    await page.waitForLoadState("networkidle");

    // Get the current date text
    const dateDisplay = page.locator("text=/\\w+,\\s+\\w+\\s+\\d{1,2},\\s+\\d{4}/").first();
    const initialDate = await dateDisplay.textContent();

    // Click previous button (has text "Previous")
    const prevButton = page.getByRole("button", { name: /previous/i });
    await prevButton.click();

    // Wait for navigation
    await page.waitForLoadState("networkidle");

    // Date should have changed
    const newDateDisplay = page.locator("text=/\\w+,\\s+\\w+\\s+\\d{1,2},\\s+\\d{4}/").first();
    const newDate = await newDateDisplay.textContent();
    expect(newDate).not.toBe(initialDate);
  });

  test("should toggle my activities only filter", async ({ page }) => {
    // Wait for page to load
    await page.waitForLoadState("networkidle");

    // The checkbox uses id="mine-only"
    const checkbox = page.locator("#mine-only");

    // Get initial state
    const initialChecked = await checkbox.isChecked();

    // Click the checkbox directly
    await checkbox.click();

    // Wait for the checkbox state to change
    await expect(checkbox).toBeChecked({ checked: !initialChecked, timeout: 5000 });
  });
});
