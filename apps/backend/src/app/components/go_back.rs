use leptos::prelude::*;
use leptos_router::components::A;

/// Quiet navigation link with a left arrow (e.g. "back to the blog list").
/// Muted by default; the arrow nudges left on hover (see `.back-link`).
#[component]
pub fn GoBack(go_to: &'static str, text: &'static str) -> impl IntoView {
    let reference = format!("/{go_to}");
    view! {
        <A href=reference attr:class="back-link">
            <svg
                class="back-link__arrow"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
            >
                <line x1="19" y1="12" x2="5" y2="12"></line>
                <polyline points="12 19 5 12 12 5"></polyline>
            </svg>
            <span>{text}</span>
        </A>
    }
}
