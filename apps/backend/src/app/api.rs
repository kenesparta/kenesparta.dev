//! Leptos server functions (the HTTP edge of the blog BC).
//!
//! The function bodies run only on the server; there they pull the dependency
//! `Container` from the reactive context and delegate to the use cases. On the
//! client the `#[server]` macro replaces the body with a network call.

use bc_blog::application::dto::{BlogPostDTO, BlogPostSummaryDTO};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::composition::Container;

/// Pull the DI container out of the reactive context.
///
/// The page router installs it (`leptos_routes_with_context`), but the error
/// fallback (`file_and_error_handler`) does not. Leptos's SSR matcher treats a
/// trailing slash as a match, so `/blog/` resolves to this data route yet is
/// served through that context-less fallback (the generated Axum route is the
/// slash-free `/blog`). `expect_context` would panic there, letting an
/// unauthenticated `GET /blog/` crash a worker thread; returning an error
/// degrades that request instead of taking the task down.
#[cfg(feature = "ssr")]
fn container() -> Result<Container, ServerFnError> {
    use_context::<Container>().ok_or_else(|| {
        tracing::error!("dependency container missing from request context");
        ServerFnError::new("service unavailable")
    })
}

#[server(GetPublishedPosts, "/api")]
pub async fn get_published_posts(
    limit: Option<i32>,
) -> Result<Vec<BlogPostSummaryDTO>, ServerFnError> {
    container()?
        .blog
        .list_published
        .execute(limit.unwrap_or(10))
        .await
        .inspect_err(|error| tracing::error!(error = %error, "listing published posts failed"))
        .map_err(ServerFnError::new)
}

#[server(GetPostBySlug, "/api")]
pub async fn get_post_by_slug(slug: String) -> Result<Option<BlogPostDTO>, ServerFnError> {
    container()?
        .blog
        .get_by_slug
        .execute(&slug)
        .await
        .inspect_err(|error| tracing::error!(error = %error, slug = %slug, "loading post failed"))
        .map_err(ServerFnError::new)
}
