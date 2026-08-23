//! HTTP wiring: server state, the Leptos server-function handler and the
//! complete router with its middleware stack.

use axum::Router;
use axum::extract::{FromRef, Request, State};
use axum::response::IntoResponse;
use leptos::prelude::LeptosOptions;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::app::{App, shell};
use crate::composition::Container;
use crate::{security, seo, telemetry};

/// Router state: Leptos options + the dependency container.
#[derive(Clone, FromRef)]
pub struct ServerState {
    pub leptos_options: LeptosOptions,
    pub container: Container,
}

/// Handle Leptos server functions with the `Container` available in the
/// reactive context (functions retrieve it with `use_context`).
pub async fn handle_server_fns(
    State(state): State<ServerState>,
    request: Request,
) -> impl IntoResponse {
    let container = state.container.clone();
    leptos_axum::handle_server_fns_with_context(
        move || leptos::context::provide_context(container.clone()),
        request,
    )
    .await
}

/// The whole application: routes, Leptos pages and every middleware, in
/// order. `main` binds it to a socket; tests drive it directly.
pub fn build_app(state: ServerState) -> Router {
    let routes = generate_route_list(App);

    let router = Router::new()
        // Leptos server functions, with the container in the reactive context.
        .route("/api/{*fn_name}", axum::routing::any(handle_server_fns))
        // Crawler endpoints (robots.txt is a static asset in public/).
        .route("/sitemap.xml", axum::routing::get(seo::sitemap))
        .route("/feed.xml", axum::routing::get(seo::feed))
        .route("/llms.txt", axum::routing::get(seo::llms_txt))
        // Publicly `/blog/{slug}.md`; the layer below rewrites it to this
        // internal path, which the router can actually express.
        .route("/blog-md/{slug}", axum::routing::get(seo::post_markdown))
        // Leptos pages (SSR + hydration).
        .leptos_routes_with_context(
            &state,
            routes,
            {
                let container = state.container.clone();
                move || leptos::context::provide_context(container.clone())
            },
            {
                let options = state.leptos_options.clone();
                move || shell(options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<ServerState, _>(shell))
        .layer(CompressionLayer::new())
        .with_state(state);

    // The Markdown-variant rewrite has to run BEFORE routing, and
    // `Router::layer` runs after it (it wraps each matched route's service), by
    // which point `/blog/{slug}.md` has already matched the Leptos page route
    // and 404s. Wrapping the whole router as an outer fallback puts the rewrite
    // in front of the matcher.
    Router::new()
        .fallback_service(router)
        .layer(axum::middleware::from_fn(seo::rewrite_markdown_suffix))
        // Collapse trailing slashes before routing (outside the rewrite, so it
        // sees the public URI). `/blog/` otherwise reaches the context-less
        // error handler and the data resource panics the worker — a crafted
        // URL must not take a thread down.
        .layer(axum::middleware::from_fn(seo::redirect_trailing_slash))
        // Security headers on every response, the redirects above included.
        .layer(axum::middleware::from_fn(security::headers))
        // Both outside the rewrite (added after it) so they see the public
        // URI, not the internal one. TraceLayer's per-request events are
        // DEBUG — quiet under the prod `info` filter, visible in dev — except
        // failures (5xx) at ERROR; it records no headers, so the CloudFront
        // origin-verify secret can never land in the logs. The access log is
        // outermost: one INFO event per page request with viewer IP + geo.
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(telemetry::access_log))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::body::Body;
    use axum::http::{HeaderMap, Request, StatusCode};
    use leptos::prelude::LeptosOptions;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::{ServerState, build_app};
    use crate::composition::wire;
    use crate::security::CSP_HEADER;

    /// The real app over a pool that never connects: pages without data
    /// render normally, data routes fail fast with a 500.
    fn app() -> axum::Router {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(1))
            .connect_lazy("postgres://nobody@127.0.0.1:1/none")
            .expect("lazy pool only parses the url");
        let leptos_options = LeptosOptions::builder()
            .output_name("kenespartadev")
            .build();
        build_app(ServerState {
            leptos_options,
            container: wire(pool),
        })
    }

    async fn get(path: &str) -> (StatusCode, HeaderMap, String) {
        let response = app()
            .oneshot(Request::get(path).body(Body::empty()).expect("request"))
            .await
            .expect("router never fails");
        let status = response.status();
        let headers = response.headers().clone();
        let body = axum::body::to_bytes(response.into_body(), 1 << 20)
            .await
            .expect("body");
        (status, headers, String::from_utf8_lossy(&body).into_owned())
    }

    fn header<'a>(headers: &'a HeaderMap, name: &str) -> &'a str {
        headers
            .get(name)
            .unwrap_or_else(|| panic!("missing header {name}"))
            .to_str()
            .expect("ascii header")
    }

    fn assert_static_headers(headers: &HeaderMap) {
        assert_eq!(header(headers, "x-content-type-options"), "nosniff");
        assert_eq!(header(headers, "x-frame-options"), "DENY");
        assert_eq!(
            header(headers, "referrer-policy"),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(header(headers, "cross-origin-opener-policy"), "same-origin");
        assert!(header(headers, "strict-transport-security").starts_with("max-age=63072000"));
        assert!(header(headers, "permissions-policy").contains("camera=()"));
    }

    /// The nonce in the policy and the nonce on the hydration script.
    fn nonces(headers: &HeaderMap, body: &str) -> (String, String) {
        let policy = header(headers, CSP_HEADER.as_str());
        let start = policy.find("'nonce-").expect("policy has a nonce") + "'nonce-".len();
        let end = policy[start..].find('\'').expect("nonce closes") + start;
        let in_policy = policy[start..end].to_owned();
        let marker = "<script type=\"module\" nonce=\"";
        let start = body.find(marker).expect("hydration script carries a nonce") + marker.len();
        let end = body[start..].find('"').expect("attribute closes") + start;
        (in_policy, body[start..end].to_owned())
    }

    #[tokio::test]
    async fn rendered_page_gets_a_policy_bound_to_its_hydration_nonce() {
        let (status, headers, body) = get("/").await;
        assert_eq!(status, StatusCode::OK);
        assert_static_headers(&headers);
        let (in_policy, on_script) = nonces(&headers, &body);
        assert_eq!(in_policy, on_script);
        assert!(in_policy.len() >= 16, "nonce too short: {in_policy}");
        let policy = header(&headers, CSP_HEADER.as_str());
        assert!(policy.contains("'wasm-unsafe-eval'"));
        assert!(policy.contains("frame-ancestors 'none'"));
        assert!(
            !policy.contains("ws://"),
            "no hot-reload socket outside watch mode"
        );
    }

    #[tokio::test]
    async fn every_request_gets_a_fresh_nonce() {
        let (_, first, body_a) = get("/").await;
        let (_, second, body_b) = get("/").await;
        assert_ne!(nonces(&first, &body_a).0, nonces(&second, &body_b).0);
    }

    #[tokio::test]
    async fn not_found_page_is_a_rendered_page_too() {
        let (status, headers, body) = get("/no-such-page").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_static_headers(&headers);
        let (in_policy, on_script) = nonces(&headers, &body);
        assert_eq!(in_policy, on_script);
    }

    #[tokio::test]
    async fn non_page_responses_get_the_fallback_policy() {
        // A redirect produced before routing …
        let (status, headers, _) = get("/blog/").await;
        assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
        assert_static_headers(&headers);
        assert_eq!(
            header(&headers, CSP_HEADER.as_str()),
            "default-src 'none'; frame-ancestors 'none'"
        );
        // … and a crawler endpoint, here failing on the dead pool.
        let (status, headers, body) = get("/sitemap.xml").await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_static_headers(&headers);
        assert_eq!(
            header(&headers, CSP_HEADER.as_str()),
            "default-src 'none'; frame-ancestors 'none'"
        );
        assert!(body.is_empty(), "a failing endpoint says nothing about why");
    }
}
