// SPDX-License-Identifier: Apache-2.0

use crate::cli::{GovernanceCommand, GovernanceExceptionsCommand};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::core::load_registry;
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
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct ExceptionsRegistry {
    schema_version: u64,
    policy: ExceptionsPolicy,
    reason_taxonomy: Vec<String>,
    exceptions: Vec<GovernanceException>,
}

#[derive(Debug, Deserialize)]
struct ExceptionsPolicy {
    max_days_by_severity: MaxDaysBySeverity,
    warning_days: i64,
    allowed_tracking_domains: Vec<String>,
    no_exception_zones: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MaxDaysBySeverity {
    low: i64,
    medium: i64,
    high: i64,
}

#[derive(Debug, Deserialize)]
struct GovernanceException {
    id: String,
    scope: ExceptionScope,
    severity: String,
    reason: String,
    owner: String,
    created_at: String,
    expires_at: String,
    mitigation: String,
    tracking_link: String,
    risk_accepted_by: String,
    verification_plan: String,
    governance_approval: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ExceptionScope {
    kind: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct ExceptionsArchive {
    schema_version: u64,
    archived_exceptions: Vec<ArchivedException>,
}

#[derive(Debug, Deserialize)]
struct ArchivedException {
    id: String,
    scope: ExceptionScope,
    severity: String,
    reason: String,
    owner: String,
    created_at: String,
    expires_at: String,
    mitigation: String,
    tracking_link: String,
    risk_accepted_by: String,
    verification_plan: String,
    archived_at: String,
    content_sha256: String,
}

fn exceptions_registry_path(root: &Path) -> PathBuf {
    root.join("configs/governance/exceptions.yaml")
}

fn exceptions_registry_schema_path(root: &Path) -> PathBuf {
    root.join("configs/contracts/governance/exceptions.schema.json")
}

fn exceptions_archive_path(root: &Path) -> PathBuf {
    root.join("configs/governance/exceptions-archive.yaml")
}

fn exceptions_archive_schema_path(root: &Path) -> PathBuf {
    root.join("configs/contracts/governance/exceptions-archive.schema.json")
}

fn exceptions_summary_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/exceptions-summary.json")
}

fn exceptions_table_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/exceptions-table.md")
}

fn exceptions_warning_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/exceptions-expiry-warning.json")
}

fn exceptions_churn_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/exceptions-churn.json")
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| matches!(idx, 4 | 7) || byte.is_ascii_digit())
}

fn parse_ymd(value: &str) -> Option<(i64, i64, i64)> {
    if !is_iso_date(value) {
        return None;
    }
    let year = value.get(0..4)?.parse().ok()?;
    let month = value.get(5..7)?.parse().ok()?;
    let day = value.get(8..10)?.parse().ok()?;
    Some((year, month, day))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_adj = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_adj + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn current_utc_day() -> Result<i64, String> {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system clock before unix epoch: {err}"))?
        .as_secs() as i64;
    Ok(secs / 86_400)
}

fn date_days(value: &str) -> Result<i64, String> {
    let (year, month, day) =
        parse_ymd(value).ok_or_else(|| format!("invalid date `{value}`; expected YYYY-MM-DD"))?;
    Ok(days_from_civil(year, month, day))
}

fn tracking_domain(url: &str) -> Option<&str> {
    let (_, rest) = url.split_once("://")?;
    Some(rest.split('/').next()?.split(':').next()?)
}

fn write_pretty_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value).map_err(|err| format!("encode json failed: {err}"))?,
    )
    .map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    fs::write(path, text).map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn known_contract_ids(root: &Path) -> Result<BTreeSet<String>, String> {
    let contracts_path = root.join("ops/inventory/contracts.json");
    let value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&contracts_path)
            .map_err(|err| format!("read {} failed: {err}", contracts_path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", contracts_path.display()))?;
    Ok(value
        .get("contract_ids")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect())
}

fn known_check_ids(root: &Path) -> Result<BTreeSet<String>, String> {
    Ok(load_registry(root)?
        .checks
        .into_iter()
        .map(|check| check.id.as_str().to_string())
        .collect())
}

fn render_exceptions_table(rows: &[serde_json::Value]) -> String {
    let mut out = String::from("# Governance Exceptions\n\n");
    out.push_str("Read-only generated view from `configs/governance/exceptions.yaml`.\n\n");
    out.push_str("| Id | Scope | Severity | Owner | Expires | Days left |\n|---|---|---|---|---|---|\n");
    for row in rows {
        let id = row.get("id").and_then(serde_json::Value::as_str).unwrap_or_default();
        let scope = row
            .get("scope")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let severity = row
            .get("severity")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let owner = row
            .get("owner")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let expires = row
            .get("expires_at")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let days_left = row
            .get("days_to_expiry")
            .and_then(serde_json::Value::as_i64)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "n/a".to_string());
        out.push_str(&format!(
            "| `{id}` | `{scope}` | `{severity}` | `{owner}` | `{expires}` | `{days_left}` |\n"
        ));
    }
    out
}

