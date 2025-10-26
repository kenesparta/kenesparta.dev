use leptos::prelude::*;

#[component]
pub fn HeaderLinks() -> impl IntoView {
    view! {
        <ul class="nav-links">
            <li><a href="/about" class="nav-links__element">"About"</a></li>
            <li><a href="/experience" class="nav-links__element">"Experience"</a></li>
            <li><a href="/blog" class="nav-links__element">"Blog"</a></li>
            <li><a href="/projects" class="nav-links__element">"Projects"</a></li>
        </ul>
    }
}
