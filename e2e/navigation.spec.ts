import { test, expect } from "@playwright/test";

/**
 * Tests for main navigation elements.
 * These tests verify authenticated navigation functionality.
 * Navigation is now in a sidebar on desktop and a drawer on mobile.
 */
test.describe("Navigation", () => {
  test.describe("Sidebar Navigation Links", () => {
    test("should display main navigation sections in sidebar", async ({
      page,
    }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      // On desktop, the sidebar should be visible
      const sidebar = page.locator("aside");
      await expect(sidebar).toBeVisible();

      // Should have Explore section with key links
      await expect(
        sidebar.getByRole("button", { name: /explore/i })
      ).toBeVisible();

      // Should have My Stuff section (auth only)
      await expect(
        sidebar.getByRole("button", { name: /my stuff/i })
      ).toBeVisible();
    });

    test("should expand Explore section and show links", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Click to expand Explore section if collapsed
      const exploreButton = sidebar.getByRole("button", { name: /explore/i });
      const isExpanded =
        (await exploreButton.getAttribute("aria-expanded")) === "true";
      if (!isExpanded) {
        await exploreButton.click();
      }

      // Verify navigation links in Explore section
      await expect(sidebar.getByRole("link", { name: /feed/i })).toBeVisible();
      await expect(
        sidebar.getByRole("link", { name: /daily activities/i })
      ).toBeVisible();
      await expect(
        sidebar.getByRole("link", { name: /segments/i })
      ).toBeVisible();
      await expect(
        sidebar.getByRole("link", { name: /leaderboards/i })
      ).toBeVisible();
      await expect(
        sidebar.getByRole("link", { name: /dig heatmap/i })
      ).toBeVisible();
    });

    test("should expand My Stuff section and show links", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Click to expand My Stuff section if collapsed
      const myStuffButton = sidebar.getByRole("button", { name: /my stuff/i });
      const isExpanded =
        (await myStuffButton.getAttribute("aria-expanded")) === "true";
      if (!isExpanded) {
        await myStuffButton.click();
      }

      // Verify My Activities link
      await expect(
        sidebar.getByRole("link", { name: /my activities/i })
      ).toBeVisible();

      // My Teams should be visible (may need to expand)
      await expect(
        sidebar.getByRole("button", { name: /my teams/i })
      ).toBeVisible();
    });

    test("should navigate to Feed page from sidebar", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Expand Explore if needed
      const exploreButton = sidebar.getByRole("button", { name: /explore/i });
      if ((await exploreButton.getAttribute("aria-expanded")) !== "true") {
        await exploreButton.click();
      }

      await sidebar.getByRole("link", { name: /feed/i }).click();
      await expect(page).toHaveURL(/\/feed/);
    });

    test("should navigate to Daily Activities page from sidebar", async ({
      page,
    }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Expand Explore if needed
      const exploreButton = sidebar.getByRole("button", { name: /explore/i });
      if ((await exploreButton.getAttribute("aria-expanded")) !== "true") {
        await exploreButton.click();
      }

      await sidebar.getByRole("link", { name: /daily activities/i }).click();
      await expect(page).toHaveURL(/\/activities\/daily/);
    });

    test("should navigate to My Activities page from sidebar", async ({
      page,
    }) => {
      await page.goto("/feed");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Expand My Stuff if needed
      const myStuffButton = sidebar.getByRole("button", { name: /my stuff/i });
      if ((await myStuffButton.getAttribute("aria-expanded")) !== "true") {
        await myStuffButton.click();
      }

      await sidebar.getByRole("link", { name: /my activities/i }).click();
      await expect(page).toHaveURL(/\/activities$/);
    });

    test("should navigate to Segments page from sidebar", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Expand Explore if needed
      const exploreButton = sidebar.getByRole("button", { name: /explore/i });
      if ((await exploreButton.getAttribute("aria-expanded")) !== "true") {
        await exploreButton.click();
      }

      await sidebar.getByRole("link", { name: /segments/i }).click();
      await expect(page).toHaveURL(/\/segments/);
    });

    test("should navigate to Leaderboards page from sidebar", async ({
      page,
    }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Expand Explore if needed
      const exploreButton = sidebar.getByRole("button", { name: /explore/i });
      if ((await exploreButton.getAttribute("aria-expanded")) !== "true") {
        await exploreButton.click();
      }

      await sidebar.getByRole("link", { name: /leaderboards/i }).click();
      await expect(page).toHaveURL(/\/leaderboards/);
    });

    test("should navigate to Dig Heatmap page from sidebar", async ({
      page,
    }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Expand Explore if needed
      const exploreButton = sidebar.getByRole("button", { name: /explore/i });
      if ((await exploreButton.getAttribute("aria-expanded")) !== "true") {
        await exploreButton.click();
      }

      await sidebar.getByRole("link", { name: /dig heatmap/i }).click();
      await expect(page).toHaveURL(/\/dig-heatmap/);
    });

    test("should navigate to Discover Teams page from sidebar", async ({
      page,
    }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      await sidebar.getByRole("link", { name: /discover teams/i }).click();
      await expect(page).toHaveURL(/\/teams\?view=discover/);
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
    test("should navigate to home when clicking logo in sidebar", async ({
      page,
    }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      // Find the logo link in the sidebar
      const sidebar = page.locator("aside");
      const logoLink = sidebar.getByRole("link", { name: /TRACKS\.RS/i });
      await expect(logoLink).toBeVisible();
      await logoLink.click();

      // Should navigate to home
      await expect(page).toHaveURL(/\/$/);
    });
  });

  test.describe("Sidebar Collapse", () => {
    test("should be able to collapse and expand sidebar", async ({ page }) => {
      await page.goto("/activities");
      await page.waitForLoadState("networkidle");

      const sidebar = page.locator("aside");

      // Find collapse button
      const collapseButton = sidebar.getByRole("button", {
        name: /collapse sidebar/i,
      });
      await expect(collapseButton).toBeVisible();

      // Click to collapse
      await collapseButton.click();

      // Now should show expand button
      const expandButton = sidebar.getByRole("button", {
        name: /expand sidebar/i,
      });
      await expect(expandButton).toBeVisible();

      // Click to expand
      await expandButton.click();

      // Collapse button should be back
      await expect(
        sidebar.getByRole("button", { name: /collapse sidebar/i })
      ).toBeVisible();
    });
  });
});
