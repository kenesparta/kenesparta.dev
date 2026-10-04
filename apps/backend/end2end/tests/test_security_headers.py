"""SECURITY.md SEC-004.

Every response carries the security headers; rendered pages carry a
Content-Security-Policy bound to the request's nonce. The policy is enforced,
so the browser half matters most: hydration (an inline module script + wasm),
client-side routing and server-function fetches must all work under it. A
policy that breaks any of those fails here, not in production.
"""

import re

import pytest
from playwright.sync_api import APIRequestContext, ConsoleMessage, Page, Request, Response, expect

POLICY = re.compile(r"script-src 'self' 'nonce-[A-Za-z0-9_-]{16,}' 'wasm-unsafe-eval'")
FALLBACK_POLICY = "default-src 'none'; frame-ancestors 'none'"

# Only genuine policy violations and uncaught exceptions. A bare console-error
# filter would also catch e.g. Chromium logging the 404 status of a not-found
# page's own document.
CSP_WORDING = re.compile(
    r"content security policy|refused to (load|execute|apply|connect|frame)", re.IGNORECASE
)


def watch(page: Page) -> list[str]:
    violations: list[str] = []

    def on_console(msg: ConsoleMessage) -> None:
        if CSP_WORDING.search(msg.text):
            violations.append(msg.text)

    page.on("console", on_console)
    page.on("pageerror", lambda err: violations.append(f"pageerror: {err}"))
    return violations


def is_js(r: Response) -> bool:
    return r.url.endswith("/pkg/kenespartadev.js")


def is_wasm(r: Response) -> bool:
    return r.url.endswith("/pkg/kenespartadev.wasm")


def first_post_path(api: APIRequestContext) -> str | None:
    """The first published post's path, or None when the database has none.

    Posts are drafts until their author publishes them — the suite must not
    depend on this machine's content.
    """
    match = re.search(r'href="(/blog/[^"]+)" class="post-link"', api.get("/blog").text())
    return match[1] if match else None


def test_a_rendered_page_carries_the_nonce_bound_policy(api: APIRequestContext) -> None:
    h = api.get("/").headers
    csp = h["content-security-policy"]
    assert POLICY.search(csp), csp
    assert "frame-ancestors 'none'" in csp
    assert "unsafe-inline" not in csp
    assert h["x-content-type-options"] == "nosniff"
    assert h["x-frame-options"] == "DENY"
    assert h["referrer-policy"] == "strict-origin-when-cross-origin"
    assert h["strict-transport-security"].startswith("max-age=63072000")
    assert h["cross-origin-opener-policy"] == "same-origin"


def test_the_nonce_is_fresh_per_request(api: APIRequestContext) -> None:
    def nonce() -> str | None:
        match = re.search(r"'nonce-([^']+)'", api.get("/").headers["content-security-policy"])
        return match[1] if match else None

    a, b = nonce(), nonce()
    assert a
    assert a != b


def test_non_page_responses_get_the_no_script_fallback_policy(api: APIRequestContext) -> None:
    for path in ["/sitemap.xml", "/feed.xml", "/llms.txt"]:
        h = api.get(path).headers
        assert h["content-security-policy"] == FALLBACK_POLICY, path
        assert h["x-content-type-options"] == "nosniff", path
    redirect = api.get("/blog/", max_redirects=0)
    assert redirect.headers["content-security-policy"] == FALLBACK_POLICY


@pytest.mark.parametrize("path", ["/", "/blog"])
def test_hydrates_under_the_enforced_policy(page: Page, path: str) -> None:
    violations = watch(page)
    with page.expect_response(is_js) as js, page.expect_response(is_wasm) as wasm:
        page.goto(path)
    # The nonce'd inline script ran (it is what imports the module) and the
    # wasm it fetches was allowed through connect-src.
    assert js.value.status == 200
    assert wasm.value.status == 200
    page.wait_for_load_state("networkidle")
    assert violations == [], "\n".join(violations)


def test_a_post_page_hydrates_under_the_enforced_policy(page: Page, api: APIRequestContext) -> None:
    path = first_post_path(api)
    if path is None:
        pytest.skip("no published posts in this database")
    violations = watch(page)
    with page.expect_response(is_wasm) as wasm:
        page.goto(path)
    assert wasm.value.status == 200
    page.wait_for_load_state("networkidle")
    expect(page.locator("h1.post-title")).to_be_visible()
    assert violations == [], "\n".join(violations)


def test_client_side_routing_and_server_function_fetches_work_under_the_policy(
    page: Page, api: APIRequestContext, base_url: str
) -> None:
    path = first_post_path(api)
    if path is None:
        pytest.skip("no published posts in this database")
    violations = watch(page)
    # The waiter is armed on entering the block, before goto fires the request.
    with page.expect_response(is_wasm):
        page.goto("/blog")
    page.wait_for_load_state("networkidle")

    # A hydrated <A> navigates in place: the post's data arrives through a
    # server-function fetch (connect-src 'self'), not a document load.
    documents: list[str] = []

    def on_request(r: Request) -> None:
        if r.resource_type == "document":
            documents.append(r.url)

    page.on("request", on_request)
    with page.expect_response(lambda r: "/api/" in r.url) as server_fn:
        page.locator(f'a.post-link[href="{path}"]').click()
    assert server_fn.value.status == 200
    expect(page).to_have_url(f"{base_url}{path}")
    expect(page.locator("h1.post-title")).to_be_visible()
    assert documents == []
    assert violations == [], "\n".join(violations)
