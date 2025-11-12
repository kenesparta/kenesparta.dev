use crate::blog::entity::post_status::PostStatus;
use crate::shared::{Datetime, PostUuid};

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

    // pub fn publish(&mut self) {
    //     self.status = PostStatus::Published;
    //     self.published_at = Some(Utc::now().timestamp());
    //     self.updated_at = Utc::now().timestamp();
    // }

    // pub fn created_at_datetime(&self) -> DateTime<Utc> {
    //     DateTime::from_timestamp(self.created_at, 0).unwrap_or_default()
    // }

    // pub fn updated_at_datetime(&self) -> DateTime<Utc> {
    //     DateTime::from_timestamp(self.updated_at, 0).unwrap_or_default()
    // }

    // pub fn published_at_datetime(&self) -> Option<DateTime<Utc>> {
    //     self.published_at
    //         .and_then(|ts| DateTime::from_timestamp(ts, 0))
    // }
}

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
