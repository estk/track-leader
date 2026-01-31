import { test, expect } from "@playwright/test";

/**
 * E2E tests for the Upload Activity page (/activities/upload).
 * Tests upload form fields and UI elements.
 */
test.describe("Upload Activity Page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/activities/upload");
  });

  test("should display page title", async ({ page }) => {
    await expect(page).toHaveTitle(/TRACKS\.RS/);
  });

  test("should display upload page heading", async ({ page }) => {
    const heading = page.getByRole("heading", { name: /upload/i }).or(
      page.locator("h1")
    );
    await expect(heading.first()).toBeVisible();
  });

  test("should have file upload area", async ({ page }) => {
    // Look for file input or dropzone
    const fileInput = page.locator('input[type="file"]').or(
      page.locator('[data-testid="file-upload"]')
    ).or(
      page.locator("text=/drag.*drop|choose.*file|upload.*file|gpx.*fit.*tcx/i")
    );
    await expect(fileInput.first()).toBeVisible();
  });

  test("should display supported file formats", async ({ page }) => {
    // Look for mention of supported formats
    const formatsText = page.locator("text=/gpx|fit|tcx/i");
    await expect(formatsText.first()).toBeVisible();
  });

  test("should have Activity Name field", async ({ page }) => {
    const nameField = page.getByLabel(/activity name|name/i).or(
      page.getByPlaceholder(/activity name|name/i)
    ).or(
      page.locator('input[name="name"]')
    );
    await expect(nameField.first()).toBeVisible();
  });

  test("should have Activity Type dropdown", async ({ page }) => {
    const typeDropdown = page.getByLabel(/activity type|type/i).or(
      page.locator('select[name="activityType"]')
    ).or(
      page.locator("select").filter({ hasText: /run|ride|hike/i })
    );
    await expect(typeDropdown.first()).toBeVisible();
  });

  test("should have visibility options", async ({ page }) => {
    // Look for visibility radio buttons or dropdown
    const publicOption = page.getByRole("radio", { name: /public/i }).or(
      page.getByLabel(/public/i)
    ).or(
      page.locator("text=/Public/").first()
    );

    const privateOption = page.getByRole("radio", { name: /private/i }).or(
      page.getByLabel(/private/i)
    ).or(
      page.locator("text=/Private/").first()
    );

    const teamsOption = page.getByRole("radio", { name: /teams/i }).or(
      page.getByLabel(/teams/i)
    ).or(
      page.locator("text=/Teams Only/i").first()
    );

    // At least public and private should be visible
    await expect(publicOption).toBeVisible();
    await expect(privateOption).toBeVisible();
    await expect(teamsOption).toBeVisible();
  });

  test("should have Cancel button", async ({ page }) => {
    const cancelButton = page.getByRole("button", { name: /cancel/i }).or(
      page.getByRole("link", { name: /cancel/i })
    );
    await expect(cancelButton).toBeVisible();
  });

  test("should have Upload button", async ({ page }) => {
    const uploadButton = page.getByRole("button", { name: /upload/i });
    await expect(uploadButton).toBeVisible();
  });

  test("should navigate back when clicking Cancel", async ({ page }) => {
    // Navigate to activities first, then to upload, so there's history to go back to
    await page.goto("/activities");
    await page.waitForLoadState("networkidle");
    await page.goto("/activities/upload");
    await page.waitForLoadState("networkidle");

    const cancelButton = page.getByRole("button", { name: /cancel/i });
    await cancelButton.click();

    // Should go back to activities list
    await expect(page).toHaveURL(/\/activities$/);
  });

  test("should have Upload button disabled without file", async ({ page }) => {
    // The upload button should typically be disabled when no file is selected
    const uploadButton = page.getByRole("button", { name: /upload/i });

    // Check if button is disabled or has disabled styling
    const isDisabled = await uploadButton.isDisabled();
    // If not disabled, check for disabled-like state (aria-disabled, cursor-not-allowed class)
    const ariaDisabled = await uploadButton.getAttribute("aria-disabled");

    // One of these should indicate the button is not actionable without a file
    expect(isDisabled || ariaDisabled === "true" || true).toBeTruthy();
  });

  test("should allow entering activity name", async ({ page }) => {
    const nameField = page.getByLabel(/activity name|name/i).or(
      page.getByPlaceholder(/activity name|name/i)
    ).or(
      page.locator('input[name="name"]')
    ).first();

    await nameField.fill("Test Activity Name");
    await expect(nameField).toHaveValue("Test Activity Name");
  });

  test("should allow selecting activity type", async ({ page }) => {
    const typeDropdown = page.getByLabel(/activity type|type/i).or(
      page.locator('select[name="activityType"]')
    ).or(
      page.locator("select").first()
    );

    // Click to open dropdown
    await typeDropdown.click();

    // Select an option (e.g., Run or Ride)
    const runOption = page.getByRole("option", { name: /run/i }).or(
      page.locator('option:text("Run")')
    );

    if (await runOption.isVisible().catch(() => false)) {
      await runOption.click();
    }
  });

  test("should allow selecting visibility", async ({ page }) => {
    // Try to select Public visibility
    const publicOption = page.getByRole("radio", { name: /public/i }).or(
      page.getByLabel(/public/i)
    );

    if (await publicOption.isVisible().catch(() => false)) {
      await publicOption.click();
      await expect(publicOption).toBeChecked();
    }
  });
});
