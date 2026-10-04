"""Regression guard for the trailing-slash panic.

See src/seo.rs `redirect_trailing_slash` and src/app/api.rs `container`. A
data route reached with a trailing slash (`/blog/`) used to fall through to the
context-less error handler, where the async resource called
`expect_context::<Container>()` and panicked the worker task — dropping the
connection. It must now normalize to the canonical, slash-free URL with a
permanent, method-preserving 308.
"""

import re

import pytest
from playwright.sync_api import APIRequestContext, Page, expect


def test_get_blog_slash_redirects_308_to_blog(api: APIRequestContext) -> None:
    res = api.get("/blog/", max_redirects=0)
    assert res.status == 308
    assert res.headers["location"] == "/blog"


def test_get_blog_slug_slash_drops_only_the_trailing_slash(api: APIRequestContext) -> None:
    res = api.get("/blog/some-slug/", max_redirects=0)
    assert res.status == 308
    assert res.headers["location"] == "/blog/some-slug"


def test_head_blog_slash_returns_308_instead_of_dropping_the_connection(
    api: APIRequestContext,
) -> None:
    res = api.head("/blog/", max_redirects=0)
    assert res.status == 308


def test_the_query_string_survives_the_redirect(api: APIRequestContext) -> None:
    res = api.get("/blog/?page=2", max_redirects=0)
    assert res.status == 308
    assert res.headers["location"] == "/blog?page=2"


def test_following_the_redirect_lands_on_the_canonical_page(page: Page, base_url: str) -> None:
    page.goto("/blog/")
    expect(page).to_have_url(f"{base_url}/blog")


def test_the_canonical_url_is_served_directly_no_redirect(api: APIRequestContext) -> None:
    res = api.get("/blog", max_redirects=0)
    assert res.status == 200


# The normalizer must never emit a protocol-relative Location (`//host` or
# `/\host`), which browsers resolve off-site — an open redirect. Every redirect
# target has to stay a single-slash, same-origin absolute path.
@pytest.mark.parametrize("path", ["//evil.com/", "///evil.com/", "/\\evil.com/", "//evil.com//"])
def test_no_open_redirect(api: APIRequestContext, base_url: str, path: str) -> None:
    # Absolute on purpose: resolved against base_url, "//evil.com/" is itself
    # a protocol-relative URL and the request would leave for evil.com.
    res = api.get(f"{base_url}{path}", max_redirects=0)
    location = res.headers.get("location", "")
    # Same-origin: starts with a single slash, and the char after it is not
    # another slash or a backslash.
    assert re.match(r"/(?![/\\])", location), location
    assert "evil.com/" not in location
