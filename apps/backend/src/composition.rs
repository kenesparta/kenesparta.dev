//! Composition / wiring of the backend.
//!
//! Instantiates the adapters (DynamoDB) and injects them into each Bounded
//! Context's use cases. The `Container` is shared with the Leptos server
//! functions via context, so it replaces the old global singleton.

use std::sync::Arc;

use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use bc_blog::application::use_cases::{GetPostById, GetPostBySlug, ListPublishedPosts};
use bc_blog::domain::repository::BlogRepository;

use crate::configuration::Configuration;
use crate::persistence::blog_dynamodb::DynamoBlogRepository;

/// Dependency container: fully wired use cases. Cheap to clone (Arcs).
#[derive(Clone)]
pub struct Container {
    pub blog: BlogUseCases,
}

#[derive(Clone)]
pub struct BlogUseCases {
    pub list_published: Arc<ListPublishedPosts>,
    pub get_by_slug: Arc<GetPostBySlug>,
    pub get_by_id: Arc<GetPostById>,
}

pub async fn compose(config: &Configuration) -> Result<Container, Box<dyn std::error::Error>> {
    let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let client = Client::new(&aws_config);

    let repository: Arc<dyn BlogRepository> =
        Arc::new(DynamoBlogRepository::new(client, config.dynamodb_table.clone()));

    let blog = BlogUseCases {
        list_published: Arc::new(ListPublishedPosts::new(repository.clone())),
        get_by_slug: Arc::new(GetPostBySlug::new(repository.clone())),
        get_by_id: Arc::new(GetPostById::new(repository)),
    };

    Ok(Container { blog })
}
