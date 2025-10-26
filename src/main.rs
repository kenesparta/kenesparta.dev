use kenesparta_dev::{About, HomePage, StickyNavBar, Blog};
use leptos::prelude::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use leptos_router::hooks::use_location;

fn main() {
    mount_to_body(|| {
        view! {
            <Router>
                <div class="app">
                    <main>
                        <ConditionalNavBar/>
                        <Routes fallback=|| "Page not found.".into_view()>
                            <Route path=StaticSegment("") view=HomePage/>
                            <Route path=StaticSegment("/about") view=About/>
                            <Route path=StaticSegment("/blog") view=Blog/>
                        </Routes>
                    </main>
                </div>
            </Router>
        }
    })
}

#[component]
fn ConditionalNavBar() -> impl IntoView {
    let location = use_location();
    let is_home = move || location.pathname.get() == "/";

    view! {
        <Show when=move || !is_home()>
            <StickyNavBar/>
        </Show>
    }
}