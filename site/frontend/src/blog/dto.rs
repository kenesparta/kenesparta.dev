use kenespartadev_core::blog::entity::{BlogPost, BlogPostSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPostDTO {
    pub post_id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub summary: String,
    pub author: String,
    pub tags: Vec<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub published_at: Option<i64>,
}

impl From<BlogPost> for BlogPostDTO {
    fn from(post: BlogPost) -> Self {
        Self {
            post_id: post.post_id,
            title: post.title,
            slug: post.slug,
            content: post.content,
            summary: post.summary,
            author: post.author,
            tags: post.tags,
            status: post.status.as_str().to_string(),
            created_at: post.created_at,
            updated_at: post.updated_at,
            published_at: post.published_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPostSummaryDTO {
    pub post_id: String,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub author: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub published_at: Option<i64>,
}

impl From<BlogPostSummary> for BlogPostSummaryDTO {
    fn from(summary: BlogPostSummary) -> Self {
        Self {
            post_id: summary.post_id,
            title: summary.title,
            slug: summary.slug,
            summary: summary.summary,
            author: summary.author,
            tags: summary.tags,
            created_at: summary.created_at,
            published_at: summary.published_at,
        }
    }
}
