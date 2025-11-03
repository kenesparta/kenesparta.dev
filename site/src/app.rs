use crate::components::StickyNavBar;
use crate::constants::{BUCKET_URL, ICON_URL, META_DESCRIPTION};
use crate::pages::{About, BlogList, BlogPost, Experience, HomePage, Projects};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Meta, MetaTags, Stylesheet, Title};
use leptos_router::hooks::use_location;
use leptos_router::{
    components::{Route, Router, Routes},
    path, StaticSegment,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/kenespartadev.css"/>

        <Title text="Ken Esparta"/>

        <Link rel="icon" type_="image/x-icon" href={ICON_URL}/>
        <Link rel="preconnect" href={BUCKET_URL} crossorigin="anonymous"/>

        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Meta name="description" content={META_DESCRIPTION}/>
        <Meta name="googlebot" content="index,follow,snippet,archive"/>

        <OgProperties/>

        <Router>
            <div class="app">
                <main>
                    <ConditionalNavBar/>
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=StaticSegment("") view=HomePage/>
                        <Route path=StaticSegment("/about") view=About/>
                        <Route path=StaticSegment("/blog") view=BlogList/>
                        <Route path=path!("/blog/:slug") view=BlogPost/>
                        <Route path=StaticSegment("/experience") view=Experience/>
                        <Route path=StaticSegment("/projects") view=Projects/>
                    </Routes>
                </main>
            </div>
        </Router>
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

#[component]
fn OgProperties() -> impl IntoView {
    view! {
        <Meta property="og:url" content="https://kenesparta.dev/"/>
        <Meta property="og:type" content="website"/>
        <Meta property="og:title" content="Ken Esparta - Software Engineer"/>
        <Meta property="og:description" content={META_DESCRIPTION}/>
        <Meta property="og:image" content={ICON_URL}/>
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
