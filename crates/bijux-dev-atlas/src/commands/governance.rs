// SPDX-License-Identifier: Apache-2.0

use crate::cli::{GovernanceCommand, GovernanceExceptionsCommand};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::docs::site_output::validate_named_report;
use bijux_dev_atlas::governance_objects::{
    collect_governance_objects, find_governance_object, governance_contract_coverage_path,
    governance_contract_coverage_payload, governance_coverage_path, governance_coverage_score,
    governance_drift_path, governance_drift_payload, governance_index_path,
    governance_index_payload, governance_lane_coverage_path, governance_lane_coverage_payload,
    governance_object_schema, governance_orphan_checks_path, governance_orphan_checks_payload,
    governance_orphan_report_path, governance_orphan_report_payload,
    governance_policy_surface_path, governance_policy_surface_payload, governance_summary_markdown,
    governance_summary_paths, validate_governance_objects,
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
            let index_path = governance_index_path(&root);
            let contract_coverage_path = governance_contract_coverage_path(&root);
            let lane_coverage_path = governance_lane_coverage_path(&root);
            let orphan_checks_path = governance_orphan_checks_path(&root);
            let policy_surface_path = governance_policy_surface_path(&root);
            let drift_path = governance_drift_path(&root);
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
            let previous_index = fs::read_to_string(&index_path)
                .ok()
                .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok());
            let index_payload = governance_index_payload(&root, &objects);
            validate_named_report(&root, "governance-index.schema.json", &index_payload)?;
            fs::write(
                &index_path,
                serde_json::to_string_pretty(&index_payload)
                    .map_err(|e| format!("encode governance index failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", index_path.display()))?;
            fs::write(
                &contract_coverage_path,
                serde_json::to_string_pretty(&governance_contract_coverage_payload(&root))
                    .map_err(|e| format!("encode contract coverage failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", contract_coverage_path.display()))?;
            fs::write(
                &lane_coverage_path,
                serde_json::to_string_pretty(&governance_lane_coverage_payload(&root))
                    .map_err(|e| format!("encode lane coverage failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", lane_coverage_path.display()))?;
            fs::write(
                &orphan_checks_path,
                serde_json::to_string_pretty(&governance_orphan_checks_payload(&root))
                    .map_err(|e| format!("encode orphan checks failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", orphan_checks_path.display()))?;
            fs::write(
                &policy_surface_path,
                serde_json::to_string_pretty(&governance_policy_surface_payload(&root))
                    .map_err(|e| format!("encode policy surface failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", policy_surface_path.display()))?;
            fs::write(
                &drift_path,
                serde_json::to_string_pretty(&governance_drift_payload(
                    &index_payload,
                    previous_index.as_ref(),
                ))
                .map_err(|e| format!("encode governance drift failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", drift_path.display()))?;
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
                    "governance_index": index_path.strip_prefix(&root).unwrap_or(&index_path).display().to_string(),
                    "governance_coverage": coverage_path.strip_prefix(&root).unwrap_or(&coverage_path).display().to_string(),
                    "contract_coverage_map": contract_coverage_path.strip_prefix(&root).unwrap_or(&contract_coverage_path).display().to_string(),
                    "lane_coverage_map": lane_coverage_path.strip_prefix(&root).unwrap_or(&lane_coverage_path).display().to_string(),
                    "orphan_checks": orphan_checks_path.strip_prefix(&root).unwrap_or(&orphan_checks_path).display().to_string(),
                    "governance_orphans": orphan_path.strip_prefix(&root).unwrap_or(&orphan_path).display().to_string(),
                    "policy_surface_map": policy_surface_path.strip_prefix(&root).unwrap_or(&policy_surface_path).display().to_string(),
                    "governance_drift": drift_path.strip_prefix(&root).unwrap_or(&drift_path).display().to_string(),
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
        GovernanceCommand::Exceptions { command } => match command {
            GovernanceExceptionsCommand::Validate {
                repo_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "governance_exceptions_validate",
                    "status": "ok",
                    "registry_path": root
                        .join("configs/governance/exceptions.yaml")
                        .strip_prefix(&root)
                        .unwrap_or_else(|_| std::path::Path::new("configs/governance/exceptions.yaml"))
                        .display()
                        .to_string(),
                    "text": "governance exceptions surface is wired; registry validation is handled by the upcoming governance exceptions implementation"
                });
                let rendered = emit_payload(format, out, &payload)?;
                Ok((rendered, 0))
            }
        },
    }
}
