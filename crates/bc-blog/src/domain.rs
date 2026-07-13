//! Domain layer of the blog BC.
//!
//! The business concepts: a blog post, its public summary and the persistence
//! port. Nothing here should know about HTTP, DynamoDB or the runtime.

pub mod errors;
pub mod model;
pub mod repository;
