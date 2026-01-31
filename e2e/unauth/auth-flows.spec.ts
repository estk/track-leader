import { test, expect } from "@playwright/test";

/**
 * Unauthenticated tests for login/registration pages and auth flows.
 * These tests run without any stored authentication state.
 */
test.describe("Authentication Pages", () => {
  test("should display login page", async ({ page }) => {
    await page.goto("/login");
    await expect(page).toHaveTitle(/TRACKS\.RS/);
    await expect(
      page.getByRole("heading", { name: /sign in/i })
    ).toBeVisible();
    await expect(page.getByLabel(/email/i)).toBeVisible();
    await expect(page.getByLabel(/password/i)).toBeVisible();
    await expect(
      page.getByRole("button", { name: /sign in/i })
    ).toBeVisible();
  });

  test("should display register page", async ({ page }) => {
    await page.goto("/register");
    await expect(
      page.getByRole("heading", { name: /create an account/i })
    ).toBeVisible();
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
    await page
      .locator("#main-content")
      .getByRole("link", { name: /sign up/i })
      .click();
    await expect(page).toHaveURL(/register/);

    await page
      .locator("#main-content")
      .getByRole("link", { name: /sign in/i })
      .click();
    await expect(page).toHaveURL(/login/);
  });
});

test.describe("Authentication Redirects", () => {
  test("should redirect unauthenticated users from /activities", async ({
    page,
  }) => {
    await page.goto("/activities");
    await expect(page).toHaveURL(/login/);
  });

  test("should redirect unauthenticated users from /profile", async ({
    page,
  }) => {
    await page.goto("/profile");
    await expect(page).toHaveURL(/login/);
  });

  test("should redirect unauthenticated users from /feed", async ({ page }) => {
    await page.goto("/feed");
    await expect(page).toHaveURL(/login/);
  });

  test("should redirect unauthenticated users from /teams", async ({
    page,
  }) => {
    await page.goto("/teams");
    await expect(page).toHaveURL(/login/);
  });
});

test.describe("Landing Page (Public)", () => {
  test("should display hero section", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      /trail/i
    );
    await expect(
      page.getByRole("link", { name: /get started/i })
    ).toBeVisible();
    await expect(
      page.getByRole("link", { name: /explore segments/i })
    ).toBeVisible();
  });

  test("should display feature cards", async ({ page }) => {
    await page.goto("/");
    await expect(
      page.getByRole("heading", { name: /create segments/i })
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: /compete openly/i })
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: /community driven/i })
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: /gpx upload/i })
    ).toBeVisible();
  });

  test("should display stats section", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText(/active users/i)).toBeVisible();
    await expect(
      page.getByText("Segments Created", { exact: true })
    ).toBeVisible();
    await expect(
      page.getByText("Activities Uploaded", { exact: true })
    ).toBeVisible();
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
