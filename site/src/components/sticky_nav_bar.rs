use crate::constants::BUCKET_URL;
use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn StickyNavBar() -> impl IntoView {
    let img_url = format!("{}/img/icon.svg", BUCKET_URL);
    view! {
        <nav class="sticky-nav-bar">
            <a href="/" class="sticky-nav-bar__brand">
                <img src={img_url} alt="" class="sticky-nav-bar__img"/>
                <div class="sticky-nav-bar__name">
                    <span class="delius-swash-caps sticky-nav-bar__name-first">"Ken"</span>
                    <span class="delius-swash-caps sticky-nav-bar__name-last">"Esparta"</span>
                </div>
            </a>
            // <HeaderLinks/>
        </nav>
    }
}
