use leptos::prelude::*;
use leptos_router::{components::A};

#[component]
pub fn GoBack(go_to: &'static str, text: &'static str) -> impl IntoView {
    let reference = format!("/{}", go_to);
    view! {
        <A href={reference} attr:class="back-link">{text}</A>
    }
}