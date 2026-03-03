// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    GovernanceBreakingCommand, GovernanceCommand, GovernanceDeprecationsCommand,
    GovernanceExceptionsCommand,
};
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
use std::collections::{BTreeMap, BTreeSet};
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

#[derive(Debug, Deserialize)]
struct CompatibilityPolicyRegistry {
    schema_version: u64,
    compatibility_rules: BTreeMap<String, CompatibilityRuleSet>,
    deprecation_window_days: BTreeMap<String, i64>,
}

#[derive(Debug, Deserialize)]
struct CompatibilityRuleSet {
    breaking_changes: Vec<String>,
    rename_requirements: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DeprecationsRegistry {
    schema_version: u64,
    deprecations: Vec<DeprecationEntry>,
}

#[derive(Debug, Deserialize)]
struct DeprecationEntry {
    id: String,
    surface: String,
    old_name: String,
    new_name: String,
    introduced: String,
    removal_target: String,
    redirect_required: bool,
    comms_required: bool,
}

#[derive(Debug, Deserialize)]
struct ChartMetadata {
    version: String,
}

fn exceptions_registry_path(root: &Path) -> PathBuf {
    root.join("configs/governance/exceptions.yaml")
}

fn compatibility_policy_path(root: &Path) -> PathBuf {
    root.join("configs/governance/compatibility.yaml")
}

fn compatibility_policy_schema_path(root: &Path) -> PathBuf {
    root.join("configs/contracts/governance/compatibility.schema.json")
}

fn deprecations_registry_path(root: &Path) -> PathBuf {
    root.join("configs/governance/deprecations.yaml")
}

fn deprecations_registry_schema_path(root: &Path) -> PathBuf {
    root.join("configs/contracts/governance/deprecations.schema.json")
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

fn deprecations_summary_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/deprecations-summary.json")
}

fn compat_warnings_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/compat-warnings.json")
}

fn breaking_changes_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/breaking-changes.json")
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

