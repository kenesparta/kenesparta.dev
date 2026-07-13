//! DynamoDB adapter for the blog BC.
//!
//! Implements the `BlogRepository` port defined in the domain. Maps AWS SDK
//! failures to `RepositoryError::Infrastructure` and translates DynamoDB items
//! into the domain `BlogPost`.

use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use bc_blog::domain::model::{BlogPost, PostStatus};
use bc_blog::domain::repository::{BlogRepository, RepositoryError};

pub struct DynamoBlogRepository {
    client: Client,
    table_name: String,
}

impl DynamoBlogRepository {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    fn item_to_blog_post(
        &self,
        item: HashMap<String, AttributeValue>,
    ) -> Result<BlogPost, RepositoryError> {
        let missing = |field: &str| RepositoryError::Infrastructure(format!("missing {field}"));

        let post_id = item
            .get("post_id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| missing("post_id"))?
            .clone();

        let title = item
            .get("title")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| missing("title"))?
            .clone();

        let slug = item
            .get("slug")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| missing("slug"))?
            .clone();

        let content = item
            .get("content")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| missing("content"))?
            .clone();

        let summary = item
            .get("summary")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| missing("summary"))?
            .clone();

        let author = item
            .get("author")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| missing("author"))?
            .clone();

        let tags = item
            .get("tags")
            .and_then(|v| v.as_l().ok())
            .map(|list| list.iter().filter_map(|v| v.as_s().ok().cloned()).collect())
            .unwrap_or_default();

        let status_str = item
            .get("status")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| missing("status"))?;

        let status = match status_str.as_str() {
            "published" => PostStatus::Published,
            _ => PostStatus::Draft,
        };

        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<i64>().ok())
            .ok_or_else(|| missing("created_at"))?;

        let updated_at = item
            .get("updated_at")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<i64>().ok())
            .ok_or_else(|| missing("updated_at"))?;

        let published_at = item
            .get("published_at")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<i64>().ok());

        Ok(BlogPost {
            post_id,
            title,
            slug,
            content,
            summary,
            author,
            tags,
            status,
            created_at,
            updated_at,
            published_at,
        })
    }

    /// Persist a post (create or overwrite). Not wired to a use case yet.
    #[allow(dead_code)]
    pub async fn update_post(&self, post: &BlogPost) -> Result<(), RepositoryError> {
        let mut item = HashMap::new();
        item.insert("post_id".to_string(), AttributeValue::S(post.post_id.clone()));
        item.insert("created_at".to_string(), AttributeValue::N(post.created_at.to_string()));
        item.insert("title".to_string(), AttributeValue::S(post.title.clone()));
        item.insert("slug".to_string(), AttributeValue::S(post.slug.clone()));
        item.insert("content".to_string(), AttributeValue::S(post.content.clone()));
        item.insert("summary".to_string(), AttributeValue::S(post.summary.clone()));
        item.insert("author".to_string(), AttributeValue::S(post.author.clone()));
        item.insert(
            "tags".to_string(),
            AttributeValue::L(post.tags.iter().map(|t| AttributeValue::S(t.clone())).collect()),
        );
        item.insert("status".to_string(), AttributeValue::S(post.status.as_str().to_string()));
        item.insert("updated_at".to_string(), AttributeValue::N(post.updated_at.to_string()));
        if let Some(published_at) = post.published_at {
            item.insert("published_at".to_string(), AttributeValue::N(published_at.to_string()));
        }

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| RepositoryError::Infrastructure(format!("failed to update post: {e}")))?;

        Ok(())
    }

    /// Delete a post by its composite key. Not wired to a use case yet.
    #[allow(dead_code)]
    pub async fn delete_post(&self, post_id: &str, created_at: i64) -> Result<(), RepositoryError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("post_id", AttributeValue::S(post_id.to_string()))
            .key("created_at", AttributeValue::N(created_at.to_string()))
            .send()
            .await
            .map_err(|e| RepositoryError::Infrastructure(format!("failed to delete post: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl BlogRepository for DynamoBlogRepository {
    async fn list_published(&self, limit: i32) -> Result<Vec<BlogPost>, RepositoryError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("StatusCreatedAtIndex")
            .key_condition_expression("#status = :status")
            .expression_attribute_names("#status", "status")
            .expression_attribute_values(":status", AttributeValue::S("published".to_string()))
            .scan_index_forward(false)
            .limit(limit)
            .send()
            .await
            .map_err(|e| RepositoryError::Infrastructure(format!("failed to list posts: {e}")))?;

        let mut posts = Vec::new();
        if let Some(items) = result.items {
            for item in items {
                posts.push(self.item_to_blog_post(item)?);
            }
        }

        Ok(posts)
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<BlogPost>, RepositoryError> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("slug = :slug")
            .expression_attribute_values(":slug", AttributeValue::S(slug.to_string()))
            .limit(1)
            .send()
            .await
            .map_err(|e| {
                RepositoryError::Infrastructure(format!("failed to get post by slug: {e}"))
            })?;

        if let Some(items) = result.items
            && let Some(item) = items.first()
        {
            return Ok(Some(self.item_to_blog_post(item.clone())?));
        }

        Ok(None)
    }

    async fn find_by_id(&self, post_id: &str) -> Result<Option<BlogPost>, RepositoryError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("post_id", AttributeValue::S(post_id.to_string()))
            .send()
            .await
            .map_err(|e| RepositoryError::Infrastructure(format!("failed to get post: {e}")))?;

        match result.item {
            Some(item) => Ok(Some(self.item_to_blog_post(item)?)),
            None => Ok(None),
        }
    }
}
