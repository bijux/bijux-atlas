// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ObserveCommand, ObserveDashboardsCommand, ObserveDashboardsCommonArgs, ObserveLogsCommand,
    ObserveLogsCommonArgs, ObserveMetricsCommand, ObserveMetricsCommonArgs, ObserveTracesCommand,
    ObserveTracesCommonArgs,
};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::reference::{logging_registry, metrics_registry, tracing_registry};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const LABEL_BUDGET_POLICY: &str = "ops/observe/metrics/label-cardinality-budget.json";
const REGISTRY_SNAPSHOT_ARTIFACT: &str = "artifacts/observe/metrics-registry-snapshot.json";
const TRACE_SPAN_REGISTRY: &str = "ops/observe/tracing/span-registry.json";
const TRACE_CORRELATION_POLICY: &str = "ops/observe/tracing/correlation-policy.json";
const TRACE_STABILITY_CONTRACT: &str = "ops/observe/contracts/tracing-stability-contract.json";
const TRACE_COVERAGE_REPORT_ARTIFACT: &str = "artifacts/observe/trace-coverage-report.json";
const TRACE_COVERAGE_SUMMARY_ARTIFACT: &str = "artifacts/observe/trace-coverage-summary.md";
const TRACE_TOPOLOGY_ARTIFACT: &str = "artifacts/observe/trace-topology-diagram.mmd";
const LOG_FORMAT_VALIDATOR_CONTRACT: &str = "ops/observe/logging/format-validator-contract.json";
const LOG_FIELDS_CONTRACT: &str = "ops/observe/contracts/logs-fields-contract.json";
const LOG_SAMPLE_CONTRACT: &str = "ops/observe/contracts/logs-sample.jsonl";
const DASHBOARD_REGISTRY: &str = "ops/observe/dashboard-registry.json";
const DASHBOARD_METADATA_SCHEMA: &str = "ops/observe/dashboard-metadata.schema.json";
const DASHBOARD_VALIDATION_CONTRACT: &str =
    "ops/observe/contracts/dashboard-json-validation-contract.json";
