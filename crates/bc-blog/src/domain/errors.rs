//! Domain-specific errors of the blog BC.
//!
//! The blog has no bespoke domain errors yet: validation uses
//! `shared_kernel::DomainError` and infrastructure failures live in the
//! repository port (`RepositoryError`). This module exists to keep the
//! scaffold's shape and host future errors.
