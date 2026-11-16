//! Connecting to Aurora DSQL (personal-infra §5.22, G51, G52).
//!
//! DSQL has no passwords. A client presents an IAM token as the password,
//! minted locally by signing a request with the caller's AWS credentials — the
//! execution role's on Lambda, the SSO profile on the laptop. A token is valid
//! for 15 minutes and a connection lives at most 60, so a pool configured once
//! at startup works until it opens a connection after minute 15 and then fails
//! authentication while every warm connection still works (G52). Two things
//! prevent that:
//!
//! - [`Refresher::ensure_fresh`] mints a new token once the current one is ten
//!   minutes old and hands the pool new connect options. The web server calls
//!   it before each request ([`refresh`]); on Lambda nothing runs between
//!   requests, so a background task would never fire.
//! - The pool retires connections after 50 minutes, before DSQL closes them.
//!
//! DSQL refuses `SET ROLE`, so a caller connects as the role it needs: the
//! function as `blog_app`, `make blog/publish` as `blog_owner`, each mapped to
//! its own IAM role with `AWS IAM GRANT` (personal-infra `make dsql/roles`).

use std::sync::Arc;
use std::time::{Duration, Instant};

use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config as TokenConfig};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{AssertSqlSafe, Executor, PgPool};
use thiserror::Error;
use tokio::sync::Mutex;

/// Seconds a token stays valid, DSQL's default.
const TOKEN_LIFETIME_SECS: u64 = 900;
/// Re-mint well before expiry, so a connection opened late in a token's life
/// still authenticates.
const REFRESH_AFTER: Duration = Duration::from_secs(600);
/// DSQL ends every connection at 60 minutes.
const CONNECTION_LIFETIME: Duration = Duration::from_secs(50 * 60);

#[derive(Debug, Error)]
pub enum DsqlError {
    #[error("DSQL endpoint `{0}` is not <cluster>.dsql.<region>.on.aws")]
    Endpoint(String),
    #[error("minting the DSQL auth token failed: {0}")]
    Token(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

/// One cluster, one role: everything needed to mint a token and connect.
pub struct Dsql {
    endpoint: String,
    user: String,
    schema: String,
    signer: AuthTokenGenerator,
    aws: SdkConfig,
}

impl Dsql {
    /// Loads the AWS credential chain once. `user` and `schema` are already
    /// validated identifiers (`Configuration::from_env`).
    ///
    /// # Errors
    ///
    /// [`DsqlError::Endpoint`] if the endpoint names no region.
    pub async fn new(endpoint: String, user: String, schema: String) -> Result<Self, DsqlError> {
        let region = region_of(&endpoint)?;
        let aws = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(region.clone()))
            .load()
            .await;
        let signer = AuthTokenGenerator::new(
            TokenConfig::builder()
                .hostname(endpoint.clone())
                .region(Region::new(region))
                .expires_in(TOKEN_LIFETIME_SECS)
                .build()
                .map_err(|e| DsqlError::Token(e.to_string()))?,
        );
        Ok(Self {
            endpoint,
            user,
            schema,
            signer,
            aws,
        })
    }

    /// Connect options carrying a freshly minted token. `admin` needs the
    /// admin variant (`dsql:DbConnectAdmin`); every other role `dsql:DbConnect`.
    async fn connect_options(&self) -> Result<PgConnectOptions, DsqlError> {
        let token = if self.user == "admin" {
            self.signer.db_connect_admin_auth_token(&self.aws).await
        } else {
            self.signer.db_connect_auth_token(&self.aws).await
        }
        .map_err(|e| DsqlError::Token(e.to_string()))?;

        Ok(PgConnectOptions::new()
            .host(&self.endpoint)
            .port(5432)
            .database("postgres")
            .username(&self.user)
            .password(token.as_str())
            .ssl_mode(PgSslMode::VerifyFull))
    }

    /// A pool sized for one request at a time per execution environment.
    /// `search_path` is applied to every new connection, so no query names a
    /// schema.
    ///
    /// # Errors
    ///
    /// A token that cannot be minted, or a first connection that fails —
    /// either one must stop the function at init.
    pub async fn connect(self) -> Result<(PgPool, Arc<Refresher>), DsqlError> {
        let options = self.connect_options().await?;
        // A validated identifier (Configuration::from_env), so interpolating
        // it is safe; a schema name cannot be bound as a parameter.
        let search_path = format!("SET search_path TO {}", self.schema);
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .max_lifetime(CONNECTION_LIFETIME)
            .after_connect(move |conn, _meta| {
                let statement = search_path.clone();
                Box::pin(async move {
                    conn.execute(AssertSqlSafe(statement)).await?;
                    Ok(())
                })
            })
            .connect_with(options)
            .await?;
        let refresher = Arc::new(Refresher {
            dsql: self,
            pool: pool.clone(),
            minted_at: Mutex::new(Instant::now()),
        });
        Ok((pool, refresher))
    }
}

/// Keeps the pool's token fresh (see the module docs).
pub struct Refresher {
    dsql: Dsql,
    pool: PgPool,
    minted_at: Mutex<Instant>,
}

impl Refresher {
    /// Mints a new token if the current one is older than [`REFRESH_AFTER`].
    /// Cheap otherwise: one lock and one clock read.
    ///
    /// # Errors
    ///
    /// [`DsqlError::Token`] if minting fails. Existing connections keep
    /// working; only a new one would then fail to authenticate.
    pub async fn ensure_fresh(&self) -> Result<(), DsqlError> {
        let mut minted_at = self.minted_at.lock().await;
        if minted_at.elapsed() < REFRESH_AFTER {
            return Ok(());
        }
        self.pool
            .set_connect_options(self.dsql.connect_options().await?);
        *minted_at = Instant::now();
        Ok(())
    }
}

/// Middleware: refresh the token before the request reaches a handler. A
/// failure is logged and the request goes on, since a warm connection may well
/// serve it.
pub async fn refresh(
    State(refresher): State<Option<Arc<Refresher>>>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(refresher) = refresher
        && let Err(error) = refresher.ensure_fresh().await
    {
        tracing::error!(%error, "DSQL token refresh failed");
    }
    next.run(request).await
}

/// `<cluster>.dsql.<region>.on.aws` → `<region>`.
fn region_of(endpoint: &str) -> Result<String, DsqlError> {
    let labels: Vec<&str> = endpoint.split('.').collect();
    match labels.as_slice() {
        [cluster, "dsql", region, "on", "aws"] if !cluster.is_empty() && !region.is_empty() => {
            Ok((*region).to_owned())
        }
        _ => Err(DsqlError::Endpoint(endpoint.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_region_comes_from_the_endpoint() {
        assert_eq!(
            region_of("abc123.dsql.us-east-1.on.aws").expect("valid"),
            "us-east-1"
        );
        for bad in [
            "abc.dsql.us-east-1.amazonaws.com",
            "dsql.us-east-1.on.aws",
            "localhost",
        ] {
            assert!(region_of(bad).is_err(), "{bad}");
        }
    }
}
