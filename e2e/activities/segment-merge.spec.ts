import { test, expect, Page } from "@playwright/test";
import path from "path";

/**
 * E2E tests for the Segment Merge Modal functionality.
 * Tests the flow when uploading multi-sport activities with adjacent same-type segments.
 */

const TEST_GPX_PATH = path.join(__dirname, "../../test-data/sample.gpx");

async function uploadFileAndEnableMultiSport(page: Page) {
  await page.goto("/activities/upload");
  await page.waitForLoadState("networkidle");

  // Upload the test GPX file
  const fileInput = page.locator('input[type="file"]');
  await fileInput.setInputFiles(TEST_GPX_PATH);

  // Wait for file to be parsed (elevation profile appears)
  await expect(page.locator("text=/Elevation Profile/i")).toBeVisible({
    timeout: 5000,
  });

  // Enable multi-sport mode
  const multiSportCheckbox = page.locator("#multi-sport");
  await multiSportCheckbox.check();

  // Verify multi-sport mode is enabled
  await expect(multiSportCheckbox).toBeChecked();

  // Wait for segment controls to appear
  await expect(page.locator("text=/Segment Types/i")).toBeVisible();
}

async function addBoundaryByClickingProfile(page: Page) {
  // Click on the elevation profile SVG to add a boundary
  // The recharts chart requires hover to set the active index before click
  const profileContainer = page.locator(".recharts-wrapper").first();
  await expect(profileContainer).toBeVisible();

  const box = await profileContainer.boundingBox();
  if (!box) throw new Error("Could not get elevation profile bounding box");

  // Calculate click position (roughly 50% across the chart)
  const clickX = box.x + box.width * 0.5;
  const clickY = box.y + box.height * 0.5;

  // First hover to trigger recharts tooltip and set lastHoveredIndex
  await page.mouse.move(clickX, clickY);
  // Wait for recharts tooltip to appear (indicating hover is active)
  await page.waitForTimeout(300);

  // Now click to add the boundary
  await page.mouse.click(clickX, clickY);

  // Wait for the second segment to appear
  await expect(page.locator("text=/Segment 2/i")).toBeVisible({ timeout: 5000 });
}

async function setSegmentTypes(page: Page, types: string[]) {
  // Find all segment type dropdowns
  const segmentSelects = page.locator(
    '.space-y-2 select, [class*="segment"] select'
  );

  for (let i = 0; i < types.length; i++) {
    const select = segmentSelects.nth(i);
    if (await select.isVisible()) {
      await select.selectOption({ label: types[i] });
    }
  }
}

