use thiserror::Error;

/// Domain errors shared across Bounded Contexts.
///
/// Only what truly means the same thing in more than one BC belongs here.
/// If an error is specific to a context, define it in that BC's
/// `domain/errors.rs`.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invariant violated: {0}")]
    Invariant(String),

    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("operation not allowed in the current state: {0}")]
    InvalidState(String),
}
