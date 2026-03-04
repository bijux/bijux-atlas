// SPDX-License-Identifier: Apache-2.0

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Clone)]
pub enum TraceExporterKind {
    Otlp,
    None,
}

#[derive(Debug, Clone)]
pub struct TraceConfig {
    pub log_json: bool,
    pub otel_enabled: bool,
    pub sampling_ratio: f64,
    pub exporter: TraceExporterKind,
    pub otlp_endpoint: Option<String>,
    pub service_name: String,
}

pub fn init_tracing(config: &TraceConfig) -> Result<(), String> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if config.otel_enabled {
        match config.exporter {
            TraceExporterKind::Otlp => {
                let mut builder = opentelemetry_otlp::SpanExporter::builder().with_http();
                if let Some(endpoint) = &config.otlp_endpoint {
                    builder = builder.with_endpoint(endpoint);
                }
                let exporter = builder
                    .build()
                    .map_err(|e| format!("failed to build OTLP span exporter: {e}"))?;
                let sampler = opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(
                    config.sampling_ratio.clamp(0.0, 1.0),
                );
                let resource = opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                    "service.name",
                    config.service_name.clone(),
                )]);
                let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
                    .with_sampler(sampler)
                    .with_resource(resource)
                    .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
                    .build()
                    .tracer("bijux-atlas-server");
                if config.log_json {
                    tracing_subscriber::registry()
                        .with(filter)
                        .with(tracing_subscriber::fmt::layer().json())
                        .with(tracing_opentelemetry::layer().with_tracer(tracer))
                        .init();
                } else {
                    tracing_subscriber::registry()
                        .with(filter)
                        .with(tracing_subscriber::fmt::layer())
                        .with(tracing_opentelemetry::layer().with_tracer(tracer))
                        .init();
                }
            }
            TraceExporterKind::None => {
                if config.log_json {
                    tracing_subscriber::registry()
                        .with(filter)
                        .with(tracing_subscriber::fmt::layer().json())
                        .init();
                } else {
                    tracing_subscriber::registry()
                        .with(filter)
                        .with(tracing_subscriber::fmt::layer())
                        .init();
                }
            }
        }
    } else if config.log_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
    Ok(())
}
