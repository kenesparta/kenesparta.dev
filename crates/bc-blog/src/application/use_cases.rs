//! Use cases of the blog BC.
//!
//! Each use case orchestrates the persistence port and maps the domain model
//! to the wire DTOs. They depend on the `BlogRepository` trait, never on a
//! concrete adapter.

use std::sync::Arc;

use shared_kernel::{Datetime, PostUuid};
use thiserror::Error;

use super::dto::{BlogPostDTO, BlogPostSummaryDTO, PostMarkdownDTO};
use crate::domain::model::{BlogPost, BlogPostSummary, PostStatus};
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
        Ok(self
            .repository
            .find_by_slug(slug)
            .await?
            // Only published posts are public. `find_by_slug` is a general
            // lookup by key (it also backs the `.md` variant, which gates on
            // status in its handler), so the page's visibility policy lives
            // here: a draft slug must read as "not found", not render its body
            // for anyone who guesses the slug.
            .filter(|post| post.status == PostStatus::Published)
            .map(BlogPostDTO::from))
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

/// Fetch a post's Markdown source by slug (the `.md` crawler variants).
///
/// Separate from [`GetPostBySlug`] so the raw source is only ever loaded for
/// the endpoint that actually serves it.
pub struct GetPostMarkdown {
    repository: Arc<dyn BlogRepository>,
}

impl GetPostMarkdown {
    pub fn new(repository: Arc<dyn BlogRepository>) -> Self {
        Self { repository }
    }

    /// # Errors
    ///
    /// [`UseCaseError::Repository`] if the persistence port fails.
    pub async fn execute(&self, slug: &str) -> Result<Option<PostMarkdownDTO>, UseCaseError> {
        Ok(self
            .repository
            .find_by_slug(slug)
            .await?
            .map(PostMarkdownDTO::from))
    }
}

/// Command to create or replace a post, keyed by slug. Both renditions are
/// stored: `content_html` is what the pages display, `content_md` the authored
/// source the `.md` crawler variants serve.
#[derive(Debug, Clone)]
pub struct UpsertPostCommand {
    pub title: String,
    pub slug: String,
    pub content_html: String,
    pub content_md: String,
    pub summary: String,
    pub author: String,
    pub tags: Vec<String>,
    pub published: bool,
    /// Authoring date, Unix seconds.
    pub date: i64,
}

/// Create or update a post by slug (the ingest write path).
pub struct UpsertPost {
    repository: Arc<dyn BlogRepository>,
}

impl UpsertPost {
    pub fn new(repository: Arc<dyn BlogRepository>) -> Self {
        Self { repository }
    }

    /// # Errors
    ///
    /// [`UseCaseError::Repository`] if the persistence port fails.
    pub async fn execute(&self, cmd: UpsertPostCommand) -> Result<(), UseCaseError> {
        let status = if cmd.published {
            PostStatus::Published
        } else {
            PostStatus::Draft
        };
        // `post_id` and `created_at` only apply on first insert; on a slug
        // conflict the repository preserves the stored values.
        let post = BlogPost {
            post_id: PostUuid::new(),
            title: cmd.title,
            slug: cmd.slug,
            content: cmd.content_html,
            content_md: cmd.content_md,
            summary: cmd.summary,
            author: cmd.author,
            tags: cmd.tags,
            published_at: (status == PostStatus::Published).then_some(cmd.date),
            status,
            created_at: cmd.date,
            updated_at: Datetime::now(),
        };
        Ok(self.repository.upsert(&post).await?)
    }
}

/// Delete posts whose source file no longer exists (the ingest `--prune`
/// write path): the database mirrors the content directory.
pub struct PrunePosts {
    repository: Arc<dyn BlogRepository>,
}

impl PrunePosts {
    pub fn new(repository: Arc<dyn BlogRepository>) -> Self {
        Self { repository }
    }

    /// Deletes every stored post whose slug is not in `keep`; returns the
    /// deleted slugs. An empty `keep` deletes ALL posts — the caller decides
    /// whether that is intentional.
    ///
    /// # Errors
    ///
    /// [`UseCaseError::Repository`] if the persistence port fails.
    pub async fn execute(&self, keep: &[String]) -> Result<Vec<String>, UseCaseError> {
        Ok(self.repository.delete_not_in(keep).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::BlogPost;

    /// In-memory `BlogRepository` for the application-layer tests. It returns
    /// posts verbatim, drafts included — exactly like the real adapter's
    /// `find_by_slug`, which is what makes the visibility policy the use case's
    /// job to enforce.
    struct InMemoryRepo {
        posts: Vec<BlogPost>,
    }

    #[async_trait::async_trait]
    impl BlogRepository for InMemoryRepo {
        async fn list_published(&self, limit: i32) -> Result<Vec<BlogPost>, RepositoryError> {
            Ok(self
                .posts
                .iter()
                .filter(|post| post.status == PostStatus::Published)
                .take(limit.max(0) as usize)
                .cloned()
                .collect())
        }

        async fn find_by_slug(&self, slug: &str) -> Result<Option<BlogPost>, RepositoryError> {
            Ok(self.posts.iter().find(|post| post.slug == slug).cloned())
        }

        async fn find_by_id(&self, post_id: &str) -> Result<Option<BlogPost>, RepositoryError> {
            Ok(self.posts.iter().find(|post| post.post_id == post_id).cloned())
        }

        async fn upsert(&self, _post: &BlogPost) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn delete_not_in(&self, _keep: &[String]) -> Result<Vec<String>, RepositoryError> {
            Ok(Vec::new())
        }
    }

    /// Drive a future to completion without a runtime: the in-memory repository
    /// never yields `Pending`, so the first poll is always `Ready`. Keeps this
    /// crate free of a runtime dependency (`tokio` is forbidden here).
    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        use std::task::{Context, Poll, Waker};
        let mut future = std::pin::pin!(future);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
                return output;
            }
        }
    }

    fn post(slug: &str, status: PostStatus) -> BlogPost {
        let mut post = BlogPost::new(
            "Title".to_owned(),
            slug.to_owned(),
            "<p>body</p>".to_owned(),
            "# body".to_owned(),
            "summary".to_owned(),
            "ken".to_owned(),
            Vec::new(),
        );
        post.status = status;
        post
    }

    fn use_case(posts: Vec<BlogPost>) -> GetPostBySlug {
        GetPostBySlug::new(Arc::new(InMemoryRepo { posts }))
    }

    // Regression: a draft must never be served by slug (it used to render its
    // full body at /blog/<slug> — only the .md variant was guarded).
    #[test]
    fn get_by_slug_hides_drafts() {
        let uc = use_case(vec![post("secret", PostStatus::Draft)]);
        let result = block_on(uc.execute("secret")).expect("use case ok");
        assert!(result.is_none(), "a draft slug must read as not found");
    }

    #[test]
    fn get_by_slug_returns_published() {
        let uc = use_case(vec![post("hello", PostStatus::Published)]);
        let result = block_on(uc.execute("hello")).expect("use case ok");
        assert_eq!(result.map(|dto| dto.slug).as_deref(), Some("hello"));
    }

    #[test]
    fn get_by_slug_missing_is_none() {
        let uc = use_case(vec![post("hello", PostStatus::Published)]);
        let result = block_on(uc.execute("nope")).expect("use case ok");
        assert!(result.is_none());
    }
}
