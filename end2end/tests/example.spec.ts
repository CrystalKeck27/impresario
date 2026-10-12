import { test, expect } from "@playwright/test";

const baseRoutes = [
  { path: "/", heading: "Welcome to Leptos!" },
  { path: "/files", heading: "Files Page" },
];

for (const route of baseRoutes) {
  test(`${route.path} renders without browser errors`, async ({ page }) => {
    const errors: string[] = [];

    page.on("console", (message) => {
      if (message.type() === "error") errors.push(`console.error: ${message.text()}`);
    });
    page.on("pageerror", (error) => errors.push(`pageerror: ${error.message}`));

    await page.goto(`http://localhost:3000${route.path}`);

    await expect(page.locator("h1")).toHaveText(route.heading);
    if (route.path === "/") {
      await expect(page).toHaveTitle("Welcome to Leptos");
    }
    expect(errors, `Browser errors on ${route.path}`).toEqual([]);
  });
}