fn stable_exception_digest(value: &serde_json::Value) -> Result<String, String> {
    let bytes =
        serde_json::to_vec(value).map_err(|err| format!("encode exception digest failed: {err}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

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
                let schema_path = exceptions_registry_schema_path(&root);
                let _: serde_json::Value = serde_json::from_str(
                    &fs::read_to_string(&schema_path)
                        .map_err(|err| format!("read {} failed: {err}", schema_path.display()))?,
                )
                .map_err(|err| format!("parse {} failed: {err}", schema_path.display()))?;
                let archive_schema_path = exceptions_archive_schema_path(&root);
                let _: serde_json::Value = serde_json::from_str(
                    &fs::read_to_string(&archive_schema_path).map_err(|err| {
                        format!("read {} failed: {err}", archive_schema_path.display())
                    })?,
                )
                .map_err(|err| format!("parse {} failed: {err}", archive_schema_path.display()))?;
                let registry_path = exceptions_registry_path(&root);
                let registry_text = fs::read_to_string(&registry_path)
                    .map_err(|err| format!("read {} failed: {err}", registry_path.display()))?;
                let registry: ExceptionsRegistry = serde_yaml::from_str(&registry_text)
                    .map_err(|err| format!("parse {} failed: {err}", registry_path.display()))?;
                let archive_path = exceptions_archive_path(&root);
                let archive_text = fs::read_to_string(&archive_path)
                    .map_err(|err| format!("read {} failed: {err}", archive_path.display()))?;
                let archive: ExceptionsArchive = serde_yaml::from_str(&archive_text)
                    .map_err(|err| format!("parse {} failed: {err}", archive_path.display()))?;
                let summary_path = exceptions_summary_path(&root);
                let table_path = exceptions_table_path(&root);
                let warning_path = exceptions_warning_path(&root);
                let churn_path = exceptions_churn_path(&root);
                let known_contracts = known_contract_ids(&root)?;
                let known_checks = known_check_ids(&root)?;
                let today = current_utc_day()?;
                let no_exception_zones: BTreeSet<String> = registry
                    .policy
                    .no_exception_zones
                    .iter()
                    .cloned()
                    .collect();
                let allowed_domains: BTreeSet<String> = registry
                    .policy
                    .allowed_tracking_domains
                    .iter()
                    .cloned()
                    .collect();
                let reason_taxonomy: BTreeSet<String> =
                    registry.reason_taxonomy.iter().cloned().collect();
                let mut errors = Vec::new();
                let mut rows = Vec::new();
                let mut seen_ids = BTreeSet::new();
                let mut warning_rows = Vec::new();

                for item in &registry.exceptions {
                    if !seen_ids.insert(item.id.clone()) {
                        errors.push(format!("duplicate exception id `{}`", item.id));
                    }
                    if item.owner.trim().is_empty() {
                        errors.push(format!("{} missing owner", item.id));
                    }
                    if item.mitigation.trim().is_empty() {
                        errors.push(format!("{} missing mitigation", item.id));
                    }
                    if item.risk_accepted_by.trim().is_empty() {
                        errors.push(format!("{} missing risk_accepted_by", item.id));
                    }
                    if item.verification_plan.trim().is_empty() {
                        errors.push(format!("{} missing verification_plan", item.id));
                    }
                    if !reason_taxonomy.contains(&item.reason) {
                        errors.push(format!("{} uses unknown reason `{}`", item.id, item.reason));
                    }
                    if !is_iso_date(&item.created_at) {
                        errors.push(format!("{} has invalid created_at `{}`", item.id, item.created_at));
                    }
                    if !is_iso_date(&item.expires_at) {
                        errors.push(format!("{} has invalid expires_at `{}`", item.id, item.expires_at));
                    }
                    let domain = tracking_domain(&item.tracking_link).unwrap_or_default();
                    if domain.is_empty() || !allowed_domains.contains(domain) {
                        errors.push(format!(
                            "{} tracking link domain `{}` is not allowlisted",
                            item.id, domain
                        ));
                    }
                    let scope_key = item.scope.id.clone();
                    if no_exception_zones.contains(&scope_key) {
                        errors.push(format!(
                            "{} targets no-exception zone `{}`",
                            item.id, scope_key
                        ));
                    }
                    match item.scope.kind.as_str() {
                        "contract" => {
                            if !known_contracts.contains(&item.scope.id) {
                                errors.push(format!(
                                    "{} references unknown contract id `{}`",
                                    item.id, item.scope.id
                                ));
                            }
                        }
                        "check" => {
                            if !known_checks.contains(&item.scope.id) {
                                errors.push(format!(
                                    "{} references unknown check id `{}`",
                                    item.id, item.scope.id
                                ));
                            }
                        }
                        other => errors.push(format!(
                            "{} uses invalid scope.kind `{}`",
                            item.id, other
                        )),
                    }

                    let expires_days = date_days(&item.expires_at)?;
                    let created_days = date_days(&item.created_at)?;
                    if expires_days < created_days {
                        errors.push(format!(
                            "{} expires before it is created ({} < {})",
                            item.id, item.expires_at, item.created_at
                        ));
                    }
                    let days_to_expiry = expires_days - today;
                    if days_to_expiry < 0 {
                        errors.push(format!("{} expired on {}", item.id, item.expires_at));
                    }
                    let max_days = match item.severity.as_str() {
                        "low" => registry.policy.max_days_by_severity.low,
                        "medium" => registry.policy.max_days_by_severity.medium,
                        "high" => registry.policy.max_days_by_severity.high,
                        other => {
                            errors.push(format!("{} uses unknown severity `{}`", item.id, other));
                            0
                        }
                    };
                    let duration_days = expires_days - created_days;
                    if duration_days > max_days
                        && !(item.severity == "high" && item.governance_approval == Some(true))
                    {
                        errors.push(format!(
                            "{} exceeds {}-day limit for severity `{}`",
                            item.id, max_days, item.severity
                        ));
                    }
                    if days_to_expiry < registry.policy.warning_days && days_to_expiry >= 0 {
                        warning_rows.push(serde_json::json!({
                            "id": item.id,
                            "scope": format!("{}:{}", item.scope.kind, item.scope.id),
                            "owner": item.owner,
                            "expires_at": item.expires_at,
                            "days_to_expiry": days_to_expiry
                        }));
                    }

                    rows.push(serde_json::json!({
                        "id": item.id,
                        "scope": format!("{}:{}", item.scope.kind, item.scope.id),
                        "severity": item.severity,
                        "reason": item.reason,
                        "owner": item.owner,
                        "created_at": item.created_at,
                        "expires_at": item.expires_at,
                        "days_to_expiry": days_to_expiry,
                        "tracking_link": item.tracking_link,
                        "risk_accepted_by": item.risk_accepted_by,
                        "verification_plan": item.verification_plan,
                        "governance_approval": item.governance_approval.unwrap_or(false)
                    }));
                }

                let mut archived_ids = BTreeSet::new();
                for item in &archive.archived_exceptions {
                    if !archived_ids.insert(item.id.clone()) {
                        errors.push(format!("duplicate archived exception id `{}`", item.id));
                    }
                    if !is_iso_date(&item.archived_at) {
                        errors.push(format!(
                            "{} has invalid archived_at `{}`",
                            item.id, item.archived_at
                        ));
                    }
                    if item.verification_plan.trim().is_empty() {
                        errors.push(format!(
                            "{} archived entry missing verification_plan",
                            item.id
                        ));
                    }
                    let digest_input = serde_json::json!({
                        "id": item.id,
                        "scope": {"kind": item.scope.kind, "id": item.scope.id},
                        "severity": item.severity,
                        "reason": item.reason,
                        "owner": item.owner,
                        "created_at": item.created_at,
                        "expires_at": item.expires_at,
                        "mitigation": item.mitigation,
                        "tracking_link": item.tracking_link,
                        "risk_accepted_by": item.risk_accepted_by,
                        "verification_plan": item.verification_plan,
                        "archived_at": item.archived_at
                    });
                    let actual_digest = stable_exception_digest(&digest_input)?;
                    if actual_digest != item.content_sha256 {
                        errors.push(format!(
                            "{} archived entry content_sha256 does not match frozen content",
                            item.id
                        ));
                    }
                }

                rows.sort_by(|left, right| {
                    left["id"]
                        .as_str()
                        .unwrap_or_default()
                        .cmp(right["id"].as_str().unwrap_or_default())
                });

                let active_rows = rows
                    .iter()
                    .filter(|row| row["days_to_expiry"].as_i64().unwrap_or(-1) >= 0)
                    .count();
                let warning_report = serde_json::json!({
                    "report_id": "exceptions-expiry-warning",
                    "version": 1,
                    "inputs": {
                        "generator": "bijux-dev-atlas governance exceptions validate",
                        "sources": ["configs/governance/exceptions.yaml"]
                    },
                    "status": "ok",
                    "summary": {
                        "total": warning_rows.len(),
                        "warning_days": registry.policy.warning_days
                    },
                    "rows": warning_rows,
                    "contracts": {
                        "GOV-EXC-007": true
                    },
                    "errors": []
                });
                validate_named_report(&root, "exceptions-expiry-warning.schema.json", &warning_report)?;
                write_pretty_json(&warning_path, &warning_report)?;
                let churn_rows = archive
                    .archived_exceptions
                    .iter()
                    .map(|item| {
                        serde_json::json!({
                            "id": item.id,
                            "status": if seen_ids.contains(&item.id) { "restored" } else { "archived" },
                            "archived_at": item.archived_at,
                            "scope": format!("{}:{}", item.scope.kind, item.scope.id)
                        })
                    })
                    .collect::<Vec<_>>();
                let churn_report = serde_json::json!({
                    "report_id": "exceptions-churn",
                    "version": 1,
                    "inputs": {
                        "generator": "bijux-dev-atlas governance exceptions validate",
                        "sources": [
                            "configs/governance/exceptions.yaml",
                            "configs/governance/exceptions-archive.yaml"
                        ]
                    },
                    "status": "ok",
                    "summary": {
                        "active": rows.len(),
                        "archived": archive.archived_exceptions.len()
                    },
                    "rows": churn_rows,
                    "contracts": {
                        "GOV-EXC-008": true
                    },
                    "errors": []
                });
                validate_named_report(&root, "exceptions-churn.schema.json", &churn_report)?;
                write_pretty_json(&churn_path, &churn_report)?;
                let release_manifest_path = root.join("release/evidence/manifest.json");
                let rel_exc_001 = if release_manifest_path.exists() {
                    let manifest: serde_json::Value = serde_json::from_str(
                        &fs::read_to_string(&release_manifest_path).map_err(|err| {
                            format!("read {} failed: {err}", release_manifest_path.display())
                        })?,
                    )
                    .map_err(|err| {
                        format!("parse {} failed: {err}", release_manifest_path.display())
                    })?;
                    manifest
                        .get("governance_assets")
                        .and_then(|value| value.get("exceptions_registry"))
                        .and_then(|value| value.get("path"))
                        .and_then(serde_json::Value::as_str)
                        == Some("configs/governance/exceptions.yaml")
                        && manifest
                            .get("governance_assets")
                            .and_then(|value| value.get("exceptions_summary"))
                            .and_then(|value| value.get("path"))
                            .and_then(serde_json::Value::as_str)
                            == Some("artifacts/governance/exceptions-summary.json")
                } else {
                    false
                };
                let summary = serde_json::json!({
                    "report_id": "exceptions-summary",
                    "version": 1,
                    "inputs": {
                        "generator": "bijux-dev-atlas governance exceptions validate",
                        "sources": [
                            "configs/governance/exceptions.yaml",
                            "ops/inventory/contracts.json",
                            "ops/inventory/registry.toml"
                        ]
                    },
                    "status": if errors.is_empty() { "ok" } else { "failed" },
                    "summary": {
                        "total": rows.len(),
                        "active": active_rows,
                        "errors": errors.len(),
                        "no_exception_zones": registry.policy.no_exception_zones.len(),
                        "archived": archive.archived_exceptions.len()
                    },
                    "rows": rows,
                    "contracts": {
                        "GOV-EXC-001": registry.schema_version == 1 && archive.schema_version == 1,
                        "GOV-EXC-007": true,
                        "GOV-EXC-008": true,
                        "GOV-EXC-002": !errors.iter().any(|err| err.contains("expired on")),
                        "GOV-EXC-003": !errors.iter().any(|err| err.contains("unknown contract id") || err.contains("unknown check id")),
                        "GOV-EXC-004": !errors.iter().any(|err| err.contains("missing mitigation")),
                        "GOV-EXC-005": !errors.iter().any(|err| err.contains("missing owner") || err.contains("tracking link domain")),
                        "GOV-EXC-006": !errors.iter().any(|err| err.contains("no-exception zone")),
                        "GOV-EXC-009": !errors.iter().any(|err| err.contains("missing verification_plan")),
                        "GOV-EXC-010": !errors.iter().any(|err| err.contains("content_sha256 does not match")),
                        "REL-EXC-001": rel_exc_001
                    },
                    "errors": errors
                });
                validate_named_report(&root, "exceptions-summary.schema.json", &summary)?;
                write_pretty_json(&summary_path, &summary)?;
                write_text(
                    &table_path,
                    &render_exceptions_table(
                        summary["rows"].as_array().map(Vec::as_slice).unwrap_or(&[]),
                    ),
                )?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "governance_exceptions_validate",
                    "status": summary["status"].clone(),
                    "registry_path": registry_path.strip_prefix(&root).unwrap_or(&registry_path).display().to_string(),
                    "archive_path": archive_path.strip_prefix(&root).unwrap_or(&archive_path).display().to_string(),
                    "summary_path": summary_path.strip_prefix(&root).unwrap_or(&summary_path).display().to_string(),
                    "table_path": table_path.strip_prefix(&root).unwrap_or(&table_path).display().to_string(),
                    "warning_path": warning_path.strip_prefix(&root).unwrap_or(&warning_path).display().to_string(),
                    "churn_path": churn_path.strip_prefix(&root).unwrap_or(&churn_path).display().to_string(),
                    "contracts": summary["contracts"].clone(),
                    "errors": summary["errors"].clone()
                });
                let rendered = emit_payload(format, out, &payload)?;
                let exit_code = if summary["status"] == "ok" { 0 } else { 1 };
                Ok((rendered, exit_code))
            }
        },
    }
}
