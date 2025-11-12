#[cfg(feature = "ssr")]
use crate::blog::service::BlogService;
use crate::blog::{BlogPostDTO, BlogPostSummaryDTO};
use leptos::prelude::*;

#[server(GetPublishedPosts, "/api")]
pub async fn get_published_posts(
    limit: Option<i32>,
) -> Result<Vec<BlogPostSummaryDTO>, ServerFnError> {
    let service = BlogService::get().await;
    let posts = service
        .list_published_posts(limit.unwrap_or(10))
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(posts)
}

#[server(GetPostBySlug, "/api")]
pub async fn get_post_by_slug(slug: String) -> Result<Option<BlogPostDTO>, ServerFnError> {
    let service = BlogService::get().await;
    let post = service
        .get_post_by_slug(&slug)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(post)
}

// #[server(GetPostById, "/api")]
// pub async fn get_post_by_id(post_id: String) -> Result<Option<BlogPostDTO>, ServerFnError> {
//     let service = BlogService::get().await;
//     let post = service
//         .get_post_by_id(&post_id)
//         .await
//         .map_err(|e| ServerFnError::new(e))?;
//     Ok(post)
// }
