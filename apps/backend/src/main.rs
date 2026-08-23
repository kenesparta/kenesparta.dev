//! Server entry point.
//!
//! The only place in the workspace that knows about the runtime, the HTTP
//! server and Postgres. Everything mounted lives in `composition.rs` and the
//! router in `http.rs`; here we only assemble them and start listening.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use backend::http::{ServerState, build_app};
    use backend::{composition, configuration};
    use leptos::prelude::get_configuration;

    backend::telemetry::init();

    let config = configuration::Configuration::from_env()?;
    let container = composition::compose(&config).await?;

    // Leptos config: in dev cargo-leptos injects it via the environment; in the
    // container the LEPTOS_* variables set it.
    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    let app = build_app(ServerState {
        leptos_options,
        container,
    });

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "kenesparta.dev listening");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// The binary only makes sense with the `ssr` feature (cargo-leptos builds it);
/// this branch exists so `cargo check` with default features does not fail.
#[cfg(not(feature = "ssr"))]
fn main() {}
