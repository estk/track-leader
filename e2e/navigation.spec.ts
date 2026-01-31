import { test, expect } from "@playwright/test";

/**
 * Tests for main navigation elements.
 * These tests verify authenticated navigation functionality.
 */
test.describe("Navigation", () => {
  test.describe("Main Navigation Links", () => {
    test("should display all main nav links", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      // Verify main navigation links are visible
      await expect(page.getByRole("link", { name: /feed/i })).toBeVisible();
      await expect(page.getByRole("link", { name: /daily/i })).toBeVisible();
      await expect(
        page.getByRole("link", { name: /my activities/i })
      ).toBeVisible();
      await expect(page.getByRole("link", { name: /segments/i })).toBeVisible();
      await expect(
        page.getByRole("link", { name: /leaderboards/i })
      ).toBeVisible();
      await expect(page.getByRole("link", { name: /teams/i })).toBeVisible();
    });

    test("should navigate to Feed page", async ({ page }) => {
      await page.goto("/activities");
      await page.getByRole("link", { name: /feed/i }).click();
      await expect(page).toHaveURL(/\/feed/);
    });

    test("should navigate to Daily page", async ({ page }) => {
      await page.goto("/activities");
      await page.getByRole("link", { name: /daily/i }).click();
      await expect(page).toHaveURL(/\/activities\/daily/);
    });

    test("should navigate to My Activities page", async ({ page }) => {
      await page.goto("/feed");
      await page.getByRole("link", { name: /my activities/i }).click();
      await expect(page).toHaveURL(/\/activities$/);
    });

    test("should navigate to Segments page", async ({ page }) => {
      await page.goto("/activities");
      await page.getByRole("link", { name: /segments/i }).click();
      await expect(page).toHaveURL(/\/segments/);
    });

    test("should navigate to Leaderboards page", async ({ page }) => {
      await page.goto("/activities");
      await page.getByRole("link", { name: /leaderboards/i }).click();
      await expect(page).toHaveURL(/\/leaderboards/);
    });

    test("should navigate to Teams page", async ({ page }) => {
      await page.goto("/activities");
      await page.getByRole("link", { name: /teams/i }).click();
      await expect(page).toHaveURL(/\/teams/);
    });
  });

  test.describe("User Menu", () => {
    test("should display user name and profile link", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      // The profile link shows the user's name
      // Look for the link in the header that goes to /profile
      const profileLink = page.locator('header a[href="/profile"]');
      await expect(profileLink).toBeVisible();
    });

    test("should navigate to profile when clicking user name", async ({
      page,
    }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      // Click the profile link in the header
      await page.locator('header a[href="/profile"]').click();
      await expect(page).toHaveURL(/\/profile/);
    });

    test("should have sign out button", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      // Sign out is a button in the header
      await expect(
        page.locator("header").getByRole("button", { name: /sign out/i })
      ).toBeVisible();
    });
  });

  test.describe("Logo Navigation", () => {
    test("should navigate to home when clicking logo", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      // Find the logo link by its aria-label
      const logoLink = page.getByRole("link", { name: /TRACKS\.RS - Home/i });
      await expect(logoLink).toBeVisible();
      await logoLink.click();

      // Should navigate to home
      await expect(page).toHaveURL(/\/$/);
    });
  });
});
