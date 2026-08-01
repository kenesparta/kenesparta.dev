//! Logging: structured JSON events on stdout.
//!
//! Both bins (server and ingest) call [`init`] before anything else. In
//! production `docker logs` ships to CloudWatch (personal-infra AD-11), so
//! lines are JSON — one object per line, event fields flattened to the top
//! level — and Logs Insights discovers the fields without any parser config.
//!
//! [`access_log`] is the per-request half: one INFO event per page request
//! with the viewer's IP and geolocation. Those come from CloudFront
//! (personal-infra AD-12): a viewer-request CloudFront Function injects
//! `true-client-ip`, and the distribution's cache policy forwards the
//! `CloudFront-Viewer-*` geo headers. Locally none of them exist and the
//! fields log as "-".

use std::time::Instant;

use axum::extract::Request;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;

/// Install the global JSON subscriber. `RUST_LOG` filters; `info` when unset.
pub fn init() {
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

/// Access log: one INFO event per request, after the response is produced.
///
/// Mounted outermost, before [`crate::seo::rewrite_markdown_suffix`], so
/// `path` is the public URI (`/blog/<slug>.md`), not the internal rewrite.
pub async fn access_log(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    // Asset fetches (css/js/wasm accompany every page view) would triple the
    // volume for zero information about who is reading what.
    if path.starts_with("/pkg/") || path == "/favicon.ico" {
        return next.run(request).await;
    }

    let method = request.method().to_string();
    let headers = request.headers();
    let client_ip = client_ip(headers);
    let country = header(headers, "cloudfront-viewer-country");
    let region = header(headers, "cloudfront-viewer-country-region-name");
    let city = header(headers, "cloudfront-viewer-city");
    let user_agent = header(headers, "user-agent");
    let referer = header(headers, "referer");

    let started = Instant::now();
    let response = next.run(request).await;

    tracing::info!(
        client_ip = %client_ip,
        country = %country,
        region = %region,
        city = %city,
        method = %method,
        path = %path,
        status = response.status().as_u16(),
        latency_ms = started.elapsed().as_millis() as u64,
        user_agent = %user_agent,
        referer = %referer,
        "request"
    );

    response
}

/// The viewer's IP. `true-client-ip` is authoritative: the CloudFront
/// Function overwrites it unconditionally at the edge, so a client-sent value
/// never survives. The `X-Forwarded-For` fallback only matters off-CloudFront
/// (local dev) and is client-controlled — a hint, not an identity.
fn client_ip(headers: &HeaderMap) -> String {
    if let Some(ip) = headers.get("true-client-ip").and_then(|v| v.to_str().ok()) {
        return ip.to_string();
    }
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|ip| ip.trim().to_string())
        .unwrap_or_else(|| "-".to_string())
}

/// Header value or `"-"`, percent-decoded: CloudFront encodes non-ASCII
/// header values per RFC 3986 ("S%C3%A3o%20Paulo"), which would otherwise
/// land verbatim in the logs.
fn header(headers: &HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(percent_decode)
        .unwrap_or_else(|| "-".to_string())
}

fn percent_decode(raw: &str) -> String {
    let mut bytes = Vec::with_capacity(raw.len());
    let mut rest = raw.bytes();
    while let Some(byte) = rest.next() {
        if byte != b'%' {
            bytes.push(byte);
            continue;
        }
        // A '%' not followed by two hex digits is kept verbatim: header
        // values are not guaranteed to be well-formed encodings.
        let pair = [rest.next(), rest.next()];
        match pair {
            [Some(hi), Some(lo)] => {
                match (char::from(hi).to_digit(16), char::from(lo).to_digit(16)) {
                    (Some(hi), Some(lo)) => bytes.push((hi * 16 + lo) as u8),
                    _ => bytes.extend([b'%', hi, lo]),
                }
            }
            [Some(hi), None] => bytes.extend([b'%', hi]),
            _ => bytes.push(b'%'),
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::percent_decode;

    #[test]
    fn decodes_cloudfront_encoded_values() {
        assert_eq!(percent_decode("S%C3%A3o%20Paulo"), "São Paulo");
        assert_eq!(percent_decode("Lima"), "Lima");
        assert_eq!(percent_decode("50%"), "50%");
        assert_eq!(percent_decode("bad%2"), "bad%2");
    }
}
