// SPDX-License-Identifier: Apache-2.0
//! Canonical distributed tracing architecture contract and span registry.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceLayer {
    Runtime,
    Request,
    Query,
    Ingest,
    Artifact,
    Registry,
    Configuration,
    Lifecycle,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceStability {
    Stable,
    Experimental,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceSpanSpec {
    pub id: &'static str,
    pub span_name: &'static str,
    pub layer: TraceLayer,
    pub summary: &'static str,
    pub required_attributes: &'static [&'static str],
    pub async_propagation_required: bool,
    pub request_id_correlation: bool,
    pub logging_correlation: bool,
    pub metrics_correlation: bool,
    pub stability: TraceStability,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceContract {
    pub schema_version: u32,
    pub architecture: &'static str,
    pub context_propagation_policy: &'static str,
    pub span_naming_convention: &'static str,
    pub sampling_strategy: &'static str,
    pub retention_policy: &'static str,
    pub span_registry: Vec<TraceSpanSpec>,
}

pub fn span_registry() -> Vec<TraceSpanSpec> {
    let mut rows = vec![
        TraceSpanSpec {
            id: "TRACE-RUNTIME-ROOT-001",
            span_name: "runtime.root",
            layer: TraceLayer::Runtime,
            summary: "Top-level runtime root span for process lifecycle.",
            required_attributes: &["service.name", "service.version", "run_id"],
            async_propagation_required: true,
            request_id_correlation: false,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-REQUEST-ENTRY-001",
            span_name: "http.request",
            layer: TraceLayer::Request,
            summary: "Inbound request span at middleware entry.",
            required_attributes: &["request_id", "route", "method", "traceparent"],
            async_propagation_required: true,
            request_id_correlation: true,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-QUERY-EXEC-001",
            span_name: "query.execution",
            layer: TraceLayer::Query,
            summary: "Query execution span for core request path.",
            required_attributes: &["request_id", "dataset", "query_class"],
            async_propagation_required: true,
            request_id_correlation: true,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-INGEST-PIPELINE-001",
            span_name: "ingest.processing",
            layer: TraceLayer::Ingest,
            summary: "Ingest pipeline processing span.",
            required_attributes: &["pipeline", "dataset", "artifact_id"],
            async_propagation_required: true,
            request_id_correlation: false,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-ARTIFACT-LOAD-001",
            span_name: "artifact.load",
            layer: TraceLayer::Artifact,
            summary: "Artifact loading span with integrity context.",
            required_attributes: &["artifact_type", "artifact_path", "sha256"],
            async_propagation_required: true,
            request_id_correlation: true,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-REGISTRY-ACCESS-001",
            span_name: "registry.access",
            layer: TraceLayer::Registry,
            summary: "Registry read/write access span.",
            required_attributes: &["registry_name", "operation", "result"],
            async_propagation_required: true,
            request_id_correlation: true,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-CONFIG-LOAD-001",
            span_name: "configuration.load",
            layer: TraceLayer::Configuration,
            summary: "Configuration load and validation span.",
            required_attributes: &["config_source", "schema_version", "result"],
            async_propagation_required: false,
            request_id_correlation: false,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-STARTUP-INIT-001",
            span_name: "lifecycle.startup",
            layer: TraceLayer::Lifecycle,
            summary: "Startup initialization span.",
            required_attributes: &["build_version", "runtime_mode"],
            async_propagation_required: false,
            request_id_correlation: false,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-SHUTDOWN-001",
            span_name: "lifecycle.shutdown",
            layer: TraceLayer::Lifecycle,
            summary: "Graceful shutdown span.",
            required_attributes: &["signal", "drain_status"],
            async_propagation_required: false,
            request_id_correlation: false,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
        TraceSpanSpec {
            id: "TRACE-ERROR-STRUCTURED-001",
            span_name: "error.structured",
            layer: TraceLayer::Error,
            summary: "Structured error span with explicit classification attributes.",
            required_attributes: &["error_code", "error_class", "request_id"],
            async_propagation_required: true,
            request_id_correlation: true,
            logging_correlation: true,
            metrics_correlation: true,
            stability: TraceStability::Stable,
        },
    ];
    rows.sort_by(|a, b| a.id.cmp(b.id));
    rows
}

pub fn tracing_contract() -> TraceContract {
    TraceContract {
        schema_version: 1,
        architecture: "request-centric distributed tracing with explicit lifecycle, query, ingest, artifact, and registry spans",
        context_propagation_policy: "propagate traceparent and request_id across async boundaries and outbound calls",
        span_naming_convention: "dot-separated lower_snake_case names scoped by subsystem (for example: query.execution)",
        sampling_strategy: "ratio-based sampling with incident override and deterministic change tracking",
        retention_policy: "keep high-cardinality full-fidelity traces for short windows and aggregated traces for long-term analysis",
        span_registry: span_registry(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn trace_span_ids_and_names_are_unique() {
        let rows = span_registry();
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        for row in rows {
            assert!(ids.insert(row.id));
            assert!(names.insert(row.span_name));
        }
    }

    #[test]
    fn tracing_contract_includes_required_core_spans() {
        let ids = span_registry()
            .into_iter()
            .map(|row| row.id)
            .collect::<BTreeSet<_>>();
        for required in [
            "TRACE-RUNTIME-ROOT-001",
            "TRACE-REQUEST-ENTRY-001",
            "TRACE-QUERY-EXEC-001",
            "TRACE-INGEST-PIPELINE-001",
            "TRACE-ARTIFACT-LOAD-001",
            "TRACE-REGISTRY-ACCESS-001",
            "TRACE-CONFIG-LOAD-001",
            "TRACE-STARTUP-INIT-001",
            "TRACE-SHUTDOWN-001",
            "TRACE-ERROR-STRUCTURED-001",
        ] {
            assert!(ids.contains(required));
        }
    }

    #[test]
    fn tracing_sampling_strategy_is_ratio_and_deterministic() {
        let contract = tracing_contract();
        assert!(contract.sampling_strategy.contains("ratio"));
        assert!(contract.sampling_strategy.contains("deterministic"));
    }

    #[test]
    fn tracing_context_policy_mentions_traceparent_and_async_boundaries() {
        let contract = tracing_contract();
        assert!(contract.context_propagation_policy.contains("traceparent"));
        assert!(contract
            .context_propagation_policy
            .contains("async boundaries"));
    }

    #[test]
    fn error_span_declares_classification_attributes() {
        let error_span = span_registry()
            .into_iter()
            .find(|row| row.id == "TRACE-ERROR-STRUCTURED-001")
            .expect("error span");
        assert!(error_span.required_attributes.contains(&"error_code"));
        assert!(error_span.required_attributes.contains(&"error_class"));
    }
}
