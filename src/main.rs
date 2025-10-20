use kenesparta_dev::HomePage;
use leptos::prelude::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

fn main() {
    mount_to_body(|| {
        view! {
            <Router>
                <div class="app">
                    <main>
                        <Routes fallback=|| "Page not found.".into_view()>
                            <Route path=StaticSegment("") view=HomePage/>
                        </Routes>
                    </main>
                </div>
            </Router>
        }
    })
}
