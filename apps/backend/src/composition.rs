//! Composition / wiring of the backend.
//!
//! Instantiates the adapters (PostgreSQL) and injects them into each Bounded
//! Context's use cases. The `Container` is shared with the Leptos server
//! functions via context, so it replaces the old global singleton.

use std::sync::Arc;
use std::time::Duration;

use bc_blog::application::use_cases::{
    GetPostById, GetPostBySlug, GetPostMarkdown, ListPublishedPosts, PrunePosts, UpsertPost,
};
use bc_blog::domain::repository::BlogRepository;
use sqlx::postgres::PgPoolOptions;

use crate::configuration::Configuration;
use crate::persistence::blog_postgres::PostgresBlogRepository;

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
    /// Crawler path — only the `/blog/<slug>.md` endpoint calls it.
    pub get_markdown: Arc<GetPostMarkdown>,
    /// Write path — only the ingest bin calls it; the web server never does.
    pub upsert: Arc<UpsertPost>,
    /// Write path — only the ingest bin calls it (`--prune`).
    pub prune: Arc<PrunePosts>,
}

pub async fn compose(config: &Configuration) -> Result<Container, Box<dyn std::error::Error>> {
    // nano container (0.25 vCPU / 512 MB), single instance: 5 connections is
    // ample and leaves DB headroom for the ingest CLI and psql.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    // Embedded migrations (apps/backend/migrations). Fail fast: an
    // unreachable/unmigratable DB must abort the deployment, not limp along.
    sqlx::migrate!().run(&pool).await?;

    let repository: Arc<dyn BlogRepository> = Arc::new(PostgresBlogRepository::new(pool));

    let blog = BlogUseCases {
        list_published: Arc::new(ListPublishedPosts::new(repository.clone())),
        get_by_slug: Arc::new(GetPostBySlug::new(repository.clone())),
        get_by_id: Arc::new(GetPostById::new(repository.clone())),
        get_markdown: Arc::new(GetPostMarkdown::new(repository.clone())),
        upsert: Arc::new(UpsertPost::new(repository.clone())),
        prune: Arc::new(PrunePosts::new(repository)),
    };

    Ok(Container { blog })
}
