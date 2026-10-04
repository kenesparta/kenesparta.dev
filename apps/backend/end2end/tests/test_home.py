import re

import pytest
from playwright.sync_api import Page, expect


def test_homepage_has_title_and_heading_text(page: Page) -> None:
    page.goto("/")

    expect(page).to_have_title("Ken Esparta - Senior Software Engineer")
    expect(page.locator("h1")).to_have_text("Ken Esparta")


def test_homepage_footer_carries_the_release_stamp(page: Page) -> None:
    page.goto("/")

    # "dev" in local builds; "vX.Y.Z · build <7-hex>" in a tagged image.
    expect(page.locator("footer.home__footer")).to_have_text(
        re.compile(r"^(dev|v\d+\.\d+\.\d+ · build [0-9a-f]{7})$")
    )


@pytest.mark.parametrize("path", ["/about", "/blog"])
def test_has_no_release_stamp(page: Page, path: str) -> None:
    page.goto(path)

    expect(page.locator("footer.home__footer")).to_have_count(0)
