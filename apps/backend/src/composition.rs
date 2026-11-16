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
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::configuration::DatabaseConfig;
use crate::dsql::{Dsql, Refresher};
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

/// The pool, and — on DSQL — what keeps its token fresh.
pub struct Database {
    pub pool: PgPool,
    /// `Some` on DSQL: the web server refreshes the token before each request
    /// (`dsql::refresh`). `None` for a plain Postgres URL.
    pub refresher: Option<Arc<Refresher>>,
}

/// Connects, and nothing else. Migrations are not run here: on Lambda every
/// cold start would race to apply them, and DSQL refuses the migrator's
/// advisory lock (personal-infra G51). `ingest` runs them (`migrations::run`).
///
/// # Errors
///
/// An unreachable database, or on DSQL a token that cannot be minted. Fail
/// fast: a function that cannot reach its data must fail at init, not serve.
pub async fn connect(config: &DatabaseConfig) -> Result<Database, Box<dyn std::error::Error>> {
    match config {
        DatabaseConfig::Url(url) => {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(Duration::from_secs(5))
                .connect(url)
                .await?;
            tracing::info!("postgres pool ready");
            Ok(Database {
                pool,
                refresher: None,
            })
        }
        DatabaseConfig::Dsql {
            endpoint,
            user,
            schema,
        } => {
            let (pool, refresher) = Dsql::new(endpoint.clone(), user.clone(), schema.clone())
                .await?
                .connect()
                .await?;
            tracing::info!(user = %user, schema = %schema, "DSQL pool ready");
            Ok(Database {
                pool,
                refresher: Some(refresher),
            })
        }
    }
}

/// Wire the use cases over an existing pool. Pure construction, no IO — so
/// tests can build a `Container` around a lazy pool that never connects.
pub fn wire(pool: PgPool) -> Container {
    let repository: Arc<dyn BlogRepository> = Arc::new(PostgresBlogRepository::new(pool));

    let blog = BlogUseCases {
        list_published: Arc::new(ListPublishedPosts::new(repository.clone())),
        get_by_slug: Arc::new(GetPostBySlug::new(repository.clone())),
        get_by_id: Arc::new(GetPostById::new(repository.clone())),
        get_markdown: Arc::new(GetPostMarkdown::new(repository.clone())),
        upsert: Arc::new(UpsertPost::new(repository.clone())),
        prune: Arc::new(PrunePosts::new(repository)),
    };

    Container { blog }
}
