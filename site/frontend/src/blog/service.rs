use crate::blog::repository::BlogRepository;
use crate::blog::{BlogPostDTO, BlogPostSummaryDTO};
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use kenespartadev_core::blog::entity::BlogPostSummary;
use std::sync::OnceLock;

static BLOG_SERVICE: OnceLock<BlogService> = OnceLock::new();

pub struct BlogService {
    repository: BlogRepository,
}

impl BlogService {
    async fn new() -> Self {
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
        let client = Client::new(&config);

        let table_name = std::env::var("DYNAMODB_TABLE_NAME")
            .unwrap_or_else(|_| "kenesparta-blog-posts".to_string());

        let repository = BlogRepository::new(client, table_name);

        Self { repository }
    }

    pub async fn get() -> &'static BlogService {
        BLOG_SERVICE.get_or_init(|| {
            tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(Self::new()))
        })
    }

    // pub async fn create_post(&self, post: BlogPost) -> Result<BlogPost, String> {
    //     self.repository.create_post(&post).await?;
    //     Ok(post)
    // }

    pub async fn get_post_by_id(&self, post_id: &str) -> Result<Option<BlogPostDTO>, String> {
        match self.repository.get_post_by_id(post_id).await? {
            Some(post) => Ok(Some(BlogPostDTO::from(post))),
            None => Err("Post not found".to_string()),
        }
    }

    pub async fn get_post_by_slug(&self, slug: &str) -> Result<Option<BlogPostDTO>, String> {
        match self.repository.get_post_by_slug(slug).await? {
            Some(post) => Ok(Some(BlogPostDTO::from(post))),
            None => Err("Post not found".to_string()),
        }
    }

    pub async fn list_published_posts(
        &self,
        limit: i32,
    ) -> Result<Vec<BlogPostSummaryDTO>, String> {
        let posts = self.repository.list_published_posts(limit).await?;
        Ok(posts
            .into_iter()
            .map(|p| BlogPostSummaryDTO::from(BlogPostSummary::from(p)))
            .collect())
    }

    // pub async fn update_post(&self, post: BlogPost) -> Result<BlogPost, String> {
    //     self.repository.update_post(&post).await?;
    //     Ok(post)
    // }

    // pub async fn delete_post(&self, post_id: &str, created_at: i64) -> Result<(), String> {
    //     self.repository.delete_post(post_id, created_at).await
    // }
}
