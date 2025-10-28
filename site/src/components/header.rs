use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <nav class="nav">
                <div>
                    <a href="/" class="brand-link">
                        <img src="/img/icon.svg" alt="Logo" class="brand-icon"/>
                        "Ken Esparta"
                    </a>
                </div>

                <ul class="nav-links">
                    <li><a href="/" class="nav-link">"Home"</a></li>
                    <li><a href="/about" class="nav-link">"About"</a></li>
                    <li><a href="/contact" class="nav-link">"Contact"</a></li>
                </ul>
            </nav>
        </header>
    }
}
