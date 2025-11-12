use chrono::Utc;

pub struct Datetime {}

impl Datetime {
    pub fn now() -> i64 {
        Utc::now().timestamp()
    }
}
