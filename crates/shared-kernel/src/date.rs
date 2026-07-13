use chrono::Utc;

/// Wall-clock helper for domain timestamps (Unix seconds).
pub struct Datetime;

impl Datetime {
    /// Current time as a Unix timestamp in seconds.
    pub fn now() -> i64 {
        Utc::now().timestamp()
    }
}
