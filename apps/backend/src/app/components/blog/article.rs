use crate::app::components::blog::tags::Tags;
use crate::app::components::blog::utils::published_date;
use crate::app::components::go_back::GoBack;
use bc_blog::application::dto::BlogPostDTO;
use leptos::prelude::*;
use leptos::{IntoView, component, view};

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

            <footer class="post-footer">
                <GoBack go_to="blog" text="All posts"/>
            </footer>
        </article>
    }
    .into_any()
}
