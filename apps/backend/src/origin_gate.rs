//! The origin gate: only requests that came through CloudFront are served.
//!
//! On Lambda the origin is a Function URL, a public HTTPS endpoint anyone can
//! call. CloudFront adds `X-Origin-Verify: <secret>` to every request it
//! forwards (personal-infra, spec §5.21), and this gate answers 403 to anything
//! that arrives without it. Until the move to Lambda, Caddy made the same check
//! in front of the container; it now lives here, in the only part of the chain
//! this repository owns (SECURITY.md SEC-025).
//!
//! Two values are accepted. `ORIGIN_SECRET` is the current one;
//! `ORIGIN_SECRET_PREVIOUS` is set only while personal-infra rotates the secret,
//! because the function's environment changes in seconds and CloudFront's
//! header in minutes (personal-infra G13). A request carrying either is served.
//!
//! The gate fails closed. On Lambda — where `AWS_LAMBDA_FUNCTION_NAME`
//! is always set — a missing or empty `ORIGIN_SECRET` stops the server at
//! startup instead of serving an open origin. Only off Lambda, under
//! `make dev/up` with no CloudFront in front, is an absent secret a disabled
//! gate.

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::fmt;
use std::sync::Arc;

/// The header CloudFront sets on every forwarded request.
pub const HEADER: &str = "x-origin-verify";

/// One accepted secret. `Debug` is written by hand so the value can never reach
/// a log line, a panic message or an error report.
struct Secret(Box<[u8]>);

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

/// The gate's state: the accepted secrets, or none when the gate is disabled.
#[derive(Clone, Debug)]
pub struct OriginGate {
    accepted: Option<Arc<[Secret]>>,
}

/// Why the gate refused to start.
#[derive(Debug, PartialEq, Eq)]
pub enum GateError {
    /// Running on Lambda with no `ORIGIN_SECRET`: the origin would be open.
    MissingOnLambda,
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOnLambda => f.write_str(
                "ORIGIN_SECRET is unset or empty on Lambda; refusing to serve an open origin",
            ),
        }
    }
}

impl std::error::Error for GateError {}

impl OriginGate {
    /// Reads `ORIGIN_SECRET` and `ORIGIN_SECRET_PREVIOUS` once, at startup.
    pub fn from_env() -> Result<Self, GateError> {
        let on_lambda = std::env::var_os("AWS_LAMBDA_FUNCTION_NAME").is_some();
        Self::from_values(
            std::env::var("ORIGIN_SECRET").ok(),
            std::env::var("ORIGIN_SECRET_PREVIOUS").ok(),
            on_lambda,
        )
    }

    /// The decision `from_env` makes, without the environment, so it can be
    /// tested. Empty values count as absent.
    pub fn from_values(
        current: Option<String>,
        previous: Option<String>,
        on_lambda: bool,
    ) -> Result<Self, GateError> {
        let present = |v: Option<String>| v.filter(|s| !s.is_empty());
        match present(current) {
            None if on_lambda => Err(GateError::MissingOnLambda),
            None => Ok(Self::disabled()),
            Some(current) => {
                let accepted: Vec<Secret> = std::iter::once(current)
                    .chain(present(previous))
                    .map(|s| Secret(s.into_bytes().into_boxed_slice()))
                    .collect();
                Ok(Self {
                    accepted: Some(accepted.into()),
                })
            }
        }
    }

    /// Serves everything. For local development and for tests of other layers.
    pub fn disabled() -> Self {
        Self { accepted: None }
    }

    fn accepts(&self, presented: Option<&[u8]>) -> bool {
        match &self.accepted {
            None => true,
            Some(accepted) => presented.is_some_and(|presented| {
                accepted.iter().fold(false, |hit, secret| {
                    hit | constant_time_eq(&secret.0, presented)
                })
            }),
        }
    }
}

/// Compares in time that depends only on the lengths, so the response time
/// says nothing about how much of a guess was right.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |diff, (x, y)| diff | (x ^ y)) == 0
}

/// The middleware. A refused request gets an empty 403, and the access log
/// records it like any other response.
pub async fn check(State(gate): State<OriginGate>, request: Request, next: Next) -> Response {
    let presented = request.headers().get(HEADER).map(|v| v.as_bytes());
    if gate.accepts(presented) {
        next.run(request).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::routing::get;
    use tower::ServiceExt;

    fn router(gate: OriginGate) -> Router {
        Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(gate, check))
    }

    async fn status(gate: OriginGate, header: Option<&str>) -> StatusCode {
        let mut request = Request::builder().uri("/");
        if let Some(value) = header {
            request = request.header(HEADER, value);
        }
        router(gate)
            .oneshot(request.body(Body::empty()).expect("a valid request"))
            .await
            .expect("the router is infallible")
            .status()
    }

    fn gate(current: &str, previous: Option<&str>) -> OriginGate {
        OriginGate::from_values(Some(current.into()), previous.map(Into::into), true)
            .expect("a secret is set")
    }

    #[tokio::test]
    async fn serves_the_current_secret() {
        assert_eq!(
            status(gate("s3cret", None), Some("s3cret")).await,
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn refuses_a_missing_or_wrong_header() {
        assert_eq!(
            status(gate("s3cret", None), None).await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            status(gate("s3cret", None), Some("s3creT")).await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            status(gate("s3cret", None), Some("s3cre")).await,
            StatusCode::FORBIDDEN
        );
    }

    #[tokio::test]
    async fn accepts_the_previous_secret_during_a_rotation() {
        let rotating = gate("new", Some("old"));
        assert_eq!(status(rotating.clone(), Some("new")).await, StatusCode::OK);
        assert_eq!(status(rotating.clone(), Some("old")).await, StatusCode::OK);
        assert_eq!(status(rotating, Some("other")).await, StatusCode::FORBIDDEN);
    }

    #[test]
    fn an_empty_previous_secret_accepts_nothing_extra() {
        let gate = OriginGate::from_values(Some("new".into()), Some(String::new()), true)
            .expect("a secret is set");
        assert!(!gate.accepts(Some(b"")));
    }

    #[test]
    fn fails_closed_on_lambda_without_a_secret() {
        assert_eq!(
            OriginGate::from_values(None, None, true).unwrap_err(),
            GateError::MissingOnLambda
        );
        assert_eq!(
            OriginGate::from_values(Some(String::new()), None, true).unwrap_err(),
            GateError::MissingOnLambda
        );
    }

    #[tokio::test]
    async fn is_disabled_off_lambda_without_a_secret() {
        let local = OriginGate::from_values(None, None, false).expect("dev has no secret");
        assert_eq!(status(local, None).await, StatusCode::OK);
    }

    #[test]
    fn debug_never_prints_a_secret() {
        let printed = format!("{:?}", gate("s3cret", Some("old")));
        assert!(
            !printed.contains("s3cret") && !printed.contains("old"),
            "{printed}"
        );
    }
}
