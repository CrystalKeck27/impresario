import { test, expect } from "@playwright/test";

const baseRoutes = [
  { path: "/", heading: "Welcome to Leptos!" },
  { path: "/files", heading: "Files Page" },
  { path: "/fingerprint", heading: "Fingerprint a directory" },
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
    if (route.path === "/fingerprint") {
      await expect(page.getByLabel("Directory path")).toBeVisible();
      await expect(page.getByRole("button", { name: "Start fingerprinting" })).toBeVisible();
    }
    expect(errors, `Browser errors on ${route.path}`).toEqual([]);
  });
}

test("files page hydrates without a marker mismatch", async ({ page }) => {
  const hydrationErrors: string[] = [];
  page.on("console", (message) => {
    if (/hydration error|hydration mismatch/i.test(message.text())) {
      hydrationErrors.push(message.text());
    }
  });
  page.on("pageerror", (error) => {
    if (/hydration error|hydration mismatch/i.test(error.message)) {
      hydrationErrors.push(error.message);
    }
  });

  await page.goto("http://localhost:3000/files");
  await expect(page.getByRole("heading", { name: "AcoustID fingerprint" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Closest AcoustID match" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Tags", level: 3 })).toBeVisible();
  await expect.poll(() => hydrationErrors).toEqual([]);
});
