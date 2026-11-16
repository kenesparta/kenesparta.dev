//! Runtime configuration read from the environment.

use std::fmt;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error(
        "missing required env var {0}. Locally: `make dev/up` (docker) or run under \
         `sops exec-env secrets/dev.enc.env '<cmd>'`. In production personal-infra sets \
         DSQL_ENDPOINT, DSQL_USER and DSQL_SCHEMA on the function. See README > Secrets."
    )]
    MissingVar(&'static str),
    #[error("{0} must be a lowercase SQL identifier ([a-z_][a-z0-9_]*)")]
    InvalidIdentifier(&'static str),
}

/// Where the posts live.
#[derive(Clone)]
pub enum DatabaseConfig {
    /// Local development: a plain Postgres connection string (`DATABASE_URL`).
    /// Secret — it carries the password — so it never reaches `Debug`.
    Url(String),
    /// Production: Aurora DSQL, authenticated with an IAM token minted per
    /// connection (personal-infra §5.22). Nothing here is secret: the
    /// credential is the caller's IAM role.
    Dsql {
        /// `<cluster id>.dsql.<region>.on.aws`.
        endpoint: String,
        /// The Postgres role to connect as: `blog_app` on Lambda, `blog_owner`
        /// from `make blog/publish`. DSQL has no `SET ROLE`, so each caller
        /// connects as the role it needs, mapped to its IAM role.
        user: String,
        /// Schema holding the tables, applied as the connection's `search_path`.
        schema: String,
    },
}

impl fmt::Debug for DatabaseConfig {
    // Hand-written so the URL's password can never reach a log line, a panic
    // message or an error report (SECURITY.md SEC-025).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(_) => f.write_str("Url(<redacted>)"),
            Self::Dsql {
                endpoint,
                user,
                schema,
            } => f
                .debug_struct("Dsql")
                .field("endpoint", endpoint)
                .field("user", user)
                .field("schema", schema)
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub database: DatabaseConfig,
}

impl Configuration {
    /// `DSQL_ENDPOINT` selects DSQL and then requires `DSQL_USER` and
    /// `DSQL_SCHEMA`; without it, `DATABASE_URL` is required.
    ///
    /// # Errors
    ///
    /// [`ConfigurationError::MissingVar`] for a required variable that is not
    /// set; [`ConfigurationError::InvalidIdentifier`] for a role or schema
    /// name that is not a plain identifier. There is deliberately no default:
    /// a misconfigured function must fail at init, not serve without its data.
    pub fn from_env() -> Result<Self, ConfigurationError> {
        Self::from_lookup(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))
    }

    /// The decision `from_env` makes, over any lookup, so it can be tested.
    fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigurationError> {
        let database = match get("DSQL_ENDPOINT") {
            Some(endpoint) => DatabaseConfig::Dsql {
                endpoint,
                user: identifier("DSQL_USER", get("DSQL_USER"))?,
                schema: identifier("DSQL_SCHEMA", get("DSQL_SCHEMA"))?,
            },
            None => DatabaseConfig::Url(
                get("DATABASE_URL").ok_or(ConfigurationError::MissingVar("DATABASE_URL"))?,
            ),
        };
        Ok(Self { database })
    }
}

/// The schema name is interpolated into `SET search_path`, which cannot take a
/// bound parameter, so only a plain identifier passes; role names are held to
/// the same rule.
fn identifier(name: &'static str, value: Option<String>) -> Result<String, ConfigurationError> {
    let value = value.ok_or(ConfigurationError::MissingVar(name))?;
    let mut chars = value.chars();
    let valid = chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    valid
        .then_some(value)
        .ok_or(ConfigurationError::InvalidIdentifier(name))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn config(vars: &[(&str, &str)]) -> Result<Configuration, ConfigurationError> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        Configuration::from_lookup(|name| vars.get(name).cloned())
    }

    #[test]
    fn database_url_selects_plain_postgres() {
        let c = config(&[("DATABASE_URL", "postgres://u:p@h/db")]).expect("valid");
        assert!(matches!(c.database, DatabaseConfig::Url(_)));
    }

    #[test]
    fn dsql_endpoint_selects_dsql_and_needs_user_and_schema() {
        let c = config(&[
            ("DSQL_ENDPOINT", "abc.dsql.us-east-1.on.aws"),
            ("DSQL_USER", "blog_app"),
            ("DSQL_SCHEMA", "blog"),
        ])
        .expect("valid");
        assert!(matches!(c.database, DatabaseConfig::Dsql { .. }));
        assert!(matches!(
            config(&[
                ("DSQL_ENDPOINT", "abc.dsql.us-east-1.on.aws"),
                ("DSQL_SCHEMA", "blog")
            ]),
            Err(ConfigurationError::MissingVar("DSQL_USER"))
        ));
    }

    #[test]
    fn nothing_set_is_an_error() {
        assert!(matches!(
            config(&[]),
            Err(ConfigurationError::MissingVar("DATABASE_URL"))
        ));
    }

    #[test]
    fn identifiers_are_plain() {
        for bad in ["Blog", "blog;drop", "1blog", "blog owner", "blog\"x"] {
            assert!(
                config(&[
                    ("DSQL_ENDPOINT", "abc.dsql.us-east-1.on.aws"),
                    ("DSQL_USER", "blog_app"),
                    ("DSQL_SCHEMA", bad),
                ])
                .is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn debug_never_prints_the_url() {
        let c = config(&[("DATABASE_URL", "postgres://u:s3cret@h/db")]).expect("valid");
        assert!(!format!("{c:?}").contains("s3cret"));
    }
}
