//! Backend of kenesparta.dev.
//!
//! Dual crate:
//! - As a **wasm library** (feature `hydrate`) it contains only the Leptos UI
//!   (`app/`) and hydrates in the browser.
//! - As a **binary** (feature `ssr`) it hosts every adapter (DynamoDB
//!   persistence, HTTP) and the wiring of the Bounded Contexts.

pub mod app;

#[cfg(feature = "ssr")]
pub mod composition;
#[cfg(feature = "ssr")]
pub mod configuration;
#[cfg(feature = "ssr")]
pub mod http;
#[cfg(feature = "ssr")]
pub mod persistence;

/// wasm entry point: hydrate the HTML already delivered by the server.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
