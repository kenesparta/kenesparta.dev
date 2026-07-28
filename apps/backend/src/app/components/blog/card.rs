use crate::app::components::blog::tags::Tags;
use crate::app::components::blog::utils::{published_date, rfc3339};
use bc_blog::application::dto::BlogPostSummaryDTO;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_router::components::A;

#[component]
pub fn BlogPostList(posts: Vec<BlogPostSummaryDTO>) -> impl IntoView {
    if posts.is_empty() {
        return view! {
            <div class="no-posts">
                <p>"No posts yet. Check back soon!"</p>
            </div>
        }
        .into_any();
    };

    view! {
        <div class="blog-posts">
            <For
                each=move || posts.clone()
                key=|post| post.post_id.clone()
                children=move |post: BlogPostSummaryDTO| {
                    view! { <BlogPostCard post=post/> }
                }
            />
        </div>
    }
    .into_any()
}

#[component]
pub fn BlogPostCard(post: BlogPostSummaryDTO) -> impl IntoView {
    let date_text = published_date(post.published_at);
    let published_iso = rfc3339(post.published_at);

    view! {
        <article class="blog-post-card">
            <A href=format!("/blog/{}", post.slug) attr:class="post-link">
                <h2 class="post-title">{post.title}</h2>
            </A>
            <div class="post-meta">
                <span class="post-author">{post.author}</span>
                <time class="post-date" datetime=published_iso>{date_text}</time>
            </div>
            <p class="post-summary">{post.summary}</p>
            <Tags tags=post.tags/>
        </article>
    }
}
