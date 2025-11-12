use crate::components::StickyNavBar;
use crate::constants::{BUCKET_URL, GLOBAL_FONTS, ICON_URL, META_DESCRIPTION, META_TITLE, SITE_URL};
use crate::pages::{About, BlogList, BlogPost, Experience, HomePage, Projects};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_location;
use leptos_router::{
    components::{Route, Router, Routes},
    path, StaticSegment,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text={META_TITLE}/>
        <Link rel="icon" type_="image/x-icon" href={ICON_URL}/>

        <Link rel="dns-prefetch" href={BUCKET_URL}/>
        <Link rel="preconnect" href={BUCKET_URL} crossorigin="anonymous"/>
        <Link rel="preload" href="/pkg/kenespartadev.css" as_="style"/>
        <Stylesheet id="leptos" href="/pkg/kenespartadev.css"/>

        <FontsPrefetch fonts=GLOBAL_FONTS/>

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
fn OgProperties() -> impl IntoView {
    view! {
        <Meta property="og:url" content={SITE_URL}/>
        <Meta property="og:type" content="website"/>
        <Meta property="og:title" content={META_TITLE}/>
        <Meta property="og:description" content={META_DESCRIPTION}/>
        <Meta property="og:image" content={ICON_URL}/>

        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Meta name="description" content={META_DESCRIPTION}/>
        <Meta name="googlebot" content="index,follow,snippet,archive"/>
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
