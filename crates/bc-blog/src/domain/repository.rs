//! Persistence port of the blog BC.
//!
//! The trait is defined here (in the domain); its concrete implementation
//! (PostgreSQL) lives in the binary crate.

use async_trait::async_trait;
use thiserror::Error;

use super::model::BlogPost;

#[async_trait]
pub trait BlogRepository: Send + Sync {
    /// Most recent published posts first, capped at `limit`.
    async fn list_published(&self, limit: i32) -> Result<Vec<BlogPost>, RepositoryError>;

    /// A single post by its slug, if any.
    async fn find_by_slug(&self, slug: &str) -> Result<Option<BlogPost>, RepositoryError>;

    /// A single post by its id, if any.
    async fn find_by_id(&self, post_id: &str) -> Result<Option<BlogPost>, RepositoryError>;

    /// Insert `post`, or — when a post with the same slug already exists —
    /// update it in place, preserving the stored `post_id` and `created_at`
    /// (stable ids and URLs across re-ingests).
    async fn upsert(&self, post: &BlogPost) -> Result<(), RepositoryError>;
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
