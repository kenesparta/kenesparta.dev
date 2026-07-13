//! Use cases of the blog BC.
//!
//! Each use case orchestrates the persistence port and maps the domain model
//! to the wire DTOs. They depend on the `BlogRepository` trait, never on a
//! concrete adapter.

use std::sync::Arc;

use thiserror::Error;

use super::dto::{BlogPostDTO, BlogPostSummaryDTO};
use crate::domain::model::BlogPostSummary;
use crate::domain::repository::{BlogRepository, RepositoryError};

#[derive(Debug, Error)]
pub enum UseCaseError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error("post not found")]
    NotFound,
}

/// List the most recent published posts.
pub struct ListPublishedPosts {
    repository: Arc<dyn BlogRepository>,
}

impl ListPublishedPosts {
    pub fn new(repository: Arc<dyn BlogRepository>) -> Self {
        Self { repository }
    }

    /// # Errors
    ///
    /// [`UseCaseError::Repository`] if the persistence port fails.
    pub async fn execute(&self, limit: i32) -> Result<Vec<BlogPostSummaryDTO>, UseCaseError> {
        let posts = self.repository.list_published(limit).await?;
        Ok(posts
            .into_iter()
            .map(|post| BlogPostSummaryDTO::from(BlogPostSummary::from(post)))
            .collect())
    }
}

/// Fetch a single post by slug.
pub struct GetPostBySlug {
    repository: Arc<dyn BlogRepository>,
}

impl GetPostBySlug {
    pub fn new(repository: Arc<dyn BlogRepository>) -> Self {
        Self { repository }
    }

    /// # Errors
    ///
    /// [`UseCaseError::Repository`] if the persistence port fails.
    pub async fn execute(&self, slug: &str) -> Result<Option<BlogPostDTO>, UseCaseError> {
        Ok(self.repository.find_by_slug(slug).await?.map(BlogPostDTO::from))
    }
}

/// Fetch a single post by id.
pub struct GetPostById {
    repository: Arc<dyn BlogRepository>,
}

impl GetPostById {
    pub fn new(repository: Arc<dyn BlogRepository>) -> Self {
        Self { repository }
    }

    /// # Errors
    ///
    /// - [`UseCaseError::NotFound`] if no post matches the id.
    /// - [`UseCaseError::Repository`] if the persistence port fails.
    pub async fn execute(&self, post_id: &str) -> Result<BlogPostDTO, UseCaseError> {
        self.repository
            .find_by_id(post_id)
            .await?
            .map(BlogPostDTO::from)
            .ok_or(UseCaseError::NotFound)
    }
}