fn read_json_value(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn load_compatibility_policy(root: &Path) -> Result<CompatibilityPolicyRegistry, String> {
    let path = compatibility_policy_path(root);
    serde_yaml::from_str(
        &fs::read_to_string(&path).map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn load_deprecations_registry(root: &Path) -> Result<DeprecationsRegistry, String> {
    let path = deprecations_registry_path(root);
    serde_yaml::from_str(
        &fs::read_to_string(&path).map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn yaml_paths(value: &serde_yaml::Value, prefix: Option<&str>, out: &mut BTreeSet<String>) {
    if let serde_yaml::Value::Mapping(map) = value {
        for (key, child) in map {
            let Some(key_text) = key.as_str() else {
                continue;
            };
            let path = match prefix {
                Some(prefix) => format!("{prefix}.{key_text}"),
                None => key_text.to_string(),
            };
            out.insert(path.clone());
            yaml_paths(child, Some(&path), out);
        }
    }
}

fn schema_supports_path(schema: &serde_json::Value, dotted_path: &str) -> bool {
    let mut current = schema;
    for segment in dotted_path.split('.') {
        current = match current.get("properties").and_then(|value| value.get(segment)) {
            Some(next) => next,
            None => return false,
        };
    }
    true
}

fn env_allowlist_contains(schema: &serde_json::Value, key: &str) -> bool {
    schema
        .get("properties")
        .and_then(|value| value.get(key))
        .is_some()
}

fn compat_warning_rows(
    root: &Path,
    registry: &DeprecationsRegistry,
) -> Result<Vec<serde_json::Value>, String> {
    let mut rows = Vec::new();
    let mut files = vec![root.join("ops/k8s/charts/bijux-atlas/values.yaml")];
    for entry in fs::read_dir(root.join("ops/k8s/values"))
        .map_err(|err| format!("read ops/k8s/values failed: {err}"))?
    {
        let entry = entry.map_err(|err| format!("read ops/k8s/values entry failed: {err}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("yaml") {
            files.push(path);
        }
    }
    files.sort();

    for path in files {
        let value: serde_yaml::Value = serde_yaml::from_str(
            &fs::read_to_string(&path).map_err(|err| format!("read {} failed: {err}", path.display()))?,
        )
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
        let mut present_paths = BTreeSet::new();
        yaml_paths(&value, None, &mut present_paths);
        for entry in &registry.deprecations {
            if matches!(entry.surface.as_str(), "chart-value" | "profile-key")
                && present_paths.contains(&entry.old_name)
            {
                rows.push(serde_json::json!({
                    "deprecation_id": entry.id,
                    "surface": entry.surface,
                    "file": path.strip_prefix(root).unwrap_or(&path).display().to_string(),
                    "deprecated_key": entry.old_name,
                    "replacement_key": entry.new_name,
                    "removal_target": entry.removal_target
                }));
            }
        }
    }

    rows.sort_by(|left, right| {
        let left_key = (
            left["file"].as_str().unwrap_or_default(),
            left["deprecated_key"].as_str().unwrap_or_default(),
        );
        let right_key = (
            right["file"].as_str().unwrap_or_default(),
            right["deprecated_key"].as_str().unwrap_or_default(),
        );
        left_key.cmp(&right_key)
    });
    Ok(rows)
}

fn read_chart_metadata(root: &Path) -> Result<ChartMetadata, String> {
    let path = root.join("ops/k8s/charts/bijux-atlas/Chart.yaml");
    serde_yaml::from_str(
        &fs::read_to_string(&path).map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn semver_major(version: &str) -> Option<u64> {
    version.split('.').next()?.parse().ok()
}

fn release_breaking_notes_meta_path(root: &Path) -> PathBuf {
    root.join("release/notes/breaking.md")
}

fn release_breaking_notes_schema_path(root: &Path) -> PathBuf {
    root.join("release/notes/breaking.schema.json")
}

fn parse_front_matter(text: &str) -> Result<serde_yaml::Value, String> {
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return Err("markdown file must start with front matter".to_string());
    }
    let mut front_matter = String::new();
    for line in lines {
        if line == "---" {
            return serde_yaml::from_str(&front_matter)
                .map_err(|err| format!("front matter parse failed: {err}"));
        }
        front_matter.push_str(line);
        front_matter.push('\n');
    }
    Err("markdown front matter missing closing delimiter".to_string())
}

fn load_breaking_notes_meta(root: &Path) -> Result<serde_json::Value, String> {
    let path = release_breaking_notes_meta_path(root);
    let schema_path = release_breaking_notes_schema_path(root);
    let text =
        fs::read_to_string(&path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let front_matter = parse_front_matter(&text)?;
    let value =
        serde_json::to_value(front_matter).map_err(|err| format!("front matter encode failed: {err}"))?;
    let schema = read_json_value(&schema_path)?;
    if schema
        .get("properties")
        .and_then(|value| value.get("schema_version"))
        .and_then(|value| value.get("const"))
        .and_then(serde_json::Value::as_u64)
        != value.get("schema_version").and_then(serde_json::Value::as_u64)
    {
        return Err(format!(
            "{} schema_version does not match {}",
            path.display(),
            schema_path.display()
        ));
    }
    if value.get("entries").and_then(serde_json::Value::as_array).is_none() {
        return Err(format!("{} front matter must declare entries array", path.display()));
    }
    Ok(value)
}

fn active_exception_for_contract(root: &Path, contract_id: &str) -> Result<bool, String> {
    let registry_path = exceptions_registry_path(root);
    let registry_text = fs::read_to_string(&registry_path)
        .map_err(|err| format!("read {} failed: {err}", registry_path.display()))?;
    let registry: ExceptionsRegistry = serde_yaml::from_str(&registry_text)
        .map_err(|err| format!("parse {} failed: {err}", registry_path.display()))?;
    let today = current_utc_day()?;
    Ok(registry.exceptions.iter().any(|item| {
        item.scope.kind == "contract"
            && item.scope.id == contract_id
            && date_days(&item.expires_at)
                .map(|expires| expires >= today)
                .unwrap_or(false)
    }))
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
        GovernanceCommand::Deprecations { command } => match command {
            GovernanceDeprecationsCommand::Validate {
                repo_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let compatibility_schema_path = compatibility_policy_schema_path(&root);
                let compatibility_schema = read_json_value(&compatibility_schema_path)?;
                let deprecations_schema_path = deprecations_registry_schema_path(&root);
                let deprecations_schema = read_json_value(&deprecations_schema_path)?;
                let compatibility = load_compatibility_policy(&root)?;
                let deprecations = load_deprecations_registry(&root)?;
                let values_schema = read_json_value(
                    &root.join("ops/k8s/charts/bijux-atlas/values.schema.json"),
                )?;
                let env_schema =
                    read_json_value(&root.join("configs/contracts/env.schema.json"))?;
                let redirects = read_json_value(&root.join("docs/redirects.json"))?;
                let redirects_map = redirects
                    .as_object()
                    .ok_or_else(|| "docs/redirects.json must be a JSON object".to_string())?;
                let summary_path = deprecations_summary_path(&root);
                let compat_path = compat_warnings_path(&root);
                let today = current_utc_day()?;
                let compat_warning_rows = compat_warning_rows(&root, &deprecations)?;
                let compat_warnings_report = serde_json::json!({
                    "report_id": "compat-warnings",
                    "version": 1,
                    "inputs": {
                        "generator": "bijux-dev-atlas governance deprecations validate",
                        "sources": [
                            "configs/governance/deprecations.yaml",
                            "ops/k8s/charts/bijux-atlas/values.yaml",
                            "ops/k8s/values"
                        ]
                    },
                    "status": "ok",
                    "summary": {
                        "total": compat_warning_rows.len()
                    },
                    "rows": compat_warning_rows,
                    "errors": []
                });
                validate_named_report(&root, "compat-warnings.schema.json", &compat_warnings_report)?;
                write_pretty_json(&compat_path, &compat_warnings_report)?;

                let mut errors = Vec::new();
                let mut rows = Vec::new();
                let mut seen_ids = BTreeSet::new();
                let required_rule_sets = [
                    "env_keys",
                    "chart_values",
                    "profile_keys",
                    "report_schemas",
                    "docs_urls",
                ];
                for rule_set in required_rule_sets {
                    if !compatibility.compatibility_rules.contains_key(rule_set) {
                        errors.push(format!("compatibility policy missing rule set `{rule_set}`"));
                    }
                    if !compatibility.deprecation_window_days.contains_key(rule_set) {
                        errors.push(format!(
                            "compatibility policy missing deprecation window for `{rule_set}`"
                        ));
                    }
                }
                for (name, rule_set) in &compatibility.compatibility_rules {
                    if rule_set.breaking_changes.is_empty() {
                        errors.push(format!("compatibility rule set `{name}` missing breaking_changes"));
                    }
                    if rule_set.rename_requirements.is_empty() {
                        errors.push(format!(
                            "compatibility rule set `{name}` missing rename_requirements"
                        ));
                    }
                }

                for entry in &deprecations.deprecations {
                    if !seen_ids.insert(entry.id.clone()) {
                        errors.push(format!("duplicate deprecation id `{}`", entry.id));
                    }
                    if !is_iso_date(&entry.introduced) {
                        errors.push(format!(
                            "{} has invalid introduced `{}`",
                            entry.id, entry.introduced
                        ));
                    }
                    if !is_iso_date(&entry.removal_target) {
                        errors.push(format!(
                            "{} has invalid removal_target `{}`",
                            entry.id, entry.removal_target
                        ));
                    }
                    let introduced_days = date_days(&entry.introduced)?;
                    let removal_days = date_days(&entry.removal_target)?;
                    if removal_days < introduced_days {
                        errors.push(format!(
                            "{} removal_target precedes introduced date",
                            entry.id
                        ));
                    }
                    let days_until_removal = removal_days - today;
                    if days_until_removal < 0 {
                        errors.push(format!(
                            "{} is past removal_target {}",
                            entry.id, entry.removal_target
                        ));
                    }

                    let mut checks = BTreeMap::new();
                    match entry.surface.as_str() {
                        "env-key" => {
                            let old_supported = env_allowlist_contains(&env_schema, &entry.old_name);
                            let new_supported = env_allowlist_contains(&env_schema, &entry.new_name);
                            let docs_updated = !entry.new_name.is_empty();
                            checks.insert("allowlist_support".to_string(), old_supported && new_supported);
                            checks.insert("docs_updated".to_string(), docs_updated);
                            if !old_supported || !new_supported {
                                errors.push(format!(
                                    "{} env rename requires old and new keys in env schema",
                                    entry.id
                                ));
                            }
                        }
                        "chart-value" => {
                            let old_supported = schema_supports_path(&values_schema, &entry.old_name);
                            let new_supported = schema_supports_path(&values_schema, &entry.new_name);
                            checks.insert(
                                "schema_overlap".to_string(),
                                old_supported && new_supported,
                            );
                            if !old_supported || !new_supported {
                                errors.push(format!(
                                    "{} chart rename requires schema support for old and new keys",
                                    entry.id
                                ));
                            }
                        }
                        "profile-key" => {
                            let warning_exists = compat_warnings_report["rows"]
                                .as_array()
                                .is_some_and(|rows| {
                                    rows.iter().any(|row| {
                                        row.get("deprecation_id")
                                            .and_then(serde_json::Value::as_str)
                                            == Some(entry.id.as_str())
                                    })
                                });
                            checks.insert("warning_report".to_string(), warning_exists);
                        }
                        "docs-url" => {
                            let redirect_ok = redirects_map
                                .get(&entry.old_name)
                                .and_then(serde_json::Value::as_str)
                                == Some(entry.new_name.as_str());
                            checks.insert("redirect_present".to_string(), redirect_ok);
                            if entry.redirect_required && !redirect_ok {
                                errors.push(format!(
                                    "{} docs move requires redirect {} -> {}",
                                    entry.id, entry.old_name, entry.new_name
                                ));
                            }
                        }
                        "report-schema" => {}
                        other => {
                            errors.push(format!("{} has unsupported surface `{}`", entry.id, other));
                        }
                    }

                    rows.push(serde_json::json!({
                        "id": entry.id,
                        "surface": entry.surface,
                        "old_name": entry.old_name,
                        "new_name": entry.new_name,
                        "introduced": entry.introduced,
                        "removal_target": entry.removal_target,
                        "days_until_removal": days_until_removal,
                        "redirect_required": entry.redirect_required,
                        "comms_required": entry.comms_required,
                        "checks": checks
                    }));
                }

                rows.sort_by(|left, right| {
                    left["id"]
                        .as_str()
                        .unwrap_or_default()
                        .cmp(right["id"].as_str().unwrap_or_default())
                });

                let summary = serde_json::json!({
                    "report_id": "deprecations-summary",
                    "version": 1,
                    "inputs": {
                        "generator": "bijux-dev-atlas governance deprecations validate",
                        "sources": [
                            "configs/governance/compatibility.yaml",
                            "configs/governance/deprecations.yaml",
                            "configs/contracts/env.schema.json",
                            "ops/k8s/charts/bijux-atlas/values.schema.json",
                            "docs/redirects.json"
                        ]
                    },
                    "status": if errors.is_empty() { "ok" } else { "failed" },
                    "summary": {
                        "total": rows.len(),
                        "compat_warning_count": compat_warnings_report["summary"]["total"].clone(),
                        "errors": errors.len()
                    },
                    "rows": rows,
                    "contracts": {
                        "GOV-COMP-001": compatibility.schema_version == 1
                            && compatibility_schema
                                .get("properties")
                                .and_then(|value| value.get("schema_version"))
                                .and_then(|value| value.get("const"))
                                .and_then(serde_json::Value::as_u64)
                                == Some(1),
                        "GOV-DEP-001": deprecations.schema_version == 1
                            && deprecations_schema
                                .get("properties")
                                .and_then(|value| value.get("schema_version"))
                                .and_then(|value| value.get("const"))
                                .and_then(serde_json::Value::as_u64)
                                == Some(1),
                        "GOV-DEP-002": !errors.iter().any(|error| error.contains("past removal_target"))
                    },
                    "errors": errors
                });
                validate_named_report(&root, "deprecations-summary.schema.json", &summary)?;
                write_pretty_json(&summary_path, &summary)?;

                let payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "governance_deprecations_validate",
                    "status": summary["status"].clone(),
                    "compatibility_policy_path": compatibility_policy_path(&root)
                        .strip_prefix(&root)
                        .unwrap_or(&compatibility_policy_path(&root))
                        .display()
                        .to_string(),
                    "deprecations_registry_path": deprecations_registry_path(&root)
                        .strip_prefix(&root)
                        .unwrap_or(&deprecations_registry_path(&root))
                        .display()
                        .to_string(),
                    "summary_path": summary_path.strip_prefix(&root).unwrap_or(&summary_path).display().to_string(),
                    "compat_warnings_path": compat_path.strip_prefix(&root).unwrap_or(&compat_path).display().to_string(),
                    "contracts": summary["contracts"].clone(),
                    "errors": summary["errors"].clone()
                });
                let rendered = emit_payload(format, out, &payload)?;
                let exit_code = if summary["status"] == "ok" { 0 } else { 1 };
                Ok((rendered, exit_code))
            }
        },
        GovernanceCommand::Breaking { command } => match command {
            GovernanceBreakingCommand::Validate {
                repo_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let deprecations = load_deprecations_registry(&root)?;
                let chart = read_chart_metadata(&root)?;
                let chart_major = semver_major(&chart.version).unwrap_or(0);
                let compat_warnings = read_json_value(&compat_warnings_path(&root))?;
                let redirects = read_json_value(&root.join("docs/redirects.json"))?;
                let redirects_map = redirects
                    .as_object()
                    .ok_or_else(|| "docs/redirects.json must be a JSON object".to_string())?;
                let notes = load_breaking_notes_meta(&root)?;
                let breaking_path = breaking_changes_path(&root);
                let today = current_utc_day()?;
                let mut errors = Vec::new();
                let mut rows = Vec::new();

                let report_schema_deprecations = deprecations
                    .deprecations
                    .iter()
                    .filter(|entry| entry.surface == "report-schema")
                    .collect::<Vec<_>>();
                let mut gov_rep_001 = true;
                for entry in &report_schema_deprecations {
                    let migration_path = root.join(format!(
                        "docs/reference/reports/migrations/{}.md",
                        entry.id
                    ));
                    if !migration_path.exists() {
                        gov_rep_001 = false;
                        errors.push(format!(
                            "{} requires migration note {}",
                            entry.id,
                            migration_path.display()
                        ));
                    }
                }

                let docs_url_rows = deprecations
                    .deprecations
                    .iter()
                    .filter(|entry| entry.surface == "docs-url")
                    .map(|entry| {
                        let redirect_target = redirects_map
                            .get(&entry.old_name)
                            .and_then(serde_json::Value::as_str);
                        let target_exists = root.join(&entry.new_name).exists();
                        let redirect_ok = redirect_target == Some(entry.new_name.as_str());
                        let redirect_tested = redirect_ok && target_exists;
                        if entry.redirect_required && !redirect_tested {
                            errors.push(format!(
                                "{} docs move missing tested redirect {} -> {}",
                                entry.id, entry.old_name, entry.new_name
                            ));
                        }
                        serde_json::json!({
                            "id": entry.id,
                            "category": "docs-nav",
                            "surface": entry.surface,
                            "change": format!("{} -> {}", entry.old_name, entry.new_name),
                            "breaking": entry.redirect_required && !redirect_tested,
                            "redirect_tested": redirect_tested
                        })
                    })
                    .collect::<Vec<_>>();

                let docs_comp_001 = docs_url_rows
                    .iter()
                    .all(|row| !row["breaking"].as_bool().unwrap_or(false));

                let prod_warning_rows = compat_warnings
                    .get("rows")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|row| {
                        row.get("file")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|file| {
                                file.contains("prod")
                                    || file.contains("profile-baseline.yaml")
                            })
                    })
                    .collect::<Vec<_>>();
                let ci_warning_rows = compat_warnings
                    .get("rows")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|row| {
                        row.get("file")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|file| file.ends_with("/ci.yaml") || file.ends_with("ci.yaml"))
                    })
                    .collect::<Vec<_>>();
                let ops_comp_exception = active_exception_for_contract(&root, "OPS-COMP-001")?;
                let ops_comp_001 = prod_warning_rows.is_empty() || ops_comp_exception;
                if !ops_comp_001 {
                    errors.push(
                        "prod profiles still use deprecated compatibility keys without exception"
                            .to_string(),
                    );
                }
                let ops_comp_002 = true;

                let env_breaks = deprecations
                    .deprecations
                    .iter()
                    .filter(|entry| entry.surface == "env-key")
                    .filter_map(|entry| {
                        let removal_days = date_days(&entry.removal_target).ok()?;
                        Some(serde_json::json!({
                            "id": entry.id,
                            "category": "runtime-env",
                            "surface": entry.surface,
                            "change": format!("{} -> {}", entry.old_name, entry.new_name),
                            "breaking": removal_days <= today
                        }))
                    })
                    .collect::<Vec<_>>();
                let chart_breaks = deprecations
                    .deprecations
                    .iter()
                    .filter(|entry| entry.surface == "chart-value")
                    .filter_map(|entry| {
                        let removal_days = date_days(&entry.removal_target).ok()?;
                        Some(serde_json::json!({
                            "id": entry.id,
                            "category": "chart",
                            "surface": entry.surface,
                            "change": format!("{} -> {}", entry.old_name, entry.new_name),
                            "breaking": removal_days <= today
                        }))
                    })
                    .collect::<Vec<_>>();
                let report_breaks = report_schema_deprecations
                    .iter()
                    .map(|entry| {
                        serde_json::json!({
                            "id": entry.id,
                            "category": "report-schema",
                            "surface": entry.surface,
                            "change": format!("{} -> {}", entry.old_name, entry.new_name),
                            "breaking": true
                        })
                    })
                    .collect::<Vec<_>>();

                rows.extend(chart_breaks);
                rows.extend(env_breaks);
                rows.extend(report_breaks);
                rows.extend(docs_url_rows);
                rows.retain(|row| row["breaking"].as_bool().unwrap_or(false));
                rows.sort_by(|left, right| {
                    left["id"]
                        .as_str()
                        .unwrap_or_default()
                        .cmp(right["id"].as_str().unwrap_or_default())
                });

                let chart_breaking = rows.iter().any(|row| row["category"] == "chart");
                let notes_entries = notes
                    .get("entries")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let notes_cover_breaks = rows.is_empty() || !notes_entries.is_empty();
                if !notes_cover_breaks {
                    errors.push(
                        "breaking changes exist but release/notes/breaking.md has no entries"
                            .to_string(),
                    );
                }
                let chart_version_ok = !chart_breaking || chart_major >= 1;
                if !chart_version_ok {
                    errors.push(format!(
                        "chart version {} does not satisfy major bump policy for breaking chart changes",
                        chart.version
                    ));
                }

                let summary = serde_json::json!({
                    "report_id": "breaking-changes",
                    "version": 1,
                    "inputs": {
                        "generator": "bijux-dev-atlas governance breaking validate",
                        "sources": [
                            "configs/governance/deprecations.yaml",
                            "artifacts/governance/compat-warnings.json",
                            "docs/redirects.json",
                            "ops/k8s/charts/bijux-atlas/Chart.yaml",
                            "release/notes/breaking.md"
                        ]
                    },
                    "status": if errors.is_empty() { "ok" } else { "failed" },
                    "summary": {
                        "total": rows.len(),
                        "chart_breaking": rows.iter().filter(|row| row["category"] == "chart").count(),
                        "runtime_env_breaking": rows.iter().filter(|row| row["category"] == "runtime-env").count(),
                        "docs_breaking": rows.iter().filter(|row| row["category"] == "docs-nav").count(),
                        "report_schema_breaking": rows.iter().filter(|row| row["category"] == "report-schema").count(),
                        "prod_compat_warnings": prod_warning_rows.len(),
                        "ci_compat_warnings": ci_warning_rows.len()
                    },
                    "rows": rows,
                    "contracts": {
                        "OPS-COMP-001": ops_comp_001,
                        "OPS-COMP-002": ops_comp_002,
                        "GOV-REP-001": gov_rep_001,
                        "DOCS-COMP-001": docs_comp_001,
                        "GOV-BREAK-001": true,
                        "GOV-BREAK-002": notes_cover_breaks && chart_version_ok
                    },
                    "errors": errors
                });
                validate_named_report(&root, "breaking-changes.schema.json", &summary)?;
                write_pretty_json(&breaking_path, &summary)?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "governance_breaking_validate",
                    "status": summary["status"].clone(),
                    "report_path": breaking_path.strip_prefix(&root).unwrap_or(&breaking_path).display().to_string(),
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
