// SPDX-License-Identifier: Apache-2.0
//! Human-readable terminal rendering.

use crate::engine;
use crate::model::engine::RunReport;
use crate::model::RunReport as ChecksRunReport;

pub mod nextest_style;
pub mod report;

pub fn render_contracts(report: &RunReport) -> String {
    engine::to_pretty(report)
}

pub fn render_checks(report: &ChecksRunReport) -> String {
    crate::core::render_text_summary(report)
}

pub fn render_suite_summary(summary: &serde_json::Value) -> String {
    let totals = &summary["summary"];
    let suite = summary["suite"].as_str().unwrap_or_default();
    let run_id = summary["run_id"].as_str().unwrap_or_default();
    let failed_ids = summary["failures"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("id").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();
    let summary_line = match (suite.is_empty(), run_id.is_empty()) {
        (false, false) => format!(
            "Suite {suite} run {run_id}: pass={} fail={} warn={} skip={} total={}",
            totals["pass"].as_u64().unwrap_or(0),
            totals["fail"].as_u64().unwrap_or(0),
            totals["warn"].as_u64().unwrap_or(0),
            totals["skip"].as_u64().unwrap_or(0),
            totals["total"].as_u64().unwrap_or(0)
        ),
        (false, true) => format!(
            "Suite {suite}: pass={} fail={} warn={} skip={} total={}",
            totals["pass"].as_u64().unwrap_or(0),
            totals["fail"].as_u64().unwrap_or(0),
            totals["warn"].as_u64().unwrap_or(0),
            totals["skip"].as_u64().unwrap_or(0),
            totals["total"].as_u64().unwrap_or(0)
        ),
        (true, false) => format!(
            "Run {run_id}: pass={} fail={} warn={} skip={} total={}",
            totals["pass"].as_u64().unwrap_or(0),
            totals["fail"].as_u64().unwrap_or(0),
            totals["warn"].as_u64().unwrap_or(0),
            totals["skip"].as_u64().unwrap_or(0),
            totals["total"].as_u64().unwrap_or(0)
        ),
        (true, true) => format!(
            "Suite summary: pass={} fail={} warn={} skip={} total={}",
            totals["pass"].as_u64().unwrap_or(0),
            totals["fail"].as_u64().unwrap_or(0),
            totals["warn"].as_u64().unwrap_or(0),
            totals["skip"].as_u64().unwrap_or(0),
            totals["total"].as_u64().unwrap_or(0)
        ),
    };
    let mut lines = vec![summary_line];
    if !failed_ids.is_empty() {
        lines.push(format!("Failed IDs: {}", failed_ids.join(", ")));
    }
    let artifact_root = summary["artifacts_root"].as_str().unwrap_or_default();
    if !artifact_root.is_empty() {
        lines.push(format!("Artifacts: {artifact_root}"));
    }
    lines.push(format!(
        "Next actions: {}",
        if failed_ids.is_empty() {
            "promote this run as a clean baseline"
        } else {
            "inspect failing result.json files and rerun affected entries"
        }
    ));
    lines.join("\n")
}

pub fn render_registry_status(report: &serde_json::Value) -> String {
    let summary = &report["summary"];
    let rows = report["rows"].as_array().map(Vec::len).unwrap_or(0);
    format!(
        "Registry status: rows={} errors={} warnings={} next={}",
        rows,
        summary["errors"].as_u64().unwrap_or(0),
        summary["warnings"].as_u64().unwrap_or(0),
        if summary["errors"].as_u64().unwrap_or(0) == 0 {
            "registry is ready"
        } else {
            "resolve missing metadata before merge"
        }
    )
}

pub fn failure_hint(code: &str) -> &'static str {
    match code {
        "not_found" => "check the selected id and rerun with atlas list or atlas describe",
        "required_failure" => "inspect required checks first; optional lanes may be noise",
        "schema_mismatch" => "compare the emitted report version to the registry schema version",
        "missing_tool" => "install the required tool or allow the engine to skip it explicitly",
        _ => "inspect the generated artifact and rerun with --debug for command details",
    }
}

#[cfg(test)]
mod tests {
    use super::render_suite_summary;

    #[test]
    fn suite_summary_uses_descriptive_headline_when_identifiers_are_missing() {
        let summary = serde_json::json!({
            "summary": {"pass": 1, "fail": 0, "warn": 0, "skip": 0, "total": 1},
            "failures": [],
            "artifacts_root": ""
        });
        let rendered = render_suite_summary(&summary);
        assert!(rendered.starts_with("Suite summary:"));
        assert!(!rendered.contains("unknown"));
    }
}
