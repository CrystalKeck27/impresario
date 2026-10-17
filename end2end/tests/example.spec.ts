import { test, expect } from "@playwright/test";

const baseRoutes = [
  { path: "/", heading: "Welcome to Leptos!" },
  { path: "/files", heading: "Files Page" },
  { path: "/identify", heading: "Identify music" },
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

test("identify page renders its path inputs and initial file explorer", async ({ page }) => {
  await page.goto("http://localhost:3000/identify");
  await expect(page.getByLabel("Source directory")).toBeVisible();
  await expect(page.getByLabel("Destination directory")).toBeVisible();
  await expect(page.getByRole("button", { name: "Run identification" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Identify music" })).toBeVisible();
});
