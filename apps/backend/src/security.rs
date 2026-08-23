//! Security response headers (SECURITY.md SEC-004).
//!
//! Nothing upstream adds them — Caddy only gates on `X-Origin-Verify` and the
//! CloudFront distribution has no response-headers policy — so the app, which
//! owns the markup, sets them on every response.
//!
//! The Content-Security-Policy is per request. Leptos hydrates through an
//! inline `<script type="module">`, so the policy carries a nonce that
//! `leptos_axum` generates for each render (leptos feature `nonce`) and
//! `<HydrationScripts>` stamps on that tag; every other inline script is
//! refused. [`provide_csp`] builds that header inside the render (called from
//! `shell`). [`headers`] adds the static headers to everything and a
//! nothing-allowed policy to responses that are not rendered pages: assets,
//! redirects and the crawler endpoints, none of which is a document that runs
//! scripts.

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;
use leptos::prelude::{LeptosOptions, use_context};

use crate::app::constants::BUCKET_URL;

/// Enforced. To observe without blocking, switch to
/// `content-security-policy-report-only`: violations then only show in the
/// browser console, and pages behave as if there were no policy.
pub const CSP_HEADER: HeaderName = HeaderName::from_static("content-security-policy");

/// What a response that is not a rendered page gets: no script, no embedding.
const FALLBACK_CSP: HeaderValue =
    HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'");

/// Set on every response. HSTS is belt-and-braces — `.dev` is on the browser
/// preload list, so plaintext is refused anyway — and is ignored over plain
/// HTTP, which is what local dev uses.
const STATIC_HEADERS: [(HeaderName, HeaderValue); 6] = [
    (
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
    ),
    (
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ),
    (header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY")),
    (
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    ),
    (
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=()"),
    ),
    (
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    ),
];

/// Middleware: the static headers on every response, plus the fallback policy
/// where the render did not set one. Mount it outside the redirect and rewrite
/// middlewares so their responses get the headers too.
pub async fn headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    for (name, value) in STATIC_HEADERS {
        headers.insert(name, value);
    }
    headers.entry(CSP_HEADER).or_insert(FALLBACK_CSP);
    response
}

/// The policy of a rendered page, bound to its `nonce`.
///
/// - `script-src`: the hydration script (nonce) and the wasm it loads from
///   `/pkg` (`'wasm-unsafe-eval'` is what lets a browser instantiate wasm at
///   all). JSON-LD `<script type="application/ld+json">` blocks are data, not
///   executed, and are not subject to `script-src`.
/// - `connect-src 'self'`: server-function fetches (`/api`) and the `.wasm`
///   fetch. `reload_ws_port` is cargo-leptos hot reload (dev only): its
///   websocket runs on another port, which `'self'` does not cover.
/// - Fonts and images come from the CDN, nothing else is loaded from anywhere.
/// - `frame-ancestors 'none'` is `X-Frame-Options: DENY` for browsers that
///   understand CSP 2; `base-uri` and `form-action` close the usual injection
///   escalations even though the site has neither `<base>` nor forms.
pub fn csp(nonce: &str, reload_ws_port: Option<u32>) -> String {
    let mut connect = String::from("'self'");
    if let Some(port) = reload_ws_port {
        connect.push_str(&format!(" ws://localhost:{port} ws://127.0.0.1:{port}"));
    }
    format!(
        "default-src 'self'; \
         script-src 'self' 'nonce-{nonce}' 'wasm-unsafe-eval'; \
         style-src 'self'; \
         img-src 'self' {BUCKET_URL}; \
         font-src {BUCKET_URL}; \
         connect-src {connect}; \
         object-src 'none'; \
         base-uri 'self'; \
         form-action 'self'; \
         frame-ancestors 'none'"
    )
}

/// Puts the page's policy on the response being built. Must run inside a
/// Leptos render — it reads the request nonce and `ResponseOptions` from the
/// reactive context; anywhere else (the client, a non-Leptos handler) it does
/// nothing and [`headers`] supplies the fallback policy instead.
pub fn provide_csp(options: &LeptosOptions) {
    let (Some(nonce), Some(response)) = (
        leptos::nonce::use_nonce(),
        use_context::<leptos_axum::ResponseOptions>(),
    ) else {
        return;
    };
    // Same gate as leptos's <AutoReload>: only cargo-leptos's watch mode sets it.
    let reload_ws_port = std::env::var("LEPTOS_WATCH")
        .is_ok()
        .then(|| options.reload_external_port.unwrap_or(options.reload_port));
    match HeaderValue::from_str(&csp(&nonce.to_string(), reload_ws_port)) {
        Ok(value) => response.insert_header(CSP_HEADER, value),
        // Unreachable in practice (the nonce is base64url, the rest is
        // static); never fail a page over a header.
        Err(error) => {
            tracing::error!(error = %error, "content-security-policy is not a valid header value")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::csp;

    #[test]
    fn policy_binds_the_nonce_and_locks_everything_else() {
        let policy = csp("abc123", None);
        assert!(policy.contains("script-src 'self' 'nonce-abc123' 'wasm-unsafe-eval';"));
        assert!(!policy.contains("unsafe-inline"));
        assert!(policy.contains("connect-src 'self';"));
        assert!(policy.contains("frame-ancestors 'none'"));
        assert!(policy.contains("object-src 'none'"));
        assert!(policy.contains("font-src https://cdn.kenesparta.dev;"));
        assert!(!policy.contains("ws://"));
    }

    #[test]
    fn dev_hot_reload_websocket_is_allowed_only_when_asked() {
        let policy = csp("n", Some(3001));
        assert!(policy.contains("connect-src 'self' ws://localhost:3001 ws://127.0.0.1:3001;"));
    }
}
