pub mod api;
pub mod components;
pub mod constants;
pub mod pages;

use crate::app::components::StickyNavBar;
use crate::app::constants::{BUCKET_URL, GLOBAL_FONTS, ICON_URL};
use crate::app::pages::{About, BlogList, BlogPost, Experience, HomePage, Projects};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_location;
use leptos_router::{
    SsrMode, StaticSegment,
    components::{Route, Router, Routes},
    path,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Link rel="icon" type_="image/x-icon" href={ICON_URL}/>

        <Link rel="dns-prefetch" href={BUCKET_URL}/>
        <Link rel="preconnect" href={BUCKET_URL} crossorigin="anonymous"/>
        <Link rel="preload" href="/pkg/kenespartadev.css" as_="style"/>
        <Stylesheet id="leptos" href="/pkg/kenespartadev.css"/>

        <FontsPrefetch fonts=GLOBAL_FONTS/>

        // Titles, descriptions, canonicals and Open Graph are per page
        // (<PageMeta>, mounted by every routed page); only route-independent
        // tags live here. The snippet/preview caps are lifted so search and
        // AI answers may quote full passages.
        <Meta name="robots" content="index, follow, max-snippet:-1, max-image-preview:large"/>
        <Link rel="alternate" type_="application/rss+xml" title="Ken Esparta - Blog" href="/feed.xml"/>

        <Router>
            <div class="app">
                <main>
                    <ConditionalNavBar/>
                    <Routes fallback=|| view! { <NotFound/> }>
                        <Route path=StaticSegment("") view=HomePage/>
                        <Route path=StaticSegment("/about") view=About/>
                        // Async SSR for the database-driven pages: the finished
                        // HTML is rendered in place. The default out-of-order
                        // streaming ships a "Loading..." fallback plus an inert
                        // <template> swapped in by script — invisible to the AI
                        // crawlers that do not execute JavaScript.
                        <Route path=StaticSegment("/blog") view=BlogList ssr=SsrMode::Async/>
                        <Route path=path!("/blog/:slug") view=BlogPost ssr=SsrMode::Async/>
                        <Route path=StaticSegment("/experience") view=Experience/>
                        <Route path=StaticSegment("/projects") view=Projects/>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

/// Set the status of the SSR response being built (no-op on the client).
/// Callable only where the answer cannot have been streamed yet — which is
/// why the data-driven routes run `SsrMode::Async`.
#[cfg(feature = "ssr")]
pub(crate) fn set_response_status(status: axum::http::StatusCode) {
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.set_status(status);
    }
}

/// Fallback for unmatched routes, with a real 404 status: a 200 here would
/// have crawlers indexing every mistyped URL as its own page ("soft 404").
#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    set_response_status(axum::http::StatusCode::NOT_FOUND);

    view! {
        <Title text="Page not found"/>
        "Page not found."
    }
}

#[component]
fn FontsPrefetch(fonts: &'static [&'static str]) -> impl IntoView {
    view! {
        {fonts.iter().map(|font_file| {
            view! {
                <Link rel="preload"
                href=format!("{}/fonts/{}", BUCKET_URL, font_file) as_="font"
                type_="font/woff2"
                crossorigin="anonymous"/>
            }
        }).collect_view()}
    }
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

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
