import { test, expect } from "@playwright/test";

test.describe("Authentication", () => {
  test("should display login page", async ({ page }) => {
    await page.goto("/login");
    await expect(page).toHaveTitle(/Track Leader/);
    await expect(page.getByRole("heading", { name: /sign in/i })).toBeVisible();
    await expect(page.getByLabel(/email/i)).toBeVisible();
    await expect(page.getByLabel(/password/i)).toBeVisible();
    await expect(page.getByRole("button", { name: /sign in/i })).toBeVisible();
  });

  test("should display register page", async ({ page }) => {
    await page.goto("/register");
    await expect(page.getByRole("heading", { name: /create an account/i })).toBeVisible();
    await expect(page.getByLabel(/name/i)).toBeVisible();
    await expect(page.getByLabel(/email/i)).toBeVisible();
    await expect(page.getByLabel("Password", { exact: true })).toBeVisible();
    await expect(page.getByLabel("Confirm Password")).toBeVisible();
  });

  test("should show validation errors for empty login", async ({ page }) => {
    await page.goto("/login");
    await page.getByRole("button", { name: /sign in/i }).click();
    // Should stay on login page or show validation
    await expect(page).toHaveURL(/login/);
  });

  test("should navigate between login and register", async ({ page }) => {
    await page.goto("/login");
    // Use the main content link (not header navigation)
    await page.locator("#main-content").getByRole("link", { name: /sign up/i }).click();
    await expect(page).toHaveURL(/register/);

    await page.locator("#main-content").getByRole("link", { name: /sign in/i }).click();
    await expect(page).toHaveURL(/login/);
  });

  test("should redirect unauthenticated users from protected routes", async ({ page }) => {
    await page.goto("/activities");
    // Should redirect to login
    await expect(page).toHaveURL(/login/);
  });
});
