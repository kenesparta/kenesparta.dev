//! PostgreSQL adapter for the blog BC (implements the `BlogRepository` port).
//!
//! Maps between the Postgres schema (uuid, timestamptz, text[]) and the
//! runtime-agnostic domain model (String ids, Unix-seconds i64).

use async_trait::async_trait;
use bc_blog::domain::model::{BlogPost, PostStatus};
use bc_blog::domain::repository::{BlogRepository, RepositoryError};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct BlogPostRow {
    post_id: Uuid,
    title: String,
    slug: String,
    content: String,
    summary: String,
    author: String,
    tags: Vec<String>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
}

impl From<BlogPostRow> for BlogPost {
    fn from(row: BlogPostRow) -> Self {
        Self {
            post_id: row.post_id.to_string(),
            status: match row.status.as_str() {
                "published" => PostStatus::Published,
                _ => PostStatus::Draft,
            },
            created_at: row.created_at.timestamp(),
            updated_at: row.updated_at.timestamp(),
            published_at: row.published_at.map(|t| t.timestamp()),
            title: row.title,
            slug: row.slug,
            content: row.content,
            summary: row.summary,
            author: row.author,
            tags: row.tags,
        }
    }
}

fn infra(err: sqlx::Error) -> RepositoryError {
    RepositoryError::Infrastructure(err.to_string())
}

fn to_utc(secs: i64) -> Result<DateTime<Utc>, RepositoryError> {
    DateTime::from_timestamp(secs, 0)
        .ok_or_else(|| RepositoryError::Infrastructure(format!("timestamp out of range: {secs}")))
}

pub struct PostgresBlogRepository {
    pool: PgPool,
}

impl PostgresBlogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BlogRepository for PostgresBlogRepository {
    async fn list_published(&self, limit: i32) -> Result<Vec<BlogPost>, RepositoryError> {
        let rows: Vec<BlogPostRow> = sqlx::query_as(
            "SELECT post_id, title, slug, content, summary, author, \
             tags, status, created_at, updated_at, published_at \
             FROM blog_posts \
             WHERE status = 'published' \
             ORDER BY created_at DESC \
             LIMIT $1",
        )
        .bind(i64::from(limit.max(0)))
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<BlogPost>, RepositoryError> {
        let row: Option<BlogPostRow> = sqlx::query_as(
            "SELECT post_id, title, slug, content, summary, author, \
             tags, status, created_at, updated_at, published_at \
             FROM blog_posts \
             WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?;

        Ok(row.map(Into::into))
    }

    async fn find_by_id(&self, post_id: &str) -> Result<Option<BlogPost>, RepositoryError> {
        // A malformed uuid cannot match any row: "not found", not an error.
        let Ok(id) = Uuid::parse_str(post_id) else {
            return Ok(None);
        };

        let row: Option<BlogPostRow> = sqlx::query_as(
            "SELECT post_id, title, slug, content, summary, author, \
             tags, status, created_at, updated_at, published_at \
             FROM blog_posts \
             WHERE post_id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?;

        Ok(row.map(Into::into))
    }

    async fn upsert(&self, post: &BlogPost) -> Result<(), RepositoryError> {
        let post_id = Uuid::parse_str(&post.post_id)
            .map_err(|e| RepositoryError::Infrastructure(format!("invalid post_id: {e}")))?;
        let published_at = post.published_at.map(to_utc).transpose()?;

        // post_id and created_at are intentionally NOT updated on conflict:
        // ids and URLs stay stable across re-ingests.
        sqlx::query(
            "INSERT INTO blog_posts \
               (post_id, title, slug, content, summary, author, tags, status, \
                created_at, updated_at, published_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (slug) DO UPDATE SET \
               title = EXCLUDED.title, \
               content = EXCLUDED.content, \
               summary = EXCLUDED.summary, \
               author = EXCLUDED.author, \
               tags = EXCLUDED.tags, \
               status = EXCLUDED.status, \
               updated_at = EXCLUDED.updated_at, \
               published_at = EXCLUDED.published_at",
        )
        .bind(post_id)
        .bind(&post.title)
        .bind(&post.slug)
        .bind(&post.content)
        .bind(&post.summary)
        .bind(&post.author)
        .bind(&post.tags)
        .bind(post.status.as_str())
        .bind(to_utc(post.created_at)?)
        .bind(to_utc(post.updated_at)?)
        .bind(published_at)
        .execute(&self.pool)
        .await
        .map_err(infra)?;

        Ok(())
    }

    async fn delete_not_in(&self, keep: &[String]) -> Result<Vec<String>, RepositoryError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("DELETE FROM blog_posts WHERE slug <> ALL($1) RETURNING slug")
                .bind(keep)
                .fetch_all(&self.pool)
                .await
                .map_err(infra)?;

        Ok(rows.into_iter().map(|(slug,)| slug).collect())
    }
}
