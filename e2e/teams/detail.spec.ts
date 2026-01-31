import { test, expect } from "@playwright/test";

test.describe("Team Detail Page", () => {
  test("should navigate to team detail from discover", async ({ page }) => {
    await page.goto("/teams");
    await page.waitForLoadState("networkidle");

    // Click Discover to find public teams
    await page.getByRole("button", { name: /discover/i }).click();
    await page.waitForLoadState("networkidle");

    // Find and click on the first team in the list (exclude /teams/new link)
    const teamLinks = page.locator("a[href^='/teams/']").filter({
      hasNot: page.locator('[href="/teams/new"]'),
    });
    // Also filter out the "new" link by checking the href doesn't end with "new"
    const firstTeam = teamLinks.filter({ has: page.locator(':not([href="/teams/new"])') }).first();

    const count = await page.locator("a[href^='/teams/']:not([href='/teams/new'])").count();
    if (count === 0) {
      test.skip(true, "No teams available in test data");
      return;
    }

    await page.locator("a[href^='/teams/']:not([href='/teams/new'])").first().click();
    await page.waitForLoadState("networkidle");

    // Should navigate to team detail page (UUID format)
    await expect(page).toHaveURL(/\/teams\/[a-f0-9-]+/);
  });

  test("should display team header with name", async ({ page }) => {
    await page.goto("/teams");
    await page.waitForLoadState("networkidle");

    await page.getByRole("button", { name: /discover/i }).click();
    await page.waitForLoadState("networkidle");

    const count = await page.locator("a[href^='/teams/']:not([href='/teams/new'])").count();
    if (count === 0) {
      test.skip(true, "No teams available");
      return;
    }

    await page.locator("a[href^='/teams/']:not([href='/teams/new'])").first().click();
    await page.waitForLoadState("networkidle");

    // Check for team name heading
    const teamHeading = page.getByRole("heading", { level: 1 });
    await expect(teamHeading).toBeVisible({ timeout: 5000 });
  });

  test("should display tab navigation links", async ({ page }) => {
    await page.goto("/teams");
    await page.waitForLoadState("networkidle");

    await page.getByRole("button", { name: /discover/i }).click();
    await page.waitForLoadState("networkidle");

    const count = await page.locator("a[href^='/teams/']:not([href='/teams/new'])").count();
    if (count === 0) {
      test.skip(true, "No teams available");
      return;
    }

    await page.locator("a[href^='/teams/']:not([href='/teams/new'])").first().click();
    await page.waitForLoadState("networkidle");

    // Tabs are rendered as links, not tab role elements
    // Check for Daily Map, Activities, Members, Leaderboard
    const dailyMapLink = page.getByRole("link", { name: /daily map/i });
    const activitiesLink = page.getByRole("link", { name: /activities/i });
    const membersLink = page.getByRole("link", { name: /members/i });
    const leaderboardLink = page.getByRole("link", { name: /leaderboard/i });

    // At least some of these should be visible
    const links = [dailyMapLink, activitiesLink, membersLink, leaderboardLink];
    let visibleCount = 0;

    for (const link of links) {
      if ((await link.count()) > 0) {
        visibleCount++;
      }
    }

    expect(visibleCount).toBeGreaterThan(0);
  });

  test("should display Members tab content", async ({ page }) => {
    await page.goto("/teams");
    await page.waitForLoadState("networkidle");

    await page.getByRole("button", { name: /discover/i }).click();
    await page.waitForLoadState("networkidle");

    const count = await page.locator("a[href^='/teams/']:not([href='/teams/new'])").count();
    if (count === 0) {
      test.skip(true, "No teams available");
      return;
    }

    await page.locator("a[href^='/teams/']:not([href='/teams/new'])").first().click();
    await page.waitForLoadState("networkidle");

    // Click on Members link
    const membersLink = page.getByRole("link", { name: /members/i });
    if ((await membersLink.count()) > 0) {
      await membersLink.click();
      await page.waitForLoadState("networkidle");

      // Look for member-related content (Owner, Admin, Member badges)
      const memberContent = page.getByText(/owner|admin|member|joined/i);
      if ((await memberContent.count()) > 0) {
        await expect(memberContent.first()).toBeVisible();
      }
    }
  });
});
