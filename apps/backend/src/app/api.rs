//! Leptos server functions (the HTTP edge of the blog BC).
//!
//! The function bodies run only on the server; there they pull the dependency
//! `Container` from the reactive context and delegate to the use cases. On the
//! client the `#[server]` macro replaces the body with a network call.

use bc_blog::application::dto::{BlogPostDTO, BlogPostSummaryDTO};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::composition::Container;

#[server(GetPublishedPosts, "/api")]
pub async fn get_published_posts(
    limit: Option<i32>,
) -> Result<Vec<BlogPostSummaryDTO>, ServerFnError> {
    let container = expect_context::<Container>();
    container
        .blog
        .list_published
        .execute(limit.unwrap_or(10))
        .await
        .map_err(ServerFnError::new)
}

#[server(GetPostBySlug, "/api")]
pub async fn get_post_by_slug(slug: String) -> Result<Option<BlogPostDTO>, ServerFnError> {
    let container = expect_context::<Container>();
    container
        .blog
        .get_by_slug
        .execute(&slug)
        .await
        .map_err(ServerFnError::new)
}
