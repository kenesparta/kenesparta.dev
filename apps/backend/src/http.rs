//! HTTP wiring: server state and the Leptos server-function handler.

use axum::extract::{FromRef, Request, State};
use axum::response::IntoResponse;
use leptos::prelude::LeptosOptions;

use crate::composition::Container;

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
