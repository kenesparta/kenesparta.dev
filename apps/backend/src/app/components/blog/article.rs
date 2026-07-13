use crate::blog::BlogPostDTO;
use crate::components::blog::tags::Tags;
use crate::components::blog::utils::published_date;
use crate::components::go_back::GoBack;
use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn Article(post: BlogPostDTO) -> impl IntoView {
    let published_date = published_date(post.published_at);

    view! {
        <article class="blog-post">
            <header class="post-header">
                <GoBack go_to="blog" text="Back to Blog"/>
                <h1 class="post-title">{post.title}</h1>
                <div class="post-meta">
                    <span class="post-author">{post.author}</span>
                    <span class="post-date">{published_date}</span>
                </div>

                <Tags tags=post.tags/>
            </header>

            <div class="post-content" inner_html=post.content></div>
        </article>
    }
    .into_any()
}
