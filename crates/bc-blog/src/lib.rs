//! Bounded Context: blog
//!
//! Posts and their summaries, plus the persistence port. Contains only the
//! `domain` and `application` layers. It knows nothing about HTTP, PostgreSQL
//! or the runtime: the adapters live in the binary crate (apps/backend).

pub mod application;
pub mod domain;
