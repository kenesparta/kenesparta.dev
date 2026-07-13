use uuid::Uuid;

/// Generator of opaque string identifiers for blog posts.
pub struct PostUuid;

impl PostUuid {
    /// A fresh random (v4) identifier as a string.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> String {
        Uuid::new_v4().to_string()
    }
}
