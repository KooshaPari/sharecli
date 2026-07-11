//! Opt-in CPU profiling HTTP surface for `sharecli serve` (audit-v38 L45).
//!
//! Enable with `SHARECLI_PPROF=1`. Profile capture is Unix-only (`pprof` crate);
//! Windows returns 501 so the route still appears in OpenAPI / ops docs.

use axum::extract::Query;
#[cfg(unix)]
use axum::http::header;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

/// Parse `SHARECLI_PPROF` style values (`None`/`""`/`"0"` ⇒ off).
pub fn pprof_enabled_from(val: Option<&str>) -> bool {
    matches!(val, Some(v) if !v.is_empty() && v != "0")
}

/// True when `SHARECLI_PPROF` is set to a non-empty, non-`0` value.
pub fn pprof_enabled() -> bool {
    pprof_enabled_from(std::env::var("SHARECLI_PPROF").ok().as_deref())
}

#[derive(Debug, Deserialize)]
pub struct ProfileQuery {
    /// Capture window in seconds (default 10, hard-capped at 60).
    #[serde(default = "default_seconds")]
    pub seconds: u64,
}

fn default_seconds() -> u64 {
    10
}

/// `GET /debug/pprof/profile` — flamegraph SVG of a short CPU sample.
///
/// Requires `SHARECLI_PPROF=1`. Honors serve Bearer auth when configured
/// (route is not public).
pub async fn profile_handler(Query(q): Query<ProfileQuery>) -> Response {
    if !pprof_enabled() {
        return (
            StatusCode::NOT_FOUND,
            "profiling disabled; set SHARECLI_PPROF=1 to enable",
        )
            .into_response();
    }

    let seconds = q.seconds.clamp(1, 60);

    #[cfg(unix)]
    {
        match capture_flamegraph(seconds).await {
            Ok(svg) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                svg,
            )
                .into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("pprof failed: {err}"))
                .into_response(),
        }
    }

    #[cfg(not(unix))]
    {
        let _ = seconds;
        (
            StatusCode::NOT_IMPLEMENTED,
            "CPU profiling via pprof is Unix-only on this build; use samply/perf externally (see docs/ops/profiling.md)",
        )
            .into_response()
    }
}

#[cfg(unix)]
async fn capture_flamegraph(seconds: u64) -> Result<Vec<u8>, String> {
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .map_err(|e| e.to_string())?;

    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;

    tokio::task::spawn_blocking(move || {
        let report = guard.report().build().map_err(|e| e.to_string())?;
        let mut body = Vec::new();
        report.flamegraph(&mut body).map_err(|e| e.to_string())?;
        Ok(body)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pprof_parse_disabled_cases() {
        assert!(!pprof_enabled_from(None));
        assert!(!pprof_enabled_from(Some("")));
        assert!(!pprof_enabled_from(Some("0")));
    }

    #[test]
    fn pprof_parse_enabled_cases() {
        assert!(pprof_enabled_from(Some("1")));
        assert!(pprof_enabled_from(Some("true")));
        assert!(pprof_enabled_from(Some("yes")));
    }
}
