//! Runtime configuration read from the environment.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error(
        "missing required env var {0}. Locally: `make dev/up` (docker) or run under \
         `sops exec-env secrets/dev.enc.env '<cmd>'`. See README > Secrets."
    )]
    MissingVar(&'static str),
}

#[derive(Debug, Clone)]
pub struct Configuration {
    /// Postgres connection string. Secret — never log it.
    pub database_url: String,
}

impl Configuration {
    /// # Errors
    ///
    /// [`ConfigurationError::MissingVar`] if `DATABASE_URL` is not set. There
    /// is deliberately no default: a misconfigured prod container must die at
    /// boot so Lightsail keeps the previous deployment serving.
    pub fn from_env() -> Result<Self, ConfigurationError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ConfigurationError::MissingVar("DATABASE_URL"))?,
        })
    }
}
