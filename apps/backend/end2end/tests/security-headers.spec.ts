import { test, expect } from "@playwright/test";

const BASE = "http://localhost:3000";
const POLICY = /script-src 'self' 'nonce-[A-Za-z0-9_-]{16,}' 'wasm-unsafe-eval'/;

// Only genuine policy violations and uncaught exceptions. A bare
// console-error filter would also catch e.g. Chromium logging the 404 status
// of a not-found page's own document.
const CSP_WORDING = /content security policy|refused to (load|execute|apply|connect|frame)/i;
function watch(page: import("@playwright/test").Page): string[] {
  const violations: string[] = [];
  page.on("console", (msg) => {
    if (CSP_WORDING.test(msg.text())) violations.push(msg.text());
  });
  page.on("pageerror", (err) => violations.push(`pageerror: ${err}`));
  return violations;
}

// The first published post's path, or null when the database has none (posts
// are drafts until their author publishes them — the spec must not depend on
// this machine's content).
async function firstPostPath(request: import("@playwright/test").APIRequestContext): Promise<string | null> {
  const html = await (await request.get(`${BASE}/blog`)).text();
  return html.match(/href="(\/blog\/[^"]+)" class="post-link"/)?.[1] ?? null;
}

// SECURITY.md SEC-004. Every response carries the security headers; rendered
// pages carry a Content-Security-Policy bound to the request's nonce. The
// policy is enforced, so the browser half matters most: hydration (an inline
// module script + wasm), client-side routing and server-function fetches must
// all work under it. A policy that breaks any of those fails here, not in
// production.
test.describe("security headers", () => {
  test("a rendered page carries the nonce-bound policy", async ({ request }) => {
    const h = (await request.get(`${BASE}/`)).headers();
    expect(h["content-security-policy"]).toMatch(POLICY);
    expect(h["content-security-policy"]).toContain("frame-ancestors 'none'");
    expect(h["content-security-policy"]).not.toContain("unsafe-inline");
    expect(h["x-content-type-options"]).toBe("nosniff");
    expect(h["x-frame-options"]).toBe("DENY");
    expect(h["referrer-policy"]).toBe("strict-origin-when-cross-origin");
    expect(h["strict-transport-security"]).toMatch(/^max-age=63072000/);
    expect(h["cross-origin-opener-policy"]).toBe("same-origin");
  });

  test("the nonce is fresh per request", async ({ request }) => {
    const nonce = (h: Record<string, string>) => h["content-security-policy"].match(/'nonce-([^']+)'/)?.[1];
    const a = nonce((await request.get(`${BASE}/`)).headers());
    const b = nonce((await request.get(`${BASE}/`)).headers());
    expect(a).toBeTruthy();
    expect(a).not.toBe(b);
  });

  test("non-page responses get the no-script fallback policy", async ({ request }) => {
    for (const path of ["/sitemap.xml", "/feed.xml", "/llms.txt"]) {
      const h = (await request.get(`${BASE}${path}`)).headers();
      expect(h["content-security-policy"], path).toBe("default-src 'none'; frame-ancestors 'none'");
      expect(h["x-content-type-options"], path).toBe("nosniff");
    }
    const redirect = await request.get(`${BASE}/blog/`, { maxRedirects: 0 });
    expect(redirect.headers()["content-security-policy"]).toBe("default-src 'none'; frame-ancestors 'none'");
  });

  for (const path of ["/", "/blog"]) {
    test(`hydrates under the enforced policy: ${path}`, async ({ page }) => {
      const violations = watch(page);
      const js = page.waitForResponse((r) => r.url().endsWith("/pkg/kenespartadev.js"));
      const wasm = page.waitForResponse((r) => r.url().endsWith("/pkg/kenespartadev.wasm"));
      await page.goto(`${BASE}${path}`);
      // The nonce'd inline script ran (it is what imports the module) and the
      // wasm it fetches was allowed through connect-src.
      expect((await js).status()).toBe(200);
      expect((await wasm).status()).toBe(200);
      await page.waitForLoadState("networkidle");
      expect(violations, violations.join("\n")).toEqual([]);
    });
  }

  test("a post page hydrates under the enforced policy", async ({ page, request }) => {
    const path = await firstPostPath(request);
    test.skip(path === null, "no published posts in this database");
    const violations = watch(page);
    const wasm = page.waitForResponse((r) => r.url().endsWith("/pkg/kenespartadev.wasm"));
    await page.goto(`${BASE}${path}`);
    expect((await wasm).status()).toBe(200);
    await page.waitForLoadState("networkidle");
    await expect(page.locator("h1.post-title")).toBeVisible();
    expect(violations, violations.join("\n")).toEqual([]);
  });

  test("client-side routing and server-function fetches work under the policy", async ({ page, request }) => {
    const path = await firstPostPath(request);
    test.skip(path === null, "no published posts in this database");
    const violations = watch(page);
    // Subscribe before navigating: the response fires during goto.
    const wasm = page.waitForResponse((r) => r.url().endsWith("/pkg/kenespartadev.wasm"));
    await page.goto(`${BASE}/blog`);
    await wasm;
    await page.waitForLoadState("networkidle");

    // A hydrated <A> navigates in place: the post's data arrives through a
    // server-function fetch (connect-src 'self'), not a document load.
    const documents: string[] = [];
    page.on("request", (r) => {
      if (r.resourceType() === "document") documents.push(r.url());
    });
    const serverFn = page.waitForResponse((r) => r.url().includes("/api/"));
    await page.locator(`a.post-link[href="${path}"]`).click();
    expect((await serverFn).status()).toBe(200);
    await expect(page).toHaveURL(`${BASE}${path}`);
    await expect(page.locator("h1.post-title")).toBeVisible();
    expect(documents).toEqual([]);
    expect(violations, violations.join("\n")).toEqual([]);
  });
});
