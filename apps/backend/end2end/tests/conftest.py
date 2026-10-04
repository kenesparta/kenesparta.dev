"""Shared fixtures for the end-to-end suite.

The suite does not start the server: `cargo leptos end-to-end` does, or point
it at one that is already running with `--base-url`. Browser tests use the
`page` fixture from pytest-playwright (a fresh context per test) and relative
paths, resolved against `base_url`.
"""

from collections.abc import Iterator

import pytest
from playwright.sync_api import APIRequestContext, Playwright


@pytest.fixture(scope="session")
def api(playwright: Playwright, base_url: str) -> Iterator[APIRequestContext]:
    """HTTP client for header and redirect checks that need no browser.

    The site sets no cookies, so one context is safely shared by every test.
    """
    context = playwright.request.new_context(base_url=base_url)
    yield context
    context.dispose()
