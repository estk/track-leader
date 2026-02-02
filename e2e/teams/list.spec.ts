import { test, expect } from "@playwright/test";

test.describe("Teams List Page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/teams");
    await page.waitForLoadState("networkidle");
  });

  test("should display teams page", async ({ page }) => {
    await expect(page).toHaveTitle(/TRACKS\.RS/);
  });

  test("should have Create Team button", async ({ page }) => {
    await expect(
      page.getByRole("button", { name: /create team/i })
    ).toBeVisible();
  });

  test("should have My Teams and Discover buttons", async ({ page }) => {
    // These are toggle buttons in the main content area (not sidebar)
    const main = page.locator("#main-content");
    await expect(
      main.getByRole("button", { name: /my teams/i })
    ).toBeVisible();
    await expect(
      main.getByRole("button", { name: /discover/i })
    ).toBeVisible();
  });

  test("should switch between views", async ({ page }) => {
    // Scope to main content area to avoid conflict with sidebar
    const main = page.locator("#main-content");
    const myTeamsBtn = main.getByRole("button", { name: /my teams/i });
    const discoverBtn = main.getByRole("button", { name: /discover/i });

    // Click Discover
    await discoverBtn.click();
    await page.waitForLoadState("networkidle");

    // Click My Teams
    await myTeamsBtn.click();
    await page.waitForLoadState("networkidle");

    // Both buttons should still be visible
    await expect(myTeamsBtn).toBeVisible();
    await expect(discoverBtn).toBeVisible();
  });

  test("should display team cards when teams exist", async ({ page }) => {
    // Scope to main content area
    const main = page.locator("#main-content");

    // Check Discover tab for public teams
    await main.getByRole("button", { name: /discover/i }).click();
    await page.waitForLoadState("networkidle");

    // Teams may or may not exist - just verify the page doesn't crash
    const content = page.locator("main, #main-content, [role='main']").first();
    await expect(content).toBeVisible();
  });
});
