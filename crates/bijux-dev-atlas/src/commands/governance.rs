// SPDX-License-Identifier: Apache-2.0

use crate::cli::GovernanceCommand;
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::governance_objects::{
    collect_governance_objects, find_governance_object, governance_coverage_path,
    governance_coverage_score, governance_object_schema, governance_orphan_report_path,
    governance_orphan_report_payload, governance_summary_markdown, governance_summary_paths,
    validate_governance_objects,
};
use std::fs;

pub(crate) fn run_governance_command(
    _quiet: bool,
    command: GovernanceCommand,
) -> Result<(String, i32), String> {
    match command {
        GovernanceCommand::List {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let objects = collect_governance_objects(&root)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_list",
                "schema": governance_object_schema(),
                "count": objects.len(),
                "objects": objects,
            });
            let rendered = emit_payload(format, out, &payload)?;
            Ok((rendered, 0))
        }
        GovernanceCommand::Explain {
            id,
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let objects = collect_governance_objects(&root)?;
            let Some(object) = find_governance_object(&objects, &id) else {
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "governance_explain",
                    "status": "not_found",
                    "id": id,
                });
                let rendered = emit_payload(format, out, &payload)?;
                return Ok((rendered, 1));
            };
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_explain",
                "status": "ok",
                "object": object,
            });
            let rendered = emit_payload(format, out, &payload)?;
            Ok((rendered, 0))
        }
        GovernanceCommand::Validate {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let objects = collect_governance_objects(&root)?;
            let validation = validate_governance_objects(&root, &objects);
            let (graph_path, summary_path) = governance_summary_paths(&root);
            let coverage_path = governance_coverage_path(&root);
            let orphan_path = governance_orphan_report_path(&root);
            if let Some(parent) = graph_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
            }
            fs::write(
                &graph_path,
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": 1,
                    "kind": "governance_graph",
                    "nodes": objects,
                }))
                .map_err(|e| format!("encode governance graph failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", graph_path.display()))?;
            fs::write(
                &summary_path,
                governance_summary_markdown(&collect_governance_objects(&root)?),
            )
            .map_err(|e| format!("write {} failed: {e}", summary_path.display()))?;
            let coverage_payload = governance_coverage_score(&objects);
            fs::write(
                &coverage_path,
                serde_json::to_string_pretty(&coverage_payload)
                    .map_err(|e| format!("encode governance coverage failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", coverage_path.display()))?;
            fs::write(
                &orphan_path,
                serde_json::to_string_pretty(&governance_orphan_report_payload(
                    &validation.orphan_rows,
                ))
                .map_err(|e| format!("encode governance orphan report failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", orphan_path.display()))?;

            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_validate",
                "status": if validation.errors.is_empty() {"ok"} else {"failed"},
                "objects": collect_governance_objects(&root)?,
                "errors": validation.errors,
                "artifacts": {
                    "governance_graph": graph_path.strip_prefix(&root).unwrap_or(&graph_path).display().to_string(),
                    "governance_summary": summary_path.strip_prefix(&root).unwrap_or(&summary_path).display().to_string(),
                    "governance_coverage": coverage_path.strip_prefix(&root).unwrap_or(&coverage_path).display().to_string(),
                    "governance_orphans": orphan_path.strip_prefix(&root).unwrap_or(&orphan_path).display().to_string(),
                }
            });
            let rendered = emit_payload(format, out, &payload)?;
            let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                1
            } else {
                0
            };
            Ok((rendered, code))
        }
    }
}
