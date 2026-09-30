import { test, expect } from "@playwright/test";

test("homepage has title and heading text", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page).toHaveTitle("Ken Esparta - Senior Software Engineer");

  await expect(page.locator("h1")).toHaveText("Ken Esparta");
});

test("homepage footer carries the release stamp", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  // "dev" in local builds; "vX.Y.Z · build <7-hex>" in a tagged image.
  await expect(page.locator("footer.home__footer")).toHaveText(
    /^(dev|v\d+\.\d+\.\d+ · build [0-9a-f]{7})$/,
  );
});

for (const path of ["/about", "/blog"]) {
  test(`${path} has no release stamp`, async ({ page }) => {
    await page.goto(`http://localhost:3000${path}`);

    await expect(page.locator("footer.home__footer")).toHaveCount(0);
  });
}
