// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ObserveCommand, ObserveMetricsCommand, ObserveMetricsCommonArgs, ObserveTracesCommand,
    ObserveTracesCommonArgs,
};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::contracts::{metrics_registry, tracing_registry};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const LABEL_BUDGET_POLICY: &str = "ops/observe/metrics/label-cardinality-budget.json";
const REGISTRY_SNAPSHOT_ARTIFACT: &str = "artifacts/observe/metrics-registry-snapshot.json";

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
        let key = serde_json::to_value(row.category)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".to_string());
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
    let mut text = Vec::<String>::new();
    text.push("# Metrics Registry Reference".to_string());
    text.push("".to_string());
    text.push("Generated by `bijux-dev-atlas observe metrics docs`.".to_string());
    text.push("".to_string());
    text.push("| Metric ID | Name | Category | Unit | Stability |".to_string());
    text.push("|---|---|---|---|---|".to_string());
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
        ObserveCommand::Traces { command } => match command {
            ObserveTracesCommand::Explain(args) => explain_traces(args),
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
