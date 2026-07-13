use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn Tags(tags: Vec<String>) -> impl IntoView {
    if tags.is_empty() {
        return view! { <div></div> }.into_any();
    }

    view! {
        <div class="post-tags">
            {tags
            .iter()
            .map(|tag| {
                let tag = tag.clone();
                view! { <span class="tag">{tag}</span> }
            })
            .collect_view()}
        </div>
    }
    .into_any()
}
