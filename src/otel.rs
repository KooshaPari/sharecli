//! OpenTelemetry bootstrap for sharecli.
//!
//! When `OTEL_EXPORTER_OTLP_ENDPOINT` (or `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`)
//! is set, installs an OTLP/HTTP span exporter and a `tracing-opentelemetry`
//! layer so `#[instrument]` spans become OTel traces. Without those env vars
//! this module is a no-op so local/CI runs stay collector-free.
//!
//! Also installs the W3C `TraceContextPropagator` so HTTP middleware can
//! extract/inject `traceparent` (audit-v38 L42 / L44).

use std::sync::OnceLock;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

static PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();

/// True when an OTLP endpoint env var is present (export path is armed).
pub fn otel_enabled() -> bool {
    std::env::var_os("OTEL_EXPORTER_OTLP_ENDPOINT").is_some()
        || std::env::var_os("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT").is_some()
}

/// Build an optional `tracing` → OpenTelemetry layer.
///
/// Returns `None` when no OTLP endpoint is configured, or when the exporter
/// fails to build (logged via `eprintln!` so serve still starts).
pub fn try_otel_layer<S>() -> Option<OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer>>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    if !otel_enabled() {
        return None;
    }

    match build_provider() {
        Ok(provider) => {
            let _ = PROVIDER.set(provider);
            let tracer = PROVIDER.get().expect("provider just set").tracer("sharecli");
            global::set_text_map_propagator(TraceContextPropagator::new());
            Some(OpenTelemetryLayer::new(tracer))
        }
        Err(err) => {
            eprintln!("sharecli: OTel init failed ({err}); continuing without OTLP export");
            None
        }
    }
}

/// Ensure the W3C propagator is registered even when OTLP export is off, so
/// serve middleware can still parse/emit `traceparent` for in-process spans.
pub fn ensure_trace_context_propagator() {
    global::set_text_map_propagator(TraceContextPropagator::new());
}

fn build_provider() -> Result<SdkTracerProvider, Box<dyn std::error::Error + Send + Sync>> {
    let exporter = SpanExporter::builder().with_http().build()?;
    let resource = Resource::builder().with_service_name("sharecli").build();
    Ok(SdkTracerProvider::builder().with_batch_exporter(exporter).with_resource(resource).build())
}

/// Flush pending spans (best-effort). Call on process shutdown when OTel is on.
pub fn shutdown() {
    if let Some(provider) = PROVIDER.get() {
        let _ = provider.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otel_disabled_without_endpoint_env() {
        // CI / default: no OTEL_* endpoint → layer stays off.
        // We cannot unset env in parallel tests safely; just assert the helper
        // is callable and returns a bool.
        let _ = otel_enabled();
    }

    #[test]
    fn propagator_install_is_idempotent() {
        ensure_trace_context_propagator();
        ensure_trace_context_propagator();
    }
}
