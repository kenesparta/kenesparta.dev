//! Server entry point.
//!
//! The only place in the workspace that knows about the runtime, the HTTP
//! server and Postgres. Everything mounted lives in `composition.rs`; here we
//! only assemble the Axum + Leptos router and start it.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;
    use backend::app::{App, shell};
    use backend::http::ServerState;
    use backend::{composition, configuration, http};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_http::compression::CompressionLayer;
    use tower_http::trace::TraceLayer;

    backend::telemetry::init();

    let config = configuration::Configuration::from_env()?;
    let container = composition::compose(&config).await?;

    // Leptos config: in dev cargo-leptos injects it via the environment; in the
    // container the LEPTOS_* variables set it.
    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = ServerState {
        leptos_options,
        container,
    };

    let router = Router::new()
        // Leptos server functions, with the container in the reactive context.
        .route(
            "/api/{*fn_name}",
            axum::routing::any(http::handle_server_fns),
        )
        // Crawler endpoints (robots.txt is a static asset in public/).
        .route("/sitemap.xml", axum::routing::get(backend::seo::sitemap))
        .route("/feed.xml", axum::routing::get(backend::seo::feed))
        .route("/llms.txt", axum::routing::get(backend::seo::llms_txt))
        // Publicly `/blog/{slug}.md`; the layer below rewrites it to this
        // internal path, which the router can actually express.
        .route(
            "/blog-md/{slug}",
            axum::routing::get(backend::seo::post_markdown),
        )
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
    let app = Router::new()
        .fallback_service(router)
        .layer(axum::middleware::from_fn(
            backend::seo::rewrite_markdown_suffix,
        ))
        // Collapse trailing slashes before routing (outside the rewrite, so it
        // sees the public URI). `/blog/` otherwise reaches the context-less
        // error handler and the data resource panics the worker — a crafted
        // URL must not take a thread down.
        .layer(axum::middleware::from_fn(
            backend::seo::redirect_trailing_slash,
        ))
        // Both outside the rewrite (added after it) so they see the public
        // URI, not the internal one. TraceLayer's per-request events are
        // DEBUG — quiet under the prod `info` filter, visible in dev — except
        // failures (5xx) at ERROR; it records no headers, so the CloudFront
        // origin-verify secret can never land in the logs. The access log is
        // outermost: one INFO event per page request with viewer IP + geo.
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(backend::telemetry::access_log));

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "kenesparta.dev listening");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// The binary only makes sense with the `ssr` feature (cargo-leptos builds it);
/// this branch exists so `cargo check` with default features does not fail.
#[cfg(not(feature = "ssr"))]
fn main() {}
