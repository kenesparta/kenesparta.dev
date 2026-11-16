//! Schema migrations (apps/backend/migrations), run by `ingest` only.
//!
//! Each migration is written for Aurora DSQL and stays valid on the plain
//! Postgres of local development (personal-infra G51): it starts with
//! `-- no-transaction` and holds exactly ONE DDL statement. sqlx then executes
//! it on its own and records it in `_sqlx_migrations` with a separate
//! statement, which is the only shape DSQL accepts, since it runs one DDL per
//! transaction and never DDL beside DML. Locking is off because DSQL has no
//! `pg_advisory_lock`; one owner running `make blog/publish` is the only
//! migrator, so nothing races.
//!
//! **Never edit a migration that has been applied, not even a comment.** sqlx
//! stores each file's checksum, and a changed file stops every later run with
//! "previously applied but has been modified". Fix it with a new migration.

use sqlx::PgPool;
use sqlx::migrate::MigrateError;

/// Applies every pending migration.
///
/// # Errors
///
/// A migration that fails, or a database whose history no longer matches the
/// files (a local database from before the DSQL baseline: recreate it with
/// `docker compose -f docker-compose.dev.yml rm -s -f postgres && docker volume rm kenespartadev_pg-data`).
pub async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    let mut migrator = sqlx::migrate!();
    migrator.set_locking(false);
    migrator.run(pool).await
}
