//! Domain model of the blog BC.

use std::fmt::Display;

use shared_kernel::{Datetime, PostUuid};

// ============================================================================
// PostStatus
// ============================================================================

/// Publication status of a post.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostStatus {
    Draft,
    Published,
}

impl PostStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PostStatus::Draft => "draft",
            PostStatus::Published => "published",
        }
    }
}

impl Display for PostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// BlogPost (aggregate root)
// ============================================================================

#[derive(Debug, Clone)]
pub struct BlogPost {
    pub post_id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub summary: String,
    pub author: String,
    pub tags: Vec<String>,
    pub status: PostStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub published_at: Option<i64>,
}

impl BlogPost {
    pub fn new(
        title: String,
        slug: String,
        content: String,
        summary: String,
        author: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            post_id: PostUuid::new(),
            title,
            slug,
            content,
            summary,
            author,
            tags,
            status: PostStatus::Draft,
            created_at: Datetime::now(),
            updated_at: Datetime::now(),
            published_at: None,
        }
    }
}

// ============================================================================
// BlogPostSummary (listing projection)
// ============================================================================

#[derive(Debug, Clone)]
pub struct BlogPostSummary {
    pub post_id: String,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub author: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub published_at: Option<i64>,
}

impl From<BlogPost> for BlogPostSummary {
    fn from(post: BlogPost) -> Self {
        Self {
            post_id: post.post_id,
            title: post.title,
            slug: post.slug,
            summary: post.summary,
            author: post.author,
            tags: post.tags,
            created_at: post.created_at,
            published_at: post.published_at,
        }
    }
}
