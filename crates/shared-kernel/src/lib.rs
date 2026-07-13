//! Shared Kernel of the workspace.
//!
//! Contains only genuinely cross-cutting types that more than one Bounded
//! Context needs to share literally (not "because they look alike"). Before
//! adding something here, ask whether two BCs really share the same concept
//! or whether it should be duplicated to avoid coupling.

pub mod date;
pub mod errors;
pub mod id;

pub use date::Datetime;
pub use errors::DomainError;
pub use id::PostUuid;
