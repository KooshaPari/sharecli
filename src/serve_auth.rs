//! Optional Bearer-token AuthN for `sharecli serve`.
//!
//! When `SHARECLI_SERVE_TOKEN` (env) or `config.serve.bearer_token` is set,
//! non-probe routes require `Authorization: Bearer <token>`. `/healthz` and
//! `/readyz` stay open so kubelet/load-balancer probes keep working.
//!
//! When no token is configured, the server stays open (localhost trust model)
//! and callers should not bind beyond loopback without setting a token.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use sha2::{Digest, Sha256};

/// Shared auth state cloned into the Axum router.
#[derive(Clone, Debug, Default)]
pub struct ServeAuth {
    /// Expected bearer token. `None` ⇒ auth disabled (open).
    pub token: Option<String>,
}

impl ServeAuth {
    /// Resolve token from env first, then config value.
    pub fn from_env_or_config(config_token: Option<&str>) -> Self {
        let token = std::env::var("SHARECLI_SERVE_TOKEN")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| config_token.map(str::to_owned).filter(|s| !s.is_empty()));
        Self { token }
    }

    pub fn enabled(&self) -> bool {
        self.token.is_some()
    }

    pub fn check_bearer(&self, header_val: Option<&str>) -> bool {
        let Some(expected) = self.token.as_deref() else {
            return true;
        };
        let Some(raw) = header_val else {
            return false;
        };
        let Some(provided) = raw.strip_prefix("Bearer ").or_else(|| raw.strip_prefix("bearer "))
        else {
            return false;
        };
        tokens_equal(provided.trim(), expected)
    }
}

fn tokens_equal(a: &str, b: &str) -> bool {
    // Compare SHA-256 digests so length differences do not short-circuit as
    // obviously; both digests are fixed-size.
    let da = Sha256::digest(a.as_bytes());
    let db = Sha256::digest(b.as_bytes());
    da.as_slice() == db.as_slice()
}

/// Paths that remain reachable without a bearer token (liveness/readiness).
pub fn is_public_path(path: &str) -> bool {
    matches!(path, "/healthz" | "/readyz")
}

/// Axum middleware: enforce bearer token when configured.
pub async fn require_bearer(
    axum::extract::State(auth): axum::extract::State<ServeAuth>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_owned();
    if is_public_path(&path) || !auth.enabled() {
        return next.run(req).await;
    }

    let header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());

    if auth.check_bearer(header) {
        crate::audit_log::emit("auth_ok", json!({ "path": path }));
        return next.run(req).await;
    }

    crate::audit_log::emit(
        "auth_fail",
        json!({
            "path": path,
            "reason": "missing_or_invalid_bearer",
        }),
    );
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer")],
        Json(json!({ "error": "unauthorized", "hint": "Authorization: Bearer <SHARECLI_SERVE_TOKEN>" })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_paths() {
        assert!(is_public_path("/healthz"));
        assert!(is_public_path("/readyz"));
        assert!(!is_public_path("/metrics/prometheus"));
        assert!(!is_public_path("/config"));
    }

    #[test]
    fn auth_disabled_allows_any() {
        let auth = ServeAuth { token: None };
        assert!(auth.check_bearer(None));
        assert!(auth.check_bearer(Some("Bearer nope")));
    }

    #[test]
    fn auth_enabled_requires_matching_bearer() {
        let auth = ServeAuth { token: Some("s3cret".into()) };
        assert!(!auth.check_bearer(None));
        assert!(!auth.check_bearer(Some("Bearer wrong")));
        assert!(!auth.check_bearer(Some("Basic s3cret")));
        assert!(auth.check_bearer(Some("Bearer s3cret")));
        assert!(auth.check_bearer(Some("bearer s3cret")));
    }

    #[test]
    fn env_overrides_config() {
        // SAFETY: test-only; serialised by nextest default for this module's unit tests.
        unsafe {
            std::env::set_var("SHARECLI_SERVE_TOKEN", "from-env");
        }
        let auth = ServeAuth::from_env_or_config(Some("from-config"));
        assert_eq!(auth.token.as_deref(), Some("from-env"));
        unsafe {
            std::env::remove_var("SHARECLI_SERVE_TOKEN");
        }
        let auth = ServeAuth::from_env_or_config(Some("from-config"));
        assert_eq!(auth.token.as_deref(), Some("from-config"));
    }
}
