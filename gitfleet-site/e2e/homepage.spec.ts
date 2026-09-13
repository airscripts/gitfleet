import { expect, test } from "@playwright/test";

async function mockGitHubStars(page: import("@playwright/test").Page, count: number) {
  await page.route("https://api.github.com/repos/airscripts/gitfleet", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ stargazers_count: count }),
    });
  });
}

test.describe("homepage", () => {
  test("renders the hero, mocked star count, and keyboard landmarks", async ({ page }) => {
    await mockGitHubStars(page, 1284);
    await page.goto("/");

    await expect(page).toHaveTitle("Gitfleet");
    await expect(page.getByRole("heading", { level: 1, name: "Gitfleet" })).toBeAttached();
    await expect(page.getByRole("link", { name: "Download Releases" })).toBeVisible();
    await expect(page.getByRole("link", { name: /Star On GitHub/ })).toBeVisible();
    await expect(page.locator("[data-star-count]")).toHaveText("1.3K");

    await page.keyboard.press("Tab");
    await expect(page.getByRole("link", { name: "Skip to content" })).toBeFocused();

    await page.keyboard.press("Enter");
    await expect(page.locator("#main-content")).toBeFocused();
  });

  test("cycles light, dark, and system themes from the footer control", async ({ page }) => {
    await mockGitHubStars(page, 12);
    await page.emulateMedia({ colorScheme: "light" });
    await page.addInitScript(() => {
      localStorage.setItem("gitfleet-site-theme", "system");
    });

    await page.goto("/");
    const toggle = page.getByRole("button", { name: /theme/i });
    const root = page.locator("html");

    await expect(toggle).toBeVisible();
    await expect(root).toHaveAttribute("data-theme-preference", "system");
    await expect(root).toHaveAttribute("data-theme", "light");

    await toggle.click();
    await expect(root).toHaveAttribute("data-theme-preference", "light");
    await expect(root).toHaveAttribute("data-theme", "light");

    await toggle.click();
    await expect(root).toHaveAttribute("data-theme-preference", "dark");
    await expect(root).toHaveAttribute("data-theme", "dark");

    await toggle.click();
    await expect(root).toHaveAttribute("data-theme-preference", "system");
    await expect(root).toHaveAttribute("data-theme", "light");
  });

  test("keeps the star button usable when GitHub is unavailable", async ({ page }) => {
    await page.route("https://api.github.com/repos/airscripts/gitfleet", async (route) => {
      await route.fulfill({ status: 500, body: "unavailable" });
    });

    await page.goto("/");
    await expect(page.getByRole("link", { name: "Star On GitHub" })).toBeVisible();
    await expect(page.locator("[data-star-count]")).toBeEmpty();
  });
});
