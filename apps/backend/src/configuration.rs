//! Runtime configuration read from the environment.

#[derive(Debug, Clone)]
pub struct Configuration {
    /// DynamoDB table that stores the blog posts.
    pub dynamodb_table: String,
}

impl Configuration {
    pub fn from_env() -> Self {
        Self {
            dynamodb_table: std::env::var("DYNAMODB_TABLE_NAME")
                .unwrap_or_else(|_| "kenesparta-blog-posts".to_string()),
        }
    }
}
