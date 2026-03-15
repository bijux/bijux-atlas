// SPDX-License-Identifier: Apache-2.0

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::adapters::outbound::telemetry::logging::LoggingConfig;

#[derive(Debug, Clone)]
pub enum TraceExporterKind {
    Otlp,
    Jaeger,
    File,
    None,
}

#[derive(Debug, Clone)]
pub struct TraceConfig {
    pub logging: LoggingConfig,
    pub otel_enabled: bool,
    pub sampling_ratio: f64,
    pub exporter: TraceExporterKind,
    pub otlp_endpoint: Option<String>,
    pub jaeger_endpoint: Option<String>,
    pub trace_file_path: Option<String>,
    pub service_name: String,
}

#[derive(Clone)]
struct SharedFileWriter {
    file: Arc<Mutex<File>>,
}

impl SharedFileWriter {
    fn new(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("failed opening trace file: {e}"))?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }
}

impl Write for SharedFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("trace file writer lock poisoned"))?;
        file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("trace file writer lock poisoned"))?;
        file.flush()
    }
}

pub fn init_tracing(config: &TraceConfig) -> Result<(), String> {
    let default_directive = config.logging.default_filter_directive();
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_directive));
    if config.otel_enabled {
        match config.exporter {
            TraceExporterKind::Otlp => {
                let mut builder = opentelemetry_otlp::SpanExporter::builder().with_http();
                if let Some(endpoint) = &config.otlp_endpoint {
                    builder = builder.with_endpoint(endpoint);
                }
                let exporter = match builder.build() {
                    Ok(exporter) => exporter,
                    Err(err) => {
                        return init_plain_subscriber(config.logging.log_json, filter).map_err(|e| {
                            format!(
                                "failed to build OTLP span exporter ({err}); fallback subscriber failed: {e}"
                            )
                        });
                    }
                };
                let sampler = opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(
                    config.sampling_ratio.clamp(0.0, 1.0),
                );
                let resource =
                    opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                        "service.name",
                        config.service_name.clone(),
                    )]);
                let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
                    .with_sampler(sampler)
                    .with_resource(resource)
                    .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
                    .build()
                    .tracer("bijux-atlas-server");
                init_otel_subscriber(config.logging.log_json, filter, tracer)?;
            }
            TraceExporterKind::Jaeger => {
                let endpoint = config
                    .jaeger_endpoint
                    .clone()
                    .unwrap_or_else(|| "http://127.0.0.1:4318/v1/traces".to_string());
                let exporter = match opentelemetry_otlp::SpanExporter::builder()
                    .with_http()
                    .with_endpoint(endpoint)
                    .build()
                {
                    Ok(exporter) => exporter,
                    Err(err) => {
                        return init_plain_subscriber(config.logging.log_json, filter).map_err(|e| {
                            format!(
                                "failed to build Jaeger span exporter ({err}); fallback subscriber failed: {e}"
                            )
                        });
                    }
                };
                let sampler = opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(
                    config.sampling_ratio.clamp(0.0, 1.0),
                );
                let resource =
                    opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                        "service.name",
                        config.service_name.clone(),
                    )]);
                let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
                    .with_sampler(sampler)
                    .with_resource(resource)
                    .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
                    .build()
                    .tracer("bijux-atlas-server");
                init_otel_subscriber(config.logging.log_json, filter, tracer)?;
            }
            TraceExporterKind::File => {
                let file_path = config
                    .trace_file_path
                    .clone()
                    .unwrap_or_else(|| "artifacts/logs/atlas-trace.jsonl".to_string());
                let writer_path = std::path::PathBuf::from(file_path);
                if let Some(parent) = writer_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("failed creating trace file directory: {e}"))?;
                }
                let writer = SharedFileWriter::new(&writer_path)?;
                tracing_subscriber::registry()
                    .with(filter)
                    .with(
                        tracing_subscriber::fmt::layer()
                            .json()
                            .with_writer(move || writer.clone()),
                    )
                    .try_init()
                    .map_err(|e| format!("failed to initialize file tracing subscriber: {e}"))?;
            }
            TraceExporterKind::None => {
                init_plain_subscriber(config.logging.log_json, filter)?;
            }
        }
    } else {
        init_plain_subscriber(config.logging.log_json, filter)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SharedFileWriter;
    use std::io::Write;

    #[test]
    fn shared_trace_writer_appends_without_panicking() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("trace.jsonl");
        let mut writer_a = SharedFileWriter::new(&path).expect("writer a");
        let mut writer_b = writer_a.clone();

        writer_a.write_all(br#"{"event":"a"}"#).expect("write a");
        writer_b.write_all(br#"{"event":"b"}"#).expect("write b");
        writer_b.flush().expect("flush");

        let contents = std::fs::read_to_string(&path).expect("read trace file");
        assert!(contents.contains(r#"{"event":"a"}"#));
        assert!(contents.contains(r#"{"event":"b"}"#));
    }
}

fn init_otel_subscriber(
    log_json: bool,
    filter: EnvFilter,
    tracer: opentelemetry_sdk::trace::Tracer,
) -> Result<(), String> {
    if log_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .try_init()
            .map_err(|e| format!("failed to initialize otel subscriber: {e}"))?;
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .try_init()
            .map_err(|e| format!("failed to initialize otel subscriber: {e}"))?;
    }
    Ok(())
}

fn init_plain_subscriber(log_json: bool, filter: EnvFilter) -> Result<(), String> {
    if log_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .try_init()
            .map_err(|e| format!("failed to initialize tracing subscriber: {e}"))?;
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init()
            .map_err(|e| format!("failed to initialize tracing subscriber: {e}"))?;
    }
    Ok(())
}
