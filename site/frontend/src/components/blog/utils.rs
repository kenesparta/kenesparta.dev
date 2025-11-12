use crate::blog::PostStatus;
use chrono::{DateTime, Utc};

pub fn published_date(published_at: Option<i64>) -> String {
    published_at
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .map(|dt: DateTime<Utc>| dt.format("%B %d, %Y").to_string())
        .unwrap_or_else(|| PostStatus::Draft.to_string())
}
