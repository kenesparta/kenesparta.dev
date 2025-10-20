use leptos::prelude::*;

#[component]
pub fn HeaderLinks() -> impl IntoView {
    view! {
        <ul class="nav-links">
            <li><a href="/" class="nav-links__element">"Home"</a></li>
            <li><a href="/about" class="nav-links__element">"About"</a></li>
            <li><a href="/about" class="nav-links__element">"Experience"</a></li>
            <li><a href="/contact" class="nav-links__element">"Talks"</a></li>
            <li><a href="/contact" class="nav-links__element">"Projects"</a></li>
        </ul>
    }
}
