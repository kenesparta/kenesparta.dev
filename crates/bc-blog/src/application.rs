//! Application layer of the blog BC.
//!
//! Orchestrates the read use cases and exposes the wire DTOs. No transport or
//! DynamoDB details here.

pub mod dto;
pub mod use_cases;
