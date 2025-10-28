use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn HeaderLinks() -> impl IntoView {
    let location = use_location();

    view! {
        <ul class="nav-links">
            <li>
                <a
                    href="/about"
                    class=move || {
                        if location.pathname.get() == "/about" {
                            "nav-links__element nav-links__element--active"
                        } else {
                            "nav-links__element"
                        }
                    }
                >
                    "About"
                </a>
            </li>
            <li>
                <a
                    href="/experience"
                    class=move || {
                        if location.pathname.get() == "/experience" {
                            "nav-links__element nav-links__element--active"
                        } else {
                            "nav-links__element"
                        }
                    }
                >
                    "Experience"
                </a>
            </li>
            <li>
                <a
                    href="/blog"
                    class=move || {
                        if location.pathname.get() == "/blog" {
                            "nav-links__element nav-links__element--active"
                        } else {
                            "nav-links__element"
                        }
                    }
                >
                    "Blog"
                </a>
            </li>
            <li>
                <a
                    href="/projects"
                    class=move || {
                        if location.pathname.get() == "/projects" {
                            "nav-links__element nav-links__element--active"
                        } else {
                            "nav-links__element"
                        }
                    }
                >
                    "Projects"
                </a>
            </li>
        </ul>
    }
}