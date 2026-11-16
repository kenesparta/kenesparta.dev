//! How long a browser may keep the files this server reads from disk.
//!
//! Everything under `site-root` — the `/pkg/` bundle cargo-leptos builds and
//! the `public/` files it copies beside it — leaves through `leptos_axum`'s
//! file handler, which says nothing about caching. Left unsaid, a browser
//! guesses from `Last-Modified`: a stylesheet a week old is kept for most of a
//! day on that guess, whether or not a deploy has replaced it since.
//!
//! [`static_files`] answers explicitly, and the answer turns on one question:
//! does the file's name change when its bytes do?
//!
//! - **`/pkg/` in a release build — yes.** The image is built with
//!   cargo-leptos's `hash-files`, so the bundle is
//!   `kenespartadev.<hash>.{css,js,wasm}` and every build is a new set of URLs.
//!   [`FINGERPRINTED`]: a year, and `immutable`, so a reload does not even ask.
//! - **`/pkg/` under `cargo leptos watch` — no.** Every rebuild reuses the
//!   name, so the browser has to ask every time: [`REVALIDATE`]. The switch is
//!   `LeptosOptions::hash_files`, the same flag that makes `shell()` link the
//!   hashed names at all, so the two cannot disagree.
//! - **`public/` — no, but** the icons and the manifest change about once per
//!   rebrand. [`STABLE_NAME`] is a day: long enough to be a cache, short
//!   enough that a replaced file ages out without anyone's help.
//!
//! CloudFront's `/pkg/*` behavior (personal-infra §5.16, `hashed_assets`)
//! keeps a file for as long as this header says, so the header is also what
//! makes the edge cache the bundle at all. It may say a year only because the
//! names are hashed (personal-infra G31).
//!
//! Rendered pages never pass through here — the layer is mounted on the file
//! handler alone (`http::build_app`) — so a page, whose CSP carries a nonce
//! minted for that one render, is never given a lifetime.

use axum::extract::{Request, State};
use axum::http::header::{ACCEPT_ENCODING, CACHE_CONTROL, VARY};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use leptos::prelude::LeptosOptions;

/// For a file whose name is derived from its contents.
///
/// A year is the conventional ceiling, and `immutable` tells the browser not
/// to revalidate even on a reload: the only way this URL's bytes change is by
/// the URL changing.
pub const FINGERPRINTED: &str = "public, max-age=31536000, immutable";

/// For a file whose name is reused by the next build.
pub const REVALIDATE: &str = "no-cache";

/// For a file whose name stays put when its bytes change.
pub const STABLE_NAME: &str = "public, max-age=86400";

/// Sets `Cache-Control` on every file the file handler finds.
///
/// Mounted on the fallback in [`crate::http::build_app`], with the app's
/// `LeptosOptions` as state.
pub async fn static_files(
    State(options): State<LeptosOptions>,
    request: Request,
    next: Next,
) -> Response {
    let policy = policy_for(&options, request.uri().path());
    let mut response = next.run(request).await;

    // Anything but a 200 out of this handler is its rendered 404, and a miss
    // is the one thing that must not be remembered — least of all for a year.
    if response.status() != StatusCode::OK {
        return response;
    }

    let headers = response.headers_mut();
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(policy));

    // A release build leaves a `.br` and a `.gz` beside every file, and the
    // handler picks one from `Accept-Encoding` without saying that it did.
    if !varies_on_encoding(headers) {
        headers.append(VARY, HeaderValue::from_name(ACCEPT_ENCODING));
    }

    response
}

/// The `Cache-Control` for the file at `path`.
pub fn policy_for(options: &LeptosOptions, path: &str) -> &'static str {
    let pkg_dir = options.site_pkg_dir.trim_matches('/');
    let in_pkg = path
        .strip_prefix('/')
        .and_then(|path| path.strip_prefix(pkg_dir))
        .is_some_and(|rest| rest.starts_with('/'));

    match (in_pkg, options.hash_files) {
        (true, true) => FINGERPRINTED,
        (true, false) => REVALIDATE,
        (false, _) => STABLE_NAME,
    }
}

fn varies_on_encoding(headers: &HeaderMap) -> bool {
    headers
        .get_all(VARY)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|name| name.trim().eq_ignore_ascii_case(ACCEPT_ENCODING.as_str()))
}

#[cfg(test)]
mod tests {
    use super::{FINGERPRINTED, REVALIDATE, STABLE_NAME, policy_for};
    use leptos::prelude::LeptosOptions;

    fn options(hash_files: bool) -> LeptosOptions {
        LeptosOptions::builder()
            .output_name("kenespartadev")
            .hash_files(hash_files)
            .build()
    }

    #[test]
    fn the_bundle_is_immutable_only_when_its_names_are_hashed() {
        let path = "/pkg/kenespartadev.CYlazpjM_Umf42C_ippKIA.css";

        assert_eq!(policy_for(&options(true), path), FINGERPRINTED);
        assert_eq!(policy_for(&options(false), path), REVALIDATE);
    }

    #[test]
    fn the_root_files_keep_their_names_so_they_keep_a_day() {
        for path in [
            "/favicon.ico",
            "/icon.svg",
            "/site.webmanifest",
            "/robots.txt",
        ] {
            assert_eq!(policy_for(&options(true), path), STABLE_NAME, "{path}");
            assert_eq!(policy_for(&options(false), path), STABLE_NAME, "{path}");
        }
    }

    /// A prefix match on the bare string would hand a year to a root file that
    /// merely starts with the same letters.
    #[test]
    fn only_the_pkg_directory_itself_counts() {
        assert_eq!(policy_for(&options(true), "/pkg"), STABLE_NAME);
        assert_eq!(policy_for(&options(true), "/pkgs/x.css"), STABLE_NAME);
        assert_eq!(policy_for(&options(true), "/pkg.css"), STABLE_NAME);
    }
}