test.describe("Segment Merge Modal", () => {
  test.beforeEach(async ({ page }) => {
    // Set activity name for all tests
    await page.goto("/activities/upload");
  });

  test("should show merge modal when adjacent segments have same type", async ({
    page,
  }) => {
    await uploadFileAndEnableMultiSport(page);
    await addBoundaryByClickingProfile(page);

    // Both segments should default to the same type (Run)
    // The modal should appear when we try to upload

    // Fill in the activity name
    const nameInput = page.getByLabel(/Activity Name/i);
    await nameInput.fill("Test Multi-Sport Activity");

    // Click upload
    const uploadButton = page.getByRole("button", { name: /upload/i });
    await uploadButton.click();

    // The merge modal should appear
    await expect(page.locator("text=/Combine Similar Segments/i")).toBeVisible({
      timeout: 3000,
    });

    // Verify modal has all three buttons
    await expect(page.getByRole("button", { name: /Merge/i })).toBeVisible();
    await expect(
      page.getByRole("button", { name: /Keep As-Is/i })
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /Edit Segments/i })
    ).toBeVisible();
  });

  test("should merge segments when clicking Merge button", async ({ page }) => {
    await uploadFileAndEnableMultiSport(page);
    await addBoundaryByClickingProfile(page);

    // Verify we have 2 segments
    await expect(page.locator("text=/Segment 2/i")).toBeVisible();

    // Fill in the activity name
    const nameInput = page.getByLabel(/Activity Name/i);
    await nameInput.fill("Merge Test Activity");

    // Click upload
    const uploadButton = page.getByRole("button", { name: /upload/i });
    await uploadButton.click();

    // Wait for merge modal
    await expect(page.locator("text=/Combine Similar Segments/i")).toBeVisible({
      timeout: 3000,
    });

    // Click Merge button
    const mergeButton = page.getByRole("button", { name: /^Merge$/i });
    await mergeButton.click();

    // Modal should close and upload should proceed
    // Should redirect to activities page on success
    await expect(page).toHaveURL(/\/activities/, { timeout: 10000 });
  });

  test("should keep segments when clicking Keep As-Is button", async ({
    page,
  }) => {
    await uploadFileAndEnableMultiSport(page);
    await addBoundaryByClickingProfile(page);

    // Fill in the activity name
    const nameInput = page.getByLabel(/Activity Name/i);
    await nameInput.fill("Keep As-Is Test Activity");

    // Click upload
    const uploadButton = page.getByRole("button", { name: /upload/i });
    await uploadButton.click();

    // Wait for merge modal
    await expect(page.locator("text=/Combine Similar Segments/i")).toBeVisible({
      timeout: 3000,
    });

    // Click Keep As-Is button
    const keepButton = page.getByRole("button", { name: /Keep As-Is/i });
    await keepButton.click();

    // Modal should close and upload should proceed with original boundaries
    // Should redirect to activities page on success
    await expect(page).toHaveURL(/\/activities/, { timeout: 10000 });
  });

  test("should return to editor when clicking Edit Segments button", async ({
    page,
  }) => {
    await uploadFileAndEnableMultiSport(page);
    await addBoundaryByClickingProfile(page);

    // Fill in the activity name
    const nameInput = page.getByLabel(/Activity Name/i);
    await nameInput.fill("Edit Test Activity");

    // Click upload
    const uploadButton = page.getByRole("button", { name: /upload/i });
    await uploadButton.click();

    // Wait for merge modal
    await expect(page.locator("text=/Combine Similar Segments/i")).toBeVisible({
      timeout: 3000,
    });

    // Click Edit Segments button
    const editButton = page.getByRole("button", { name: /Edit Segments/i });
    await editButton.click();

    // Modal should close
    await expect(
      page.locator("text=/Combine Similar Segments/i")
    ).not.toBeVisible();

    // Should still be on upload page with segments visible
    await expect(page).toHaveURL(/\/activities\/upload/);
    await expect(page.locator("text=/Segment 2/i")).toBeVisible();
  });

  test("should not show merge modal when segments have different types", async ({
    page,
  }) => {
    await uploadFileAndEnableMultiSport(page);
    await addBoundaryByClickingProfile(page);

    // Set different types for the two segments (Run and Road Cycling)
    await setSegmentTypes(page, ["Run", "Road Cycling"]);

    // Fill in the activity name
    const nameInput = page.getByLabel(/Activity Name/i);
    await nameInput.fill("Different Types Activity");

    // Click upload
    const uploadButton = page.getByRole("button", { name: /upload/i });
    await uploadButton.click();

    // The merge modal should NOT appear - should go straight to activities
    // (or dig tagging modal if stopped segments detected)
    await expect(
      page.locator("text=/Combine Similar Segments/i")
    ).not.toBeVisible({ timeout: 2000 });

    // Should proceed to upload (either redirect or show dig modal)
    await page.waitForURL(/\/activities/, { timeout: 10000 });
  });

  test("should display correct segment information in merge modal", async ({
    page,
  }) => {
    await uploadFileAndEnableMultiSport(page);
    await addBoundaryByClickingProfile(page);

    // Fill in the activity name
    const nameInput = page.getByLabel(/Activity Name/i);
    await nameInput.fill("Info Display Test");

    // Click upload
    const uploadButton = page.getByRole("button", { name: /upload/i });
    await uploadButton.click();

    // Wait for merge modal
    await expect(page.locator("text=/Combine Similar Segments/i")).toBeVisible({
      timeout: 3000,
    });

    // Verify the modal shows information about the mergeable segments
    // Should show "2 adjacent Run segments" in the modal
    await expect(page.getByText(/2 adjacent Run segments/i)).toBeVisible();

    // Should show segment numbers in the modal
    const modal = page.locator('[class*="fixed"]').filter({ hasText: "Combine Similar Segments" });
    await expect(modal.getByText(/Segment 1/i)).toBeVisible();
    await expect(modal.getByText(/Segment 2/i)).toBeVisible();
  });

  test("should not show merge modal for single segment activities", async ({
    page,
  }) => {
    await uploadFileAndEnableMultiSport(page);

    // Do NOT add any boundaries - single segment

    // Fill in the activity name
    const nameInput = page.getByLabel(/Activity Name/i);
    await nameInput.fill("Single Segment Activity");

    // Click upload
    const uploadButton = page.getByRole("button", { name: /upload/i });
    await uploadButton.click();

    // The merge modal should NOT appear
    await expect(
      page.locator("text=/Combine Similar Segments/i")
    ).not.toBeVisible({ timeout: 2000 });

    // Should proceed directly to upload
    await page.waitForURL(/\/activities/, { timeout: 10000 });
  });
});
