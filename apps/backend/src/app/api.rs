//! Leptos server functions (the HTTP edge of the blog BC).
//!
//! The function bodies run only on the server; there they pull the dependency
//! `Container` from the reactive context and delegate to the use cases. On the
//! client the `#[server]` macro replaces the body with a network call.

use bc_blog::application::dto::{BlogPostDTO, BlogPostSummaryDTO};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::composition::Container;

/// Pull the DI container out of the reactive context.
///
/// The page router installs it (`leptos_routes_with_context`), but the error
/// fallback (`file_and_error_handler`) does not. Leptos's SSR matcher treats a
/// trailing slash as a match, so `/blog/` resolves to this data route yet is
/// served through that context-less fallback (the generated Axum route is the
/// slash-free `/blog`). `expect_context` would panic there, letting an
/// unauthenticated `GET /blog/` crash a worker thread; returning an error
/// degrades that request instead of taking the task down.
#[cfg(feature = "ssr")]
fn container() -> Result<Container, ServerFnError> {
    use_context::<Container>().ok_or_else(|| {
        tracing::error!("dependency container missing from request context");
        ServerFnError::new(UNAVAILABLE)
    })
}

/// The only failure text a client ever receives. The real error — which for
/// a repository failure carries the SQLx/Postgres message — goes to the server
/// log and nowhere else (SECURITY.md SEC-005).
#[cfg(feature = "ssr")]
const UNAVAILABLE: &str = "service unavailable";

#[server(GetPublishedPosts, "/api")]
pub async fn get_published_posts(
    limit: Option<i32>,
) -> Result<Vec<BlogPostSummaryDTO>, ServerFnError> {
    container()?
        .blog
        .list_published
        // Public input (SECURITY.md SEC-012): whatever the client sends is
        // folded into [1, 100] — the blog will not reach 100 posts soon, and
        // the complete listings (sitemap, feed) are server-side callers of the
        // use case, not of this function.
        .execute(limit.unwrap_or(10).clamp(1, 100))
        .await
        .inspect_err(|error| tracing::error!(error = %error, "listing published posts failed"))
        .map_err(|_| ServerFnError::new(UNAVAILABLE))
}

