use crate::blog::blog::{BlogPost, PostStatus};
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use std::collections::HashMap;

pub struct BlogRepository {
    client: Client,
    table_name: String,
}

impl BlogRepository {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    pub async fn create_post(&self, post: &BlogPost) -> Result<(), String> {
        let mut item = HashMap::new();
        item.insert(
            "post_id".to_string(),
            AttributeValue::S(post.post_id.clone()),
        );
        item.insert("title".to_string(), AttributeValue::S(post.title.clone()));
        item.insert("slug".to_string(), AttributeValue::S(post.slug.clone()));
        item.insert(
            "content".to_string(),
            AttributeValue::S(post.content.clone()),
        );
        item.insert(
            "summary".to_string(),
            AttributeValue::S(post.summary.clone()),
        );
        item.insert("author".to_string(), AttributeValue::S(post.author.clone()));
        item.insert(
            "tags".to_string(),
            AttributeValue::L(
                post.tags
                    .iter()
                    .map(|t| AttributeValue::S(t.clone()))
                    .collect(),
            ),
        );
        item.insert(
            "status".to_string(),
            AttributeValue::S(post.status.as_str().to_string()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::N(post.created_at.to_string()),
        );
        item.insert(
            "updated_at".to_string(),
            AttributeValue::N(post.updated_at.to_string()),
        );
        if let Some(published_at) = post.published_at {
            item.insert(
                "published_at".to_string(),
                AttributeValue::N(published_at.to_string()),
            );
        }

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| format!("Failed to create post: {}", e))?;

        Ok(())
    }

    pub async fn get_post_by_id(&self, post_id: &str) -> Result<Option<BlogPost>, String> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("post_id", AttributeValue::S(post_id.to_string()))
            .send()
            .await
            .map_err(|e| format!("Failed to get post: {}", e))?;

        match result.item {
            Some(item) => Ok(Some(self.item_to_blog_post(item)?)),
            None => Ok(None),
        }
    }

    pub async fn get_post_by_slug(&self, slug: &str) -> Result<Option<BlogPost>, String> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("slug = :slug")
            .expression_attribute_values(":slug", AttributeValue::S(slug.to_string()))
            .limit(1)
            .send()
            .await
            .map_err(|e| format!("Failed to get post by slug: {}", e))?;

        if let Some(items) = result.items {
            if let Some(item) = items.first() {
                return Ok(Some(self.item_to_blog_post(item.clone())?));
            }
        }

        Ok(None)
    }

    pub async fn list_published_posts(&self, limit: i32) -> Result<Vec<BlogPost>, String> {
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
            .map_err(|e| format!("Failed to list posts: {}", e))?;

        let mut posts = Vec::new();
        if let Some(items) = result.items {
            for item in items {
                posts.push(self.item_to_blog_post(item)?);
            }
        }

        Ok(posts)
    }

    pub async fn list_all_posts(&self) -> Result<Vec<BlogPost>, String> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list all posts: {}", e))?;

        let mut posts = Vec::new();
        if let Some(items) = result.items {
            for item in items {
                posts.push(self.item_to_blog_post(item)?);
            }
        }

        // Sort by created_at descending (newest first)
        posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(posts)
    }

    pub async fn update_post(&self, post: &BlogPost) -> Result<(), String> {
        let mut item = HashMap::new();
        item.insert(
            "post_id".to_string(),
            AttributeValue::S(post.post_id.clone()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::N(post.created_at.to_string()),
        );
        item.insert("title".to_string(), AttributeValue::S(post.title.clone()));
        item.insert("slug".to_string(), AttributeValue::S(post.slug.clone()));
        item.insert(
            "content".to_string(),
            AttributeValue::S(post.content.clone()),
        );
        item.insert(
            "summary".to_string(),
            AttributeValue::S(post.summary.clone()),
        );
        item.insert("author".to_string(), AttributeValue::S(post.author.clone()));
        item.insert(
            "tags".to_string(),
            AttributeValue::L(
                post.tags
                    .iter()
                    .map(|t| AttributeValue::S(t.clone()))
                    .collect(),
            ),
        );
        item.insert(
            "status".to_string(),
            AttributeValue::S(post.status.as_str().to_string()),
        );
        item.insert(
            "updated_at".to_string(),
            AttributeValue::N(post.updated_at.to_string()),
        );
        if let Some(published_at) = post.published_at {
            item.insert(
                "published_at".to_string(),
                AttributeValue::N(published_at.to_string()),
            );
        }

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| format!("Failed to update post: {}", e))?;

        Ok(())
    }

    pub async fn delete_post(&self, post_id: &str, created_at: i64) -> Result<(), String> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("post_id", AttributeValue::S(post_id.to_string()))
            .key("created_at", AttributeValue::N(created_at.to_string()))
            .send()
            .await
            .map_err(|e| format!("Failed to delete post: {}", e))?;

        Ok(())
    }

    fn item_to_blog_post(&self, item: HashMap<String, AttributeValue>) -> Result<BlogPost, String> {
        let post_id = item
            .get("post_id")
            .and_then(|v| v.as_s().ok())
            .ok_or("Missing post_id")?
            .clone();

        let title = item
            .get("title")
            .and_then(|v| v.as_s().ok())
            .ok_or("Missing title")?
            .clone();

        let slug = item
            .get("slug")
            .and_then(|v| v.as_s().ok())
            .ok_or("Missing slug")?
            .clone();

        let content = item
            .get("content")
            .and_then(|v| v.as_s().ok())
            .ok_or("Missing content")?
            .clone();

        let summary = item
            .get("summary")
            .and_then(|v| v.as_s().ok())
            .ok_or("Missing summary")?
            .clone();

        let author = item
            .get("author")
            .and_then(|v| v.as_s().ok())
            .ok_or("Missing author")?
            .clone();

        let tags = item
            .get("tags")
            .and_then(|v| v.as_l().ok())
            .map(|list| list.iter().filter_map(|v| v.as_s().ok().cloned()).collect())
            .unwrap_or_default();

        let status_str = item
            .get("status")
            .and_then(|v| v.as_s().ok())
            .ok_or("Missing status")?;

        let status = match status_str.as_str() {
            "draft" => PostStatus::Draft,
            "published" => PostStatus::Published,
            _ => PostStatus::Draft,
        };

        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<i64>().ok())
            .ok_or("Missing or invalid created_at")?;

        let updated_at = item
            .get("updated_at")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<i64>().ok())
            .ok_or("Missing or invalid updated_at")?;

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
}
