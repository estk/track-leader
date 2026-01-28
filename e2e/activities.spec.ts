import { test, expect } from "@playwright/test";

test.describe("Activities (Public)", () => {
  test("should redirect to login when not authenticated", async ({ page }) => {
    await page.goto("/activities");
    await expect(page).toHaveURL(/login/);
  });

  test("should redirect upload page to login when not authenticated", async ({ page }) => {
    await page.goto("/activities/upload");
    await expect(page).toHaveURL(/login/);
  });
});

test.describe("Landing Page", () => {
  test("should display hero section", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1 })).toContainText(/trail/i);
    await expect(page.getByRole("link", { name: /get started/i })).toBeVisible();
    await expect(page.getByRole("link", { name: /explore segments/i })).toBeVisible();
  });

  test("should display feature cards", async ({ page }) => {
    await page.goto("/");
    // Use heading role to target feature card titles specifically
    await expect(page.getByRole("heading", { name: /create segments/i })).toBeVisible();
    await expect(page.getByRole("heading", { name: /compete openly/i })).toBeVisible();
    await expect(page.getByRole("heading", { name: /community driven/i })).toBeVisible();
    await expect(page.getByRole("heading", { name: /gpx upload/i })).toBeVisible();
  });

  test("should display stats section", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText(/active users/i)).toBeVisible();
    // Use exact match to target stats section labels
    await expect(page.getByText("Segments Created", { exact: true })).toBeVisible();
    await expect(page.getByText("Activities Uploaded", { exact: true })).toBeVisible();
  });

  test("should navigate to register from hero", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: /get started/i }).click();
    await expect(page).toHaveURL(/register/);
  });

  test("should navigate to segments from hero", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: /explore segments/i }).click();
    await expect(page).toHaveURL(/segments/);
  });
});