#[server(GetPostBySlug, "/api")]
pub async fn get_post_by_slug(slug: String) -> Result<Option<BlogPostDTO>, ServerFnError> {
    container()?
        .blog
        .get_by_slug
        .execute(&slug)
        .await
        .inspect_err(|error| tracing::error!(error = %error, slug = %slug, "loading post failed"))
        .map_err(|_| ServerFnError::new(UNAVAILABLE))
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;
    use std::task::{Context, Poll, Waker};

    use async_trait::async_trait;
    use bc_blog::application::use_cases::{
        GetPostById, GetPostBySlug, GetPostMarkdown, ListPublishedPosts, PrunePosts, UpsertPost,
    };
    use bc_blog::domain::model::BlogPost;
    use bc_blog::domain::repository::{BlogRepository, RepositoryError};
    use leptos::prelude::provide_context;
    use leptos::reactive::owner::Owner;

    use super::{get_post_by_slug, get_published_posts};
    use crate::composition::{BlogUseCases, Container};

    /// What the Postgres adapter produces when the schema is missing — the
    /// kind of detail that must never reach a browser.
    const LEAKY: &str = "error returned from database: relation \"blog_posts\" does not exist";

    struct Broken;

    #[async_trait]
    impl BlogRepository for Broken {
        async fn list_published(&self, _: i32) -> Result<Vec<BlogPost>, RepositoryError> {
            Err(RepositoryError::Infrastructure(LEAKY.to_owned()))
        }
        async fn find_by_slug(&self, _: &str) -> Result<Option<BlogPost>, RepositoryError> {
            Err(RepositoryError::Infrastructure(LEAKY.to_owned()))
        }
        async fn find_by_id(&self, _: &str) -> Result<Option<BlogPost>, RepositoryError> {
            Err(RepositoryError::Infrastructure(LEAKY.to_owned()))
        }
        async fn upsert(&self, _: &BlogPost) -> Result<(), RepositoryError> {
            Err(RepositoryError::Infrastructure(LEAKY.to_owned()))
        }
        async fn delete_not_in(&self, _: &[String]) -> Result<Vec<String>, RepositoryError> {
            Err(RepositoryError::Infrastructure(LEAKY.to_owned()))
        }
    }

    /// Repository double for the clamp test (SEC-012): any limit outside
    /// [1, 100] reaching persistence means the boundary failed.
    struct LimitProbe;

    #[async_trait]
    impl BlogRepository for LimitProbe {
        async fn list_published(&self, limit: i32) -> Result<Vec<BlogPost>, RepositoryError> {
            assert!(
                (1..=100).contains(&limit),
                "unclamped client limit reached the repository: {limit}"
            );
            Ok(Vec::new())
        }
        async fn find_by_slug(&self, _: &str) -> Result<Option<BlogPost>, RepositoryError> {
            unreachable!("not part of the clamp test")
        }
        async fn find_by_id(&self, _: &str) -> Result<Option<BlogPost>, RepositoryError> {
            unreachable!("not part of the clamp test")
        }
        async fn upsert(&self, _: &BlogPost) -> Result<(), RepositoryError> {
            unreachable!("not part of the clamp test")
        }
        async fn delete_not_in(&self, _: &[String]) -> Result<Vec<String>, RepositoryError> {
            unreachable!("not part of the clamp test")
        }
    }

    fn container_of(repo: Arc<dyn BlogRepository>) -> Container {
        Container {
            blog: BlogUseCases {
                list_published: Arc::new(ListPublishedPosts::new(repo.clone())),
                get_by_slug: Arc::new(GetPostBySlug::new(repo.clone())),
                get_by_id: Arc::new(GetPostById::new(repo.clone())),
                get_markdown: Arc::new(GetPostMarkdown::new(repo.clone())),
                upsert: Arc::new(UpsertPost::new(repo.clone())),
                prune: Arc::new(PrunePosts::new(repo)),
            },
        }
    }

    /// The in-memory repository answers on the first poll; anything else is a
    /// test bug, not something to wait for.
    fn resolve<F: std::future::Future>(future: F) -> F::Output {
        let mut future = std::pin::pin!(future);
        match future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("server function did not resolve on the first poll"),
        }
    }

    /// Runs `call` inside a reactive owner that carries a container over
    /// `repo`, the way `handle_server_fns` provides it per request.
    fn with_repo<T>(repo: Arc<dyn BlogRepository>, call: impl FnOnce() -> T) -> T {
        Owner::new().with(|| {
            provide_context(container_of(repo));
            call()
        })
    }

    #[test]
    fn repository_failures_are_opaque_to_clients() {
        let list = with_repo(Arc::new(Broken), || resolve(get_published_posts(None)))
            .expect_err("the repository is broken");
        let post = with_repo(Arc::new(Broken), || {
            resolve(get_post_by_slug("any".to_owned()))
        })
        .expect_err("the repository is broken");
        for error in [list.to_string(), post.to_string()] {
            assert!(
                error.contains("service unavailable"),
                "unexpected message: {error}"
            );
            assert!(
                !error.contains("database"),
                "database detail leaked: {error}"
            );
            assert!(
                !error.contains("blog_posts"),
                "schema detail leaked: {error}"
            );
        }
    }

    // SECURITY.md SEC-012: `limit` is client-controlled; the repository must
    // only ever see it folded into [1, 100].
    #[test]
    fn client_limit_is_clamped() {
        for limit in [
            Some(i32::MAX),
            Some(101),
            Some(0),
            Some(-5),
            Some(i32::MIN),
            None,
        ] {
            let posts = with_repo(Arc::new(LimitProbe), || resolve(get_published_posts(limit)))
                .unwrap_or_else(|_| panic!("limit {limit:?} should succeed"));
            assert!(posts.is_empty());
        }
    }

    #[test]
    fn missing_container_is_opaque_too() {
        let error = Owner::new()
            .with(|| resolve(get_published_posts(None)))
            .expect_err("no container in context");
        assert!(error.to_string().contains("service unavailable"));
    }
}
