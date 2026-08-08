import { test, expect } from "@playwright/test";

const BASE = "http://localhost:3000";

// Regression guard for the trailing-slash panic (see src/seo.rs
// `redirect_trailing_slash` and src/app/api.rs `container`). A data route
// reached with a trailing slash (`/blog/`) used to fall through to the
// context-less error handler, where the async resource called
// `expect_context::<Container>()` and panicked the worker task — dropping the
// connection. It must now normalize to the canonical, slash-free URL with a
// permanent, method-preserving 308.
test.describe("trailing-slash normalization", () => {
  test("GET /blog/ redirects 308 to /blog", async ({ request }) => {
    const res = await request.get(`${BASE}/blog/`, { maxRedirects: 0 });
    expect(res.status()).toBe(308);
    expect(res.headers()["location"]).toBe("/blog");
  });

  test("GET /blog/<slug>/ drops only the trailing slash", async ({ request }) => {
    const res = await request.get(`${BASE}/blog/some-slug/`, { maxRedirects: 0 });
    expect(res.status()).toBe(308);
    expect(res.headers()["location"]).toBe("/blog/some-slug");
  });

  test("HEAD /blog/ returns 308 instead of dropping the connection", async ({ request }) => {
    const res = await request.head(`${BASE}/blog/`, { maxRedirects: 0 });
    expect(res.status()).toBe(308);
  });

  test("the query string survives the redirect", async ({ request }) => {
    const res = await request.get(`${BASE}/blog/?page=2`, { maxRedirects: 0 });
    expect(res.status()).toBe(308);
    expect(res.headers()["location"]).toBe("/blog?page=2");
  });

  test("following the redirect lands on the canonical page", async ({ page }) => {
    await page.goto(`${BASE}/blog/`);
    expect(page.url()).toBe(`${BASE}/blog`);
  });

  test("the canonical URL is served directly, no redirect", async ({ request }) => {
    const res = await request.get(`${BASE}/blog`, { maxRedirects: 0 });
    expect(res.status()).toBe(200);
  });
});