const DASHBOARD_COVERAGE_ARTIFACT: &str = "artifacts/observe/dashboard-coverage-report.json";
const DASHBOARD_HEALTH_SUMMARY_ARTIFACT: &str = "artifacts/observe/dashboard-health-summary.json";
const OPERATIONAL_READINESS_ARTIFACT: &str = "artifacts/observe/operational-readiness-report.json";
const TELEMETRY_SUMMARY_ARTIFACT: &str = "artifacts/observe/operational-telemetry-summary.json";

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn write_json(path: &Path, payload: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(payload)
        .map_err(|err| format!("failed to encode {}: {err}", path.display()))?;
    fs::write(path, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn validate_registry_labels_and_budget(root: &Path) -> Result<Vec<String>, String> {
    let policy = read_json(&root.join(LABEL_BUDGET_POLICY))?;
    let allowed = policy
        .get("allowed_labels")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<BTreeSet<_>>();
    let max_budget = policy
        .get("max_cardinality_budget")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(200) as u32;

    let mut errors = Vec::new();
    for metric in metrics_registry::registry() {
        if metric.cardinality_budget > max_budget {
            errors.push(format!(
                "{} exceeds max cardinality budget {}",
                metric.id, max_budget
            ));
        }
        for label in metric.labels {
            if !allowed.contains(*label) {
                errors.push(format!("{} uses unknown label `{}`", metric.id, label));
            }
        }
    }
    Ok(errors)
}

fn snapshot_payload() -> serde_json::Value {
    let rows = metrics_registry::registry();
    let mut category_counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in &rows {
        let key = format!("{:?}", row.category).to_ascii_lowercase();
        *category_counts.entry(key).or_insert(0) += 1;
    }
    serde_json::json!({
        "schema_version": 1,
        "kind": "metrics_registry_snapshot",
        "metrics": rows,
        "summary": {
            "count": rows.len(),
            "by_category": category_counts,
        }
    })
}

fn list_metrics(common: ObserveMetricsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let snapshot = snapshot_payload();
    let snapshot_path = root.join(REGISTRY_SNAPSHOT_ARTIFACT);
    write_json(&snapshot_path, &snapshot)?;

    let budget_errors = validate_registry_labels_and_budget(&root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_metrics_list",
        "status": if budget_errors.is_empty() { "ok" } else { "failed" },
        "rows": snapshot.get("metrics").cloned().unwrap_or_else(|| serde_json::json!([])),
        "budget_validation_errors": budget_errors,
        "artifacts": {
            "metrics_registry_snapshot": REGISTRY_SNAPSHOT_ARTIFACT,
        }
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    let code = if payload["status"] == "ok" { 0 } else { 2 };
    Ok((rendered, code))
}

fn explain_metric(
    id_or_name: String,
    common: ObserveMetricsCommonArgs,
) -> Result<(String, i32), String> {
    let mut rows = metrics_registry::registry();
    rows.sort_by(|a, b| a.id.cmp(b.id));
    let found = rows
        .into_iter()
        .find(|row| row.id == id_or_name || row.name == id_or_name);
    let payload = if let Some(metric) = found {
        serde_json::json!({
            "schema_version": 1,
            "kind": "observe_metrics_explain",
            "status": "ok",
            "metric": metric,
        })
    } else {
        serde_json::json!({
            "schema_version": 1,
            "kind": "observe_metrics_explain",
            "status": "failed",
            "error": format!("unknown metric `{}`", id_or_name),
        })
    };
    let rendered = emit_payload(common.format, common.out, &payload)?;
    let code = if payload["status"] == "ok" { 0 } else { 2 };
    Ok((rendered, code))
}

fn generate_docs(common: ObserveMetricsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let mut rows = metrics_registry::registry();
    rows.sort_by(|a, b| a.id.cmp(b.id));
    let mut text = vec![
        "# Metrics Registry Reference".to_string(),
        String::new(),
        "Generated by `bijux-dev-atlas observe metrics docs`.".to_string(),
        String::new(),
        "| Metric ID | Name | Category | Unit | Stability |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ];
    for row in rows {
        text.push(format!(
            "| {} | {} | {:?} | {} | {:?} |",
            row.id, row.name, row.category, row.unit, row.stability
        ));
    }
    let out_path = root.join("docs/operations/observability/metrics-registry-reference.md");
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&out_path, format!("{}\n", text.join("\n")))
        .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_metrics_docs",
        "status": "ok",
        "output": out_path.strip_prefix(&root).unwrap_or(&out_path).display().to_string(),
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn explain_traces(common: ObserveTracesCommonArgs) -> Result<(String, i32), String> {
    let _root = resolve_repo_root(common.repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_traces_explain",
        "status": "ok",
        "contract": tracing_registry::tracing_contract(),
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn trace_topology_mermaid() -> String {
    let rows = tracing_registry::span_registry();
    let mut out = vec![
        "flowchart TD".to_string(),
        "  RUNTIME[runtime.root] --> REQUEST[http.request]".to_string(),
    ];
    for row in rows {
        if row.span_name == "runtime.root" || row.span_name == "http.request" {
            continue;
        }
        let node_id = row.span_name.replace('.', "_").to_uppercase();
        out.push(format!("  REQUEST --> {}[{}]", node_id, row.span_name));
    }
    out.join("\n")
}

fn trace_coverage_payload() -> serde_json::Value {
    let contract = tracing_registry::tracing_contract();
    let mut registry_spans = contract
        .span_registry
        .iter()
        .map(|row| row.span_name)
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for span in [
        "runtime.root",
        "http.request",
        "query.execution",
        "ingest.processing",
        "artifact.load",
        "registry.access",
        "configuration.load",
        "lifecycle.startup",
        "lifecycle.shutdown",
        "error.structured",
    ] {
        rows.push(serde_json::json!({
            "span_name": span,
            "covered": registry_spans.remove(span),
        }));
    }
    let covered = rows
        .iter()
        .filter(|row| {
            row.get("covered")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .count();
    serde_json::json!({
        "schema_version": 1,
        "kind": "trace_coverage_report",
        "coverage": rows,
        "summary": {
            "required_count": 10,
            "covered_count": covered,
            "coverage_ratio": (covered as f64) / 10.0,
        }
    })
}

fn write_trace_coverage_summary(
    root: &Path,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let rows = payload
        .get("coverage")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut text = vec![
        "# Trace Coverage Summary".to_string(),
        "".to_string(),
        "| Span | Covered |".to_string(),
        "|---|---|".to_string(),
    ];
    for row in rows {
        let Some(span) = row.get("span_name").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let covered = row
            .get("covered")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        text.push(format!(
            "| {} | {} |",
            span,
            if covered { "yes" } else { "no" }
        ));
    }
    let out_path = root.join(TRACE_COVERAGE_SUMMARY_ARTIFACT);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&out_path, format!("{}\n", text.join("\n")))
        .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    Ok(out_path
        .strip_prefix(root)
        .unwrap_or(&out_path)
        .display()
        .to_string())
}

fn run_trace_integrity_checks(root: &Path) -> Result<Vec<String>, String> {
    let mut violations = Vec::new();
    let span_registry = read_json(&root.join(TRACE_SPAN_REGISTRY))?;
    let expected = span_registry
        .get("required_spans")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<BTreeSet<_>>();
    let actual = tracing_registry::span_registry()
        .into_iter()
        .map(|row| row.span_name.to_string())
        .collect::<BTreeSet<_>>();
    for span in expected.difference(&actual) {
        violations.push(format!("missing required trace span `{span}`"));
    }
    for span in actual.difference(&expected) {
        violations.push(format!("untracked trace span `{span}`"));
    }

    let correlation = read_json(&root.join(TRACE_CORRELATION_POLICY))?;
    let correlation_has_request = correlation
        .get("request_id")
        .and_then(serde_json::Value::as_str)
        .map(|text| text.contains("required"))
        .unwrap_or(false);
    if !correlation_has_request {
        violations.push("trace correlation policy must require request identifiers".to_string());
    }

    let stability = read_json(&root.join(TRACE_STABILITY_CONTRACT))?;
    let stable_ids = stability
        .get("stable_trace_ids")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<BTreeSet<_>>();
    let current_ids = tracing_registry::span_registry()
        .into_iter()
        .map(|row| row.id.to_string())
        .collect::<BTreeSet<_>>();
    for missing in stable_ids.difference(&current_ids) {
        violations.push(format!(
            "stable trace id missing from contract registry `{missing}`"
        ));
    }

    Ok(violations)
}

fn coverage_traces(common: ObserveTracesCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let payload = trace_coverage_payload();
    write_json(&root.join(TRACE_COVERAGE_REPORT_ARTIFACT), &payload)?;
    let summary_path = write_trace_coverage_summary(&root, &payload)?;
    let result = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_traces_coverage",
        "status": "ok",
        "report": payload,
        "artifacts": {
            "coverage_report": TRACE_COVERAGE_REPORT_ARTIFACT,
            "coverage_summary": summary_path,
        }
    });
    let rendered = emit_payload(common.format, common.out, &result)?;
    Ok((rendered, 0))
}

fn topology_traces(common: ObserveTracesCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let topology = trace_topology_mermaid();
    let out_path = root.join(TRACE_TOPOLOGY_ARTIFACT);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&out_path, format!("{topology}\n"))
        .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_traces_topology",
        "status": "ok",
        "artifact": TRACE_TOPOLOGY_ARTIFACT,
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn verify_traces(common: ObserveTracesCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let violations = run_trace_integrity_checks(&root)?;
    let coverage = trace_coverage_payload();
    write_json(&root.join(TRACE_COVERAGE_REPORT_ARTIFACT), &coverage)?;
    write_trace_coverage_summary(&root, &coverage)?;
    let topology = trace_topology_mermaid();
    let topology_path = root.join(TRACE_TOPOLOGY_ARTIFACT);
    if let Some(parent) = topology_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&topology_path, format!("{topology}\n"))
        .map_err(|err| format!("failed to write {}: {err}", topology_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_traces_verify",
        "status": if violations.is_empty() { "ok" } else { "failed" },
        "violations": violations,
        "artifacts": {
            "coverage_report": TRACE_COVERAGE_REPORT_ARTIFACT,
            "coverage_summary": TRACE_COVERAGE_SUMMARY_ARTIFACT,
            "topology_diagram": TRACE_TOPOLOGY_ARTIFACT,
        }
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    let code = if payload["status"] == "ok" { 0 } else { 2 };
    Ok((rendered, code))
}

fn explain_logs(common: ObserveLogsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let fields_contract = read_json(&root.join(LOG_FIELDS_CONTRACT))?;
    let format_validator = read_json(&root.join(LOG_FORMAT_VALIDATOR_CONTRACT))?;
    let sample_rows = fs::read_to_string(root.join(LOG_SAMPLE_CONTRACT))
        .map_err(|err| format!("failed to read {LOG_SAMPLE_CONTRACT}: {err}"))?
        .lines()
        .take(3)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .unwrap_or_else(|_| serde_json::json!({}))
        })
        .collect::<Vec<_>>();
    let classification = logging_registry::summarize_classes(&sample_rows);
    let sample_validation = sample_rows
        .iter()
        .map(logging_registry::validate_log_record)
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_logs_explain",
        "status": "ok",
        "logging_contract": logging_registry::schema_contract(),
        "ops_contracts": {
            "log_fields": fields_contract,
            "format_validator": format_validator,
        },
        "sample_classification": classification,
        "sample_validation": sample_validation,
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn list_dashboards(common: ObserveDashboardsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let registry = read_json(&root.join(DASHBOARD_REGISTRY))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_dashboards_list",
        "status": "ok",
        "registry": registry,
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn verify_dashboards(common: ObserveDashboardsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let contract = read_json(&root.join(DASHBOARD_VALIDATION_CONTRACT))?;
    let schema = read_json(&root.join("ops/schema/observe/dashboard.schema.json"))?;
    let dashboards = contract
        .get("dashboards")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut validation_rows = Vec::new();
    let mut missing = Vec::new();
    for row in dashboards {
        let rel = row.as_str().unwrap_or_default();
        let abs = root.join(rel);
        if !abs.exists() {
            missing.push(rel.to_string());
            validation_rows.push(serde_json::json!({
                "path": rel,
                "status": "missing",
            }));
            continue;
        }
        let payload = read_json(&abs)?;
        let has_title = payload.get("title").is_some();
        let has_uid = payload.get("uid").is_some();
        let has_panels = payload
            .get("panels")
            .and_then(serde_json::Value::as_array)
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        validation_rows.push(serde_json::json!({
            "path": rel,
            "status": if has_title && has_uid && has_panels { "ok" } else { "failed" },
            "checks": {
                "title": has_title,
                "uid": has_uid,
                "panels": has_panels
            }
        }));
    }
    let covered = validation_rows
        .iter()
        .filter(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("ok"))
        .count();
    let coverage_payload = serde_json::json!({
        "schema_version": 1,
        "kind": "dashboard_coverage_report",
        "covered_count": covered,
        "required_count": validation_rows.len(),
        "rows": validation_rows,
    });
    write_json(&root.join(DASHBOARD_COVERAGE_ARTIFACT), &coverage_payload)?;
    write_json(
        &root.join(DASHBOARD_HEALTH_SUMMARY_ARTIFACT),
        &serde_json::json!({
            "schema_version": 1,
            "kind": "dashboard_health_summary",
            "status": if missing.is_empty() { "ok" } else { "failed" },
            "missing_dashboards": missing,
            "coverage_ratio": if coverage_payload["required_count"].as_u64().unwrap_or(0) == 0 { 0.0 } else { covered as f64 / coverage_payload["required_count"].as_u64().unwrap_or(1) as f64 },
        }),
    )?;
    write_json(
        &root.join(OPERATIONAL_READINESS_ARTIFACT),
        &serde_json::json!({
            "schema_version": 1,
            "kind": "operational_readiness_report",
            "dashboard_contract": DASHBOARD_VALIDATION_CONTRACT,
            "dashboard_schema": "ops/schema/observe/dashboard.schema.json",
            "ready": missing.is_empty(),
        }),
    )?;
    write_json(
        &root.join(TELEMETRY_SUMMARY_ARTIFACT),
        &serde_json::json!({
            "schema_version": 1,
            "kind": "operational_telemetry_summary",
            "dashboard_count": coverage_payload["required_count"],
            "metrics_contract": "ops/observe/contracts/metrics-contract.json",
            "trace_contract": TRACE_STABILITY_CONTRACT,
            "log_contract": LOG_FIELDS_CONTRACT,
        }),
    )?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_dashboards_verify",
        "status": if missing.is_empty() { "ok" } else { "failed" },
        "schema": schema,
        "artifacts": {
            "dashboard_coverage_report": DASHBOARD_COVERAGE_ARTIFACT,
            "dashboard_health_summary": DASHBOARD_HEALTH_SUMMARY_ARTIFACT,
            "operational_readiness_report": OPERATIONAL_READINESS_ARTIFACT,
            "operational_telemetry_summary": TELEMETRY_SUMMARY_ARTIFACT,
        }
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    let code = if missing.is_empty() { 0 } else { 2 };
    Ok((rendered, code))
}

fn explain_dashboards(common: ObserveDashboardsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let registry = read_json(&root.join(DASHBOARD_REGISTRY))?;
    let metadata_schema = read_json(&root.join(DASHBOARD_METADATA_SCHEMA))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "observe_dashboards_explain",
        "status": "ok",
        "registry": registry,
        "metadata_schema": metadata_schema,
        "validation_contract": DASHBOARD_VALIDATION_CONTRACT,
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_observe_command(
    _quiet: bool,
    command: ObserveCommand,
) -> Result<(String, i32), String> {
    match command {
        ObserveCommand::Metrics { command } => match command {
            ObserveMetricsCommand::List(args) => list_metrics(args),
            ObserveMetricsCommand::Explain(args) => explain_metric(args.id_or_name, args.common),
            ObserveMetricsCommand::Docs(args) => generate_docs(args),
        },
        ObserveCommand::Dashboards { command } => match command {
            ObserveDashboardsCommand::List(args) => list_dashboards(args),
            ObserveDashboardsCommand::Verify(args) => verify_dashboards(args),
            ObserveDashboardsCommand::Explain(args) => explain_dashboards(args),
        },
        ObserveCommand::Logs { command } => match command {
            ObserveLogsCommand::Explain(args) => explain_logs(args),
        },
        ObserveCommand::Traces { command } => match command {
            ObserveTracesCommand::Explain(args) => explain_traces(args),
            ObserveTracesCommand::Verify(args) => verify_traces(args),
            ObserveTracesCommand::Coverage(args) => coverage_traces(args),
            ObserveTracesCommand::Topology(args) => topology_traces(args),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_registry_has_expected_foundation_count() {
        let rows = metrics_registry::registry();
        assert!(rows.len() >= 20);
    }

    #[test]
    fn metrics_ids_are_unique_and_human_readable() {
        let rows = metrics_registry::registry();
        let mut seen = BTreeSet::new();
        for row in rows {
            assert!(seen.insert(row.id));
            assert!(row.description.contains(' '));
        }
    }
}
