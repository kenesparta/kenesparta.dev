use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PostStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "published")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let now = Utc::now().timestamp();
        Self {
            post_id: Uuid::new_v4().to_string(),
            title,
            slug,
            content,
            summary,
            author,
            tags,
            status: PostStatus::Draft,
            created_at: now,
            updated_at: now,
            published_at: None,
        }
    }

    pub fn publish(&mut self) {
        self.status = PostStatus::Published;
        self.published_at = Some(Utc::now().timestamp());
        self.updated_at = Utc::now().timestamp();
    }

    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_default()
    }

    pub fn updated_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.updated_at, 0).unwrap_or_default()
    }

    pub fn published_at_datetime(&self) -> Option<DateTime<Utc>> {
        self.published_at
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
    }
}

// Lightweight version for listing posts
#[derive(Debug, Clone, Serialize, Deserialize)]
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
