// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    FormatArg, GovernanceAdrCommand, GovernanceBreakingCommand, GovernanceCommand,
    GovernanceDeprecationsCommand, GovernanceExceptionsCommand, RegistryCommand,
    RegistryMissingArg,
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
use bijux_dev_atlas::reference::governance_enforcement;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
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

#[derive(Debug, Deserialize)]
struct GovernanceSuiteRegistry {
    schema_version: u64,
    suite_id: String,
    purpose: String,
    owner: String,
    stability: String,
    entries: Vec<GovernanceSuiteEntry>,
}

#[derive(Debug, Deserialize)]
struct GovernanceSuiteEntry {
    id: String,
    kind: String,
    mode: String,
    owner: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ChecksRegistryCatalog {
    schema_version: u64,
    registry_id: String,
    checks: Vec<ChecksRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct ChecksRegistryEntry {
    check_id: String,
    summary: String,
    owner: String,
    mode: String,
    group: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    commands: Vec<String>,
    report_ids: Vec<String>,
    reports: Vec<String>,
    depends_on: Option<Vec<String>>,
    replaces: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    since_version: Option<String>,
    retries: Option<u64>,
    overlaps_with: Option<Vec<String>>,
    requires_tools: Option<Vec<String>>,
    missing_tools_policy: Option<String>,
    suite_membership: Vec<String>,
    severity: String,
    stage: String,
    runtime_cost: String,
    determinism: String,
    cpu_hint: Option<String>,
    mem_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContractsRegistryCatalog {
    schema_version: u64,
    registry_id: String,
    contracts: Vec<ContractsRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct ContractsRegistryEntry {
    contract_id: String,
    summary: String,
    owner: String,
    mode: String,
    group: String,
    runner: String,
    reports: Vec<String>,
    suite_membership: Vec<String>,
    tags: Option<Vec<String>>,
    retries: Option<u64>,
    overlaps_with: Option<Vec<String>>,
    requires_tools: Option<Vec<String>>,
    missing_tools_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GovernanceTagsTaxonomy {
    schema_version: u64,
    taxonomy_id: String,
    owner: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GovernanceSuitesIndex {
    schema_version: u64,
    index_id: String,
    owner: String,
    suites: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GroupRegistry {
    schema_version: u64,
    group_set_id: String,
    owner: String,
    groups: Vec<GroupEntry>,
}

#[derive(Debug, Deserialize)]
struct GroupEntry {
    id: String,
    summary: String,
}

#[derive(Debug, Deserialize)]
struct RegistryCompletenessPolicy {
    schema_version: u64,
    policy_id: String,
    owner: String,
    required_field_coverage_percent: u64,
    required_rules: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GovernanceRegistry {
    schema_version: u64,
    governance_version: u64,
    compatibility_policy: String,
    contract_evolution_policy: ContractEvolutionPolicy,
    components: GovernanceComponents,
    policy_authority_mappings: PolicyAuthorityMappings,
    policies: Vec<GovernancePolicy>,
}

#[derive(Debug, Deserialize)]
struct ContractEvolutionPolicy {
    authority: String,
    compatibility_mode: String,
}

#[derive(Debug, Deserialize)]
struct GovernanceComponents {
    policies_registry: String,
    contracts_registry: String,
    checks_registry: String,
    exceptions_registry: String,
}

#[derive(Debug, Deserialize)]
struct PolicyAuthorityMappings {
    checks: String,
    contracts: String,
}

#[derive(Debug, Deserialize)]
struct GovernancePolicy {
    id: String,
    authority: String,
    enforcement: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GovernanceVersionHistory {
    schema_version: u64,
    current: u64,
    versions: Vec<u64>,
}

#[derive(Debug, Deserialize)]
struct ContractPolicyAuthorityMap {
    schema_version: u64,
    owner: String,
    mappings: Vec<ContractPolicyAuthorityMapping>,
}

#[derive(Debug, Deserialize)]
struct ContractPolicyAuthorityMapping {
    contract_id: String,
    policy_id: String,
    authority: String,
}

#[derive(Debug, Deserialize)]
struct DefaultJobsPolicy {
    schema_version: u64,
    policy_id: String,
    owner: String,
    suites: Vec<DefaultJobsEntry>,
}

#[derive(Debug, Deserialize)]
struct DefaultJobsEntry {
    suite_id: String,
    auto_jobs: u64,
    low_core_cap: u64,
}

#[derive(Debug, Deserialize)]
struct SuiteBaselinePolicy {
    schema_version: u64,
    baseline_id: String,
    owner: String,
    suites: Vec<SuiteBaselineEntry>,
}

#[derive(Debug, Deserialize)]
struct SuiteBaselineEntry {
    suite_id: String,
    expected_total: u64,
    minimum_group_counts: BTreeMap<String, u64>,
}

#[derive(Debug, Deserialize)]
struct CheckReportSchemaIndex {
    schema_version: u64,
    index_id: String,
    schemas: Vec<CheckReportSchemaEntry>,
}

#[derive(Debug, Deserialize)]
struct CheckReportSchemaEntry {
    report_id: String,
    schema_path: String,
}

fn exceptions_registry_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/exceptions.yaml")
}

fn checks_suite_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/suites/checks.suite.json")
}

fn contracts_suite_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/suites/contracts.suite.json")
}

fn tests_suite_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/suites/tests.suite.json")
}

fn checks_schema_index_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/report-checks/schema-index.json")
}

fn suite_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/suite.schema.json")
}

fn governance_registry_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/governance.json")
}

fn governance_registry_schema_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/schemas/governance.schema.json")
}

fn governance_version_history_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/governance-version-history.json")
}

fn contract_policy_authority_map_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/contract-policy-authority-map.json")
}

fn checks_registry_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/checks.registry.json")
}

fn checks_registry_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/checks-registry.schema.json")
}

fn contracts_registry_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/contracts.registry.json")
}

fn contracts_registry_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/contracts-registry.schema.json")
}

fn checks_inventory_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/checks-inventory.json")
}

fn registry_missing_fields_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/registry-missing-fields.json")
}

fn registry_status_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/registry-status.json")
}

fn registry_work_remaining_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/registry-work-remaining.json")
}

fn registry_status_markdown_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/registry-status.md")
}

fn suites_index_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/suites/suites.index.json")
}

fn suites_index_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/suites-index.schema.json")
}

fn tags_taxonomy_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/tags.json")
}

fn tags_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/tags.schema.json")
}

fn check_groups_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/check-groups.json")
}

fn check_groups_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/check-groups.schema.json")
}

fn contract_groups_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/contract-groups.json")
}

fn contract_groups_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/contract-groups.schema.json")
}

fn registry_completeness_policy_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/registry-completeness-policy.json")
}

fn registry_completeness_policy_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/registry-completeness-policy.schema.json")
}

fn default_jobs_policy_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/suites/default-jobs.json")
}

fn default_jobs_policy_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/default-jobs.schema.json")
}

fn suite_baseline_policy_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/suites/baseline.json")
}

fn suite_baseline_policy_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/suite-baseline.schema.json")
}

fn compatibility_policy_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/compatibility.yaml")
}

fn compatibility_policy_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/compatibility.schema.json")
}

fn deprecations_registry_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/deprecations.yaml")
}

fn deprecations_registry_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/deprecations.schema.json")
}

fn exceptions_registry_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/exceptions.schema.json")
}

fn exceptions_archive_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/governance/exceptions-archive.yaml")
}

fn exceptions_archive_schema_path(root: &Path) -> PathBuf {
    root.join("configs/schemas/contracts/governance/exceptions-archive.schema.json")
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

fn governance_doctor_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/governance-doctor.json")
}

fn governance_adr_index_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/adr-index.json")
}

fn governance_health_report_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/governance-health-report.json")
}

fn governance_enforcement_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/enforcement-report.json")
}

fn governance_enforcement_coverage_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/enforcement-coverage.json")
}

fn governance_enforcement_metrics_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/enforcement-metrics.json")
}

fn governance_enforcement_coverage_payload(
    registry: &governance_enforcement::GovernanceRuleRegistry,
) -> serde_json::Value {
    let mut by_severity: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_rule_type: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_classification: BTreeMap<String, usize> = BTreeMap::new();
    for rule in &registry.rules {
        let sev = serde_json::to_value(&rule.severity)
            .ok()
            .and_then(|v| v.as_str().map(ToString::to_string))
            .unwrap_or_else(|| "unknown".to_string());
        *by_severity.entry(sev).or_insert(0) += 1;
        let rule_type = serde_json::to_value(&rule.rule_type)
            .ok()
            .and_then(|v| v.as_str().map(ToString::to_string))
            .unwrap_or_else(|| "unknown".to_string());
        *by_rule_type.entry(rule_type).or_insert(0) += 1;
        let classification = serde_json::to_value(&rule.classification)
            .ok()
            .and_then(|v| v.as_str().map(ToString::to_string))
            .unwrap_or_else(|| "unknown".to_string());
        *by_classification.entry(classification).or_insert(0) += 1;
    }
    serde_json::json!({
        "schema_version": 1,
        "kind": "governance_enforcement_coverage",
        "rule_count": registry.rules.len(),
        "coverage": {
            "by_severity": by_severity,
            "by_rule_type": by_rule_type,
            "by_classification": by_classification,
        }
    })
}

fn institutional_delta_inputs_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/institutional-delta-inputs.json")
}

fn institutional_delta_markdown_path(root: &Path) -> PathBuf {
    root.join("artifacts/governance/institutional-delta.md")
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
    rest.split('/').next()?.split(':').next()
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

fn registry_status_markdown(payload: &serde_json::Value) -> String {
    let mut out = String::from("# Registry Status\n\n");
    out.push_str(&format!(
        "Status: `{}`\n\n",
        payload["status"].as_str().unwrap_or("unknown")
    ));
    out.push_str("| Kind | Id | Missing |\n|---|---|---|\n");
    for row in payload["rows"].as_array().into_iter().flatten() {
        let missing = row["missing"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            row["kind"].as_str().unwrap_or_default(),
            row["id"].as_str().unwrap_or_default(),
            if missing.is_empty() { "none" } else { &missing }
        ));
    }
    out
}

fn registry_missing_field_name(filter: RegistryMissingArg) -> &'static str {
    match filter {
        RegistryMissingArg::Owner => "owner",
        RegistryMissingArg::Reports => "reports",
        RegistryMissingArg::SuiteMembership => "suite_membership",
        RegistryMissingArg::Command => "command",
        RegistryMissingArg::BrokenCommand => "broken_command",
    }
}

fn registry_required_field_total(kind: &str) -> usize {
    match kind {
        "check" => 5,
        "contract" => 4,
        _ => 0,
    }
}

fn registry_status_payload(root: &Path) -> Result<serde_json::Value, String> {
    let checks_inventory = validate_checks_inventory(root)?;
    let missing_fields = read_json_value(&registry_missing_fields_path(root))?;
    let checks_registry: ChecksRegistryCatalog = read_json_file(&checks_registry_path(root))?;
    let contracts_registry: ContractsRegistryCatalog =
        read_json_file(&contracts_registry_path(root))?;
    let resolvable_commands = known_commands(root)?;

    let mut rows = Vec::<serde_json::Value>::new();
    let missing_map = missing_fields["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            Some((
                format!(
                    "{}:{}",
                    row.get("kind")?.as_str()?,
                    row.get("id")?.as_str()?
                ),
                row.get("missing")?.as_array()?.clone(),
            ))
        })
        .collect::<BTreeMap<_, _>>();

    for check in &checks_registry.checks {
        let mut missing = Vec::<String>::new();
        if check.owner.trim().is_empty() {
            missing.push("owner".to_string());
        }
        if check.reports.is_empty() {
            missing.push("reports".to_string());
        }
        if check.suite_membership.is_empty() {
            missing.push("suite_membership".to_string());
        }
        if check.commands.is_empty() {
            missing.push("command".to_string());
        }
        if check
            .commands
            .iter()
            .any(|command| !resolvable_commands.contains(command))
        {
            missing.push("broken_command".to_string());
        }
        if let Some(extra_missing) = missing_map.get(&format!("check:{}", check.check_id)) {
            for value in extra_missing.iter().filter_map(serde_json::Value::as_str) {
                if !missing.iter().any(|entry| entry == value) {
                    missing.push(value.to_string());
                }
            }
        }
        rows.push(serde_json::json!({
            "kind": "check",
            "id": check.check_id,
            "owner": check.owner,
            "missing": missing,
            "reports": check.reports,
            "suite_membership": check.suite_membership,
            "command_count": check.commands.len(),
        }));
    }
    for contract in &contracts_registry.contracts {
        let mut missing = Vec::<String>::new();
        if contract.owner.trim().is_empty() {
            missing.push("owner".to_string());
        }
        if contract.reports.is_empty() {
            missing.push("reports".to_string());
        }
        if contract.suite_membership.is_empty() {
            missing.push("suite_membership".to_string());
        }
        if contract.runner.trim().is_empty() {
            missing.push("command".to_string());
        } else if !resolvable_commands.contains(&contract.runner) {
            missing.push("broken_command".to_string());
        }
        if let Some(extra_missing) = missing_map.get(&format!("contract:{}", contract.contract_id))
        {
            for value in extra_missing.iter().filter_map(serde_json::Value::as_str) {
                if !missing.iter().any(|entry| entry == value) {
                    missing.push(value.to_string());
                }
            }
        }
        rows.push(serde_json::json!({
            "kind": "contract",
            "id": contract.contract_id,
            "owner": contract.owner,
            "missing": missing,
            "reports": contract.reports,
            "suite_membership": contract.suite_membership,
            "command_count": if contract.runner.trim().is_empty() { 0 } else { 1 },
        }));
    }

    rows.sort_by(|left, right| {
        left["kind"]
            .as_str()
            .unwrap_or_default()
            .cmp(right["kind"].as_str().unwrap_or_default())
            .then_with(|| {
                left["id"]
                    .as_str()
                    .unwrap_or_default()
                    .cmp(right["id"].as_str().unwrap_or_default())
            })
    });

    let fully_specified = rows
        .iter()
        .filter(|row| {
            row["missing"]
                .as_array()
                .is_some_and(|items| items.is_empty())
        })
        .count();
    let missing = rows
        .iter()
        .filter(|row| {
            let field_count = row["missing"].as_array().map_or(0, |items| items.len());
            field_count == registry_required_field_total(row["kind"].as_str().unwrap_or_default())
        })
        .count();
    let partially_specified = rows.len().saturating_sub(fully_specified + missing);
    let work_remaining = rows
        .iter()
        .filter(|row| {
            row["missing"]
                .as_array()
                .is_some_and(|items| !items.is_empty())
        })
        .cloned()
        .collect::<Vec<_>>();

    let payload = serde_json::json!({
        "report_id": "registry-status",
        "version": 1,
        "inputs": {
            "checks_inventory": checks_inventory_path(root).strip_prefix(root).unwrap_or(&checks_inventory_path(root)).display().to_string(),
            "missing_fields_report": registry_missing_fields_path(root).strip_prefix(root).unwrap_or(&registry_missing_fields_path(root)).display().to_string()
        },
        "status": if checks_inventory["status"] == "ok" { "ok" } else { "failed" },
        "summary": {
            "total": rows.len(),
            "fully_specified": fully_specified,
            "partially_specified": partially_specified,
            "missing": missing,
            "work_remaining": work_remaining.len()
        },
        "rows": rows,
        "work_remaining": work_remaining,
        "errors": checks_inventory["errors"].clone()
    });
    Ok(payload)
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
    out.push_str("Read-only generated view from `configs/sources/governance/governance/exceptions.yaml`.\n\n");
    out.push_str(
        "| Id | Scope | Severity | Owner | Expires | Days left |\n|---|---|---|---|---|---|\n",
    );
    for row in rows {
        let id = row
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
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
    let bytes = serde_json::to_vec(value)
        .map_err(|err| format!("encode exception digest failed: {err}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn read_json_value(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn read_yaml_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    serde_yaml::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn governance_version_value(root: &Path) -> Result<u64, String> {
    let registry: GovernanceRegistry = read_json_file(&governance_registry_path(root))?;
    Ok(registry.governance_version)
}

fn validate_governance_registry(root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let schema_path = governance_registry_schema_path(root);
    let schema = read_json_value(&schema_path)?;
    let required = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let registry_value = read_json_value(&governance_registry_path(root))?;
    let Some(registry_obj) = registry_value.as_object() else {
        return Ok(vec!["governance registry must be a JSON object".to_string()]);
    };
    for key in required.iter().filter_map(serde_json::Value::as_str) {
        if !registry_obj.contains_key(key) {
            errors.push(format!("governance registry missing required key `{key}`"));
        }
    }
    let registry: GovernanceRegistry =
        serde_json::from_value(registry_value.clone()).map_err(|err| {
            format!(
                "parse {} failed: {err}",
                governance_registry_path(root).display()
            )
        })?;
    if registry.schema_version != 1 {
        errors.push("governance schema_version must be 1".to_string());
    }
    if registry.governance_version == 0 {
        errors.push("governance version must exist and be greater than zero".to_string());
    }

    let version_history: GovernanceVersionHistory =
        read_json_file(&governance_version_history_path(root))?;
    if version_history.schema_version != 1 {
        errors.push("governance version history schema_version must be 1".to_string());
    }
    if version_history.current != registry.governance_version {
        errors.push(format!(
            "governance version history current `{}` must equal governance version `{}`",
            version_history.current, registry.governance_version
        ));
    }
    if version_history.versions.is_empty() {
        errors.push("governance version history must include at least one version".to_string());
    } else {
        let mut prev = 0_u64;
        for version in &version_history.versions {
            if *version <= prev {
                errors.push("governance versions must be strictly increasing".to_string());
                break;
            }
            prev = *version;
        }
        if version_history.versions.last().copied().unwrap_or_default()
            != registry.governance_version
        {
            errors.push(
                "governance version must be monotonic and end at current version".to_string(),
            );
        }
    }

    let compatibility_path = root.join(&registry.compatibility_policy);
    if !compatibility_path.exists() {
        errors.push(format!(
            "compatibility policy path missing: {}",
            registry.compatibility_policy
        ));
    }
    if registry
        .contract_evolution_policy
        .authority
        .trim()
        .is_empty()
        || registry
            .contract_evolution_policy
            .compatibility_mode
            .trim()
            .is_empty()
    {
        errors.push(
            "contract evolution policy must include authority and compatibility_mode".to_string(),
        );
    }

    for component in [
        &registry.components.policies_registry,
        &registry.components.contracts_registry,
        &registry.components.checks_registry,
        &registry.components.exceptions_registry,
    ] {
        if !root.join(component).exists() {
            errors.push(format!("governance component path missing: {component}"));
        }
    }

    let check_map_path = root.join(&registry.policy_authority_mappings.checks);
    let check_map = read_json_value(&check_map_path)?;
    let check_mappings = check_map
        .get("mappings")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if check_mappings.is_empty() {
        errors.push("checks-to-policy authority mappings must not be empty".to_string());
    }
    let known_check_ids = known_check_ids(root)?;
    let check_patterns = check_mappings
        .iter()
        .flat_map(|entry| {
            entry
                .get("applies_to")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    for check_id in &known_check_ids {
        if !check_patterns
            .iter()
            .any(|pattern| wildcard_match(check_id, pattern))
        {
            errors.push(format!(
                "check `{check_id}` must cite policy authority via check-policy-authority-map"
            ));
        }
    }

    let contract_map_path = if registry.policy_authority_mappings.contracts
        == "configs/sources/governance/governance/contract-policy-authority-map.json"
    {
        contract_policy_authority_map_path(root)
    } else {
        root.join(&registry.policy_authority_mappings.contracts)
    };
    let contract_map: ContractPolicyAuthorityMap = read_json_file(&contract_map_path)?;
    if contract_map.schema_version != 1 {
        errors.push("contract policy authority map schema_version must be 1".to_string());
    }
    if !owner_format_valid(&contract_map.owner) {
        errors.push(format!(
            "contract policy authority map owner `{}` has invalid format",
            contract_map.owner
        ));
    }
    let policy_ids: BTreeSet<&str> = registry
        .policies
        .iter()
        .map(|policy| policy.id.as_str())
        .collect();
    if policy_ids.is_empty() {
        errors.push("governance policies registry must declare at least one policy".to_string());
    }
    for policy in &registry.policies {
        if policy.authority.trim().is_empty() {
            errors.push(format!("policy `{}` must declare authority", policy.id));
        }
        if policy.enforcement.is_empty() {
            errors.push(format!(
                "policy `{}` must declare at least one enforcement mechanism",
                policy.id
            ));
        }
    }
    let known_contract_ids = known_contract_ids(root)?;
    for mapping in &contract_map.mappings {
        if !policy_ids.contains(mapping.policy_id.as_str()) {
            errors.push(format!(
                "contract policy mapping references unknown policy `{}`",
                mapping.policy_id
            ));
        }
        if mapping.authority.trim().is_empty() {
            errors.push(format!(
                "contract policy mapping for `{}` must declare authority",
                mapping.contract_id
            ));
        }
        if !mapping.contract_id.contains('*') && !known_contract_ids.contains(&mapping.contract_id)
        {
            errors.push(format!(
                "contract policy mapping references unknown contract `{}`",
                mapping.contract_id
            ));
        }
    }
    for contract_id in &known_contract_ids {
        if !contract_map
            .mappings
            .iter()
            .any(|mapping| wildcard_match(contract_id, &mapping.contract_id))
        {
            errors.push(format!(
                "contract `{contract_id}` must cite policy authority via contract-policy-authority-map"
            ));
        }
    }
    Ok(errors)
}

fn owner_format_valid(owner: &str) -> bool {
    let owner = owner.trim();
    if let Some(team) = owner.strip_prefix("team:") {
        return !team.is_empty()
            && team
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
    }
    if let Some(handle) = owner.strip_prefix('@') {
        return !handle.is_empty()
            && handle
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));
    }
    false
}

fn wildcard_match(value: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return value == pattern;
    }
    let mut remainder = value;
    let mut first = true;
    for part in pattern.split('*') {
        if part.is_empty() {
            continue;
        }
        if first {
            if !remainder.starts_with(part) {
                return false;
            }
            remainder = &remainder[part.len()..];
            first = false;
            continue;
        }
        let Some(index) = remainder.find(part) else {
            return false;
        };
        remainder = &remainder[index + part.len()..];
    }
    if !pattern.ends_with('*') && !remainder.is_empty() {
        return false;
    }
    true
}

fn validate_command_order(commands: &[String]) -> bool {
    let mut sorted = commands.to_vec();
    sorted.sort();
    sorted == commands
}

fn known_commands(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut commands = BTreeSet::new();
    let public_targets = read_json_value(&root.join("configs/sources/repository/makes/public-targets.json"))?;
    for row in public_targets
        .get("public_targets")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(name) = row.get("name").and_then(serde_json::Value::as_str) {
            commands.insert(format!("make {name}"));
        }
    }

    let command_index = read_json_value(&root.join("docs/_internal/generated/command-index.json"))?;
    for row in command_index
        .get("commands")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(id) = row.get("id").and_then(serde_json::Value::as_str) {
            commands.insert(format!("bijux dev atlas {id}"));
            if let Some(subcommands) = row.get("subcommands").and_then(serde_json::Value::as_array)
            {
                for subcommand in subcommands.iter().filter_map(serde_json::Value::as_str) {
                    commands.insert(format!("bijux dev atlas {id} {subcommand}"));
                }
            }
        }
    }

    for rel in [
        "makes/cargo.mk",
        "makes/root.mk",
        "makes/contracts.mk",
        "makes/configs.mk",
        "makes/docs.mk",
        "makes/k8s.mk",
    ] {
        let path = root.join(rel);
        let text = fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?;
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some((target, _)) = trimmed.split_once(':') {
                if !target.is_empty()
                    && !target.starts_with('.')
                    && target.chars().all(|ch| {
                        ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_')
                    })
                {
                    commands.insert(format!("make {target}"));
                }
            }
            if let Some(command) = trimmed
                .strip_prefix("@printf '%s\\n' \"run: ")
                .and_then(|value| value.strip_suffix('"'))
            {
                let canonical = command.replace("$(DEV_ATLAS)", "bijux dev atlas");
                commands.insert(canonical);
            }
        }
    }

    let contract_gate_map = read_json_value(&root.join("ops/inventory/contract-gate-map.json"))?;
    for row in contract_gate_map
        .get("mappings")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(command) = row.get("command").and_then(serde_json::Value::as_str) {
            commands.insert(command.to_string());
        }
        if let Some(command) = row.get("repro_command").and_then(serde_json::Value::as_str) {
            commands.insert(command.to_string());
        }
    }
    Ok(commands)
}

fn validate_checks_inventory(root: &Path) -> Result<serde_json::Value, String> {
    let checks_suite_schema = read_json_value(&suite_schema_path(root))?;
    let checks_registry_schema = read_json_value(&checks_registry_schema_path(root))?;
    let contracts_registry_schema = read_json_value(&contracts_registry_schema_path(root))?;
    let _: serde_json::Value = read_json_value(&suites_index_schema_path(root))?;
    let _: serde_json::Value = read_json_value(&tags_schema_path(root))?;
    let _: serde_json::Value = read_json_value(&check_groups_schema_path(root))?;
    let _: serde_json::Value = read_json_value(&contract_groups_schema_path(root))?;
    let _: serde_json::Value = read_json_value(&registry_completeness_policy_schema_path(root))?;
    let _: serde_json::Value = read_json_value(&default_jobs_policy_schema_path(root))?;
    let _: serde_json::Value = read_json_value(&suite_baseline_policy_schema_path(root))?;
    if checks_suite_schema
        .get("properties")
        .and_then(|value| value.get("schema_version"))
        .and_then(|value| value.get("const"))
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Err("suite schema must pin schema_version=1".to_string());
    }
    if checks_registry_schema
        .get("properties")
        .and_then(|value| value.get("registry_id"))
        .and_then(|value| value.get("const"))
        .and_then(serde_json::Value::as_str)
        != Some("checks-registry")
    {
        return Err("checks registry schema must pin registry_id=checks-registry".to_string());
    }
    if contracts_registry_schema
        .get("properties")
        .and_then(|value| value.get("registry_id"))
        .and_then(|value| value.get("const"))
        .and_then(serde_json::Value::as_str)
        != Some("contracts-registry")
    {
        return Err(
            "contracts registry schema must pin registry_id=contracts-registry".to_string(),
        );
    }

    let checks_suite: GovernanceSuiteRegistry = read_json_file(&checks_suite_path(root))?;
    let contracts_suite: GovernanceSuiteRegistry = read_json_file(&contracts_suite_path(root))?;
    let tests_suite: GovernanceSuiteRegistry = read_json_file(&tests_suite_path(root))?;
    let suites_index: GovernanceSuitesIndex = read_json_file(&suites_index_path(root))?;
    let checks_registry: ChecksRegistryCatalog = read_json_file(&checks_registry_path(root))?;
    let checks_schema_index: CheckReportSchemaIndex =
        read_json_file(&checks_schema_index_path(root))?;
    let contracts_registry: ContractsRegistryCatalog =
        read_json_file(&contracts_registry_path(root))?;
    let tags_taxonomy: GovernanceTagsTaxonomy = read_json_file(&tags_taxonomy_path(root))?;
    let check_groups: GroupRegistry = read_json_file(&check_groups_path(root))?;
    let contract_groups: GroupRegistry = read_json_file(&contract_groups_path(root))?;
    let completeness_policy: RegistryCompletenessPolicy =
        read_json_file(&registry_completeness_policy_path(root))?;
    let default_jobs_policy: DefaultJobsPolicy = read_json_file(&default_jobs_policy_path(root))?;
    let suite_baseline_policy: SuiteBaselinePolicy =
        read_json_file(&suite_baseline_policy_path(root))?;
    let exceptions_registry: ExceptionsRegistry = read_yaml_file(&exceptions_registry_path(root))?;
    let deprecations_registry: serde_yaml::Value =
        read_yaml_file(&deprecations_registry_path(root))?;
    let checks_docs =
        fs::read_to_string(root.join("docs/_internal/governance/checks-and-contracts.md"))
            .map_err(|err| format!("read checks-and-contracts.md failed: {err}"))?;
    let suite_membership_policy =
        fs::read_to_string(root.join("docs/_internal/governance/suite-membership-policy.md"))
            .map_err(|err| format!("read suite-membership-policy.md failed: {err}"))?;

    let mut errors = Vec::<String>::new();
    let mut check_ids = BTreeSet::new();
    let mut contract_ids = BTreeSet::new();
    let mut missing_rows = Vec::<serde_json::Value>::new();
    let mut referenced_check_schema_ids = BTreeSet::<String>::new();
    let allowed_tags = tags_taxonomy.tags.iter().cloned().collect::<BTreeSet<_>>();
    let known_check_groups = check_groups
        .groups
        .iter()
        .map(|group| group.id.clone())
        .collect::<BTreeSet<_>>();
    let known_contract_groups = contract_groups
        .groups
        .iter()
        .map(|group| group.id.clone())
        .collect::<BTreeSet<_>>();
    let active_exception_scopes = exceptions_registry
        .exceptions
        .iter()
        .map(|item| format!("{}:{}", item.scope.kind, item.scope.id))
        .collect::<BTreeSet<_>>();
    let deprecation_rows = deprecations_registry["deprecations"]
        .as_sequence()
        .cloned()
        .unwrap_or_default();
    let resolvable_commands = known_commands(root)?;
    let expected_suite_files = [
        "checks.suite.json".to_string(),
        "contracts.suite.json".to_string(),
        "tests.suite.json".to_string(),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let indexed_suite_files = suites_index.suites.iter().cloned().collect::<BTreeSet<_>>();
    let check_schema_ids = checks_schema_index
        .schemas
        .iter()
        .map(|entry| entry.report_id.as_str())
        .collect::<BTreeSet<_>>();

    for suite in [&checks_suite, &contracts_suite, &tests_suite] {
        if suite.schema_version != 1 {
            errors.push(format!(
                "suite `{}` must declare schema_version=1",
                suite.suite_id
            ));
        }
        if suite.purpose.trim().is_empty() {
            errors.push(format!("suite `{}` must declare purpose", suite.suite_id));
        }
        if !owner_format_valid(&suite.owner) {
            errors.push(format!(
                "suite `{}` owner `{}` has invalid format",
                suite.suite_id, suite.owner
            ));
        }
        if !matches!(suite.stability.as_str(), "draft" | "stable" | "deprecated") {
            errors.push(format!(
                "suite `{}` stability `{}` is invalid",
                suite.suite_id, suite.stability
            ));
        }
        for entry in &suite.entries {
            if entry.id.trim().is_empty() {
                errors.push(format!(
                    "suite `{}` contains an entry with empty id",
                    suite.suite_id
                ));
            }
            if !matches!(entry.kind.as_str(), "check" | "contract") {
                errors.push(format!(
                    "suite `{}` entry `{}` has invalid kind `{}`",
                    suite.suite_id, entry.id, entry.kind
                ));
            }
            if !matches!(entry.mode.as_str(), "pure" | "effect") {
                errors.push(format!(
                    "suite `{}` entry `{}` has invalid mode `{}`",
                    suite.suite_id, entry.id, entry.mode
                ));
            }
            if !owner_format_valid(&entry.owner) {
                errors.push(format!(
                    "suite `{}` entry `{}` owner `{}` has invalid format",
                    suite.suite_id, entry.id, entry.owner
                ));
            }
            for tag in &entry.tags {
                if !allowed_tags.contains(tag) {
                    errors.push(format!(
                        "suite `{}` entry `{}` uses unknown tag `{}`",
                        suite.suite_id, entry.id, tag
                    ));
                }
            }
        }
    }

    if checks_suite.suite_id != "checks" {
        errors.push("checks suite must use suite_id `checks`".to_string());
    }
    if contracts_suite.suite_id != "contracts" {
        errors.push("contracts suite must use suite_id `contracts`".to_string());
    }
    if tests_suite.suite_id != "tests" {
        errors.push("tests suite must use suite_id `tests`".to_string());
    }
    if suites_index.schema_version != 1 || suites_index.index_id != "governance-suites" {
        errors.push(
            "suites index must declare schema_version=1 and index_id=governance-suites".to_string(),
        );
    }
    if !owner_format_valid(&suites_index.owner) {
        errors.push(format!(
            "suites index owner `{}` has invalid format",
            suites_index.owner
        ));
    }
    if indexed_suite_files != expected_suite_files {
        errors.push(format!(
            "suites index must match suite files on disk: expected {:?}, found {:?}",
            expected_suite_files, indexed_suite_files
        ));
    }
    if tags_taxonomy.schema_version != 1 || tags_taxonomy.taxonomy_id != "governance-tags" {
        errors.push(
            "tags taxonomy must declare schema_version=1 and taxonomy_id=governance-tags"
                .to_string(),
        );
    }
    if !owner_format_valid(&tags_taxonomy.owner) {
        errors.push(format!(
            "tags taxonomy owner `{}` has invalid format",
            tags_taxonomy.owner
        ));
    }
    if check_groups.schema_version != 1 || check_groups.group_set_id != "check-groups" {
        errors.push(
            "check groups must declare schema_version=1 and group_set_id=check-groups".to_string(),
        );
    }
    if !owner_format_valid(&check_groups.owner) {
        errors.push(format!(
            "check groups owner `{}` has invalid format",
            check_groups.owner
        ));
    }
    if contract_groups.schema_version != 1 || contract_groups.group_set_id != "contract-groups" {
        errors.push(
            "contract groups must declare schema_version=1 and group_set_id=contract-groups"
                .to_string(),
        );
    }
    if !owner_format_valid(&contract_groups.owner) {
        errors.push(format!(
            "contract groups owner `{}` has invalid format",
            contract_groups.owner
        ));
    }
    if completeness_policy.schema_version != 1
        || completeness_policy.policy_id != "registry-completeness-threshold"
    {
        errors.push("registry completeness policy must declare schema_version=1 and policy_id=registry-completeness-threshold".to_string());
    }
    if !owner_format_valid(&completeness_policy.owner) {
        errors.push(format!(
            "registry completeness policy owner `{}` has invalid format",
            completeness_policy.owner
        ));
    }
    if default_jobs_policy.schema_version != 1
        || default_jobs_policy.policy_id != "suite-default-jobs"
    {
        errors.push(
            "default jobs policy must declare schema_version=1 and policy_id=suite-default-jobs"
                .to_string(),
        );
    }
    if !owner_format_valid(&default_jobs_policy.owner) {
        errors.push(format!(
            "default jobs policy owner `{}` has invalid format",
            default_jobs_policy.owner
        ));
    }
    for entry in &default_jobs_policy.suites {
        if !matches!(entry.suite_id.as_str(), "checks" | "contracts") {
            errors.push(format!(
                "default jobs policy references unknown suite `{}`",
                entry.suite_id
            ));
        }
        if entry.low_core_cap > entry.auto_jobs {
            errors.push(format!(
                "default jobs policy for `{}` cannot set low_core_cap above auto_jobs",
                entry.suite_id
            ));
        }
    }
    for required_suite in ["checks", "contracts"] {
        if !default_jobs_policy
            .suites
            .iter()
            .any(|entry| entry.suite_id == required_suite)
        {
            errors.push(format!(
                "default jobs policy must declare suite `{required_suite}`"
            ));
        }
        if !suite_baseline_policy
            .suites
            .iter()
            .any(|entry| entry.suite_id == required_suite)
        {
            errors.push(format!(
                "suite baseline policy must declare suite `{required_suite}`"
            ));
        }
    }
    if suite_baseline_policy.schema_version != 1
        || suite_baseline_policy.baseline_id != "suite-baseline"
    {
        errors.push(
            "suite baseline policy must declare schema_version=1 and baseline_id=suite-baseline"
                .to_string(),
        );
    }
    if !owner_format_valid(&suite_baseline_policy.owner) {
        errors.push(format!(
            "suite baseline policy owner `{}` has invalid format",
            suite_baseline_policy.owner
        ));
    }
    if checks_schema_index.schema_version != 1
        || checks_schema_index.index_id != "checks-schema-index"
    {
        errors.push(
            "check schema index must declare schema_version=1 and index_id=checks-schema-index"
                .to_string(),
        );
    }
    for entry in &checks_schema_index.schemas {
        if entry.report_id.trim().is_empty() {
            errors.push("check schema index report_id must not be blank".to_string());
        }
        if entry.schema_path.trim().is_empty() {
            errors.push(format!(
                "check schema index entry `{}` missing schema_path",
                entry.report_id
            ));
        } else if !root.join(&entry.schema_path).is_file() {
            errors.push(format!(
                "check schema index entry `{}` points to missing schema `{}`",
                entry.report_id, entry.schema_path
            ));
        }
    }

    if checks_registry.schema_version != 1 || checks_registry.registry_id != "checks-registry" {
        errors.push(
            "checks registry must declare schema_version=1 and registry_id=checks-registry"
                .to_string(),
        );
    }
    for check in &checks_registry.checks {
        if !check_ids.insert(check.check_id.clone()) {
            errors.push(format!(
                "duplicate check id `{}` in checks registry",
                check.check_id
            ));
        }
        if check.summary.trim().is_empty() {
            errors.push(format!("{} missing summary", check.check_id));
        }
        if !owner_format_valid(&check.owner) {
            errors.push(format!(
                "{} owner `{}` has invalid format",
                check.check_id, check.owner
            ));
        }
        if check.owner.contains('+') || check.owner.contains(',') {
            errors.push(format!("{} must have exactly one owner", check.check_id));
        }
        if !matches!(check.mode.as_str(), "pure" | "effect") {
            errors.push(format!(
                "{} mode `{}` is invalid",
                check.check_id, check.mode
            ));
        }
        if !known_check_groups.contains(&check.group) {
            errors.push(format!(
                "{} group `{}` is invalid",
                check.check_id, check.group
            ));
        }
        if check.inputs.is_empty() || check.outputs.is_empty() {
            errors.push(format!(
                "{} must declare non-empty inputs and outputs",
                check.check_id
            ));
            missing_rows.push(serde_json::json!({"kind":"check","id":check.check_id,"missing":["inputs_or_outputs"]}));
        }
        if check.commands.is_empty() {
            errors.push(format!(
                "{} must declare at least one command",
                check.check_id
            ));
            missing_rows.push(
                serde_json::json!({"kind":"check","id":check.check_id,"missing":["commands"]}),
            );
        }
        if !validate_command_order(&check.commands) {
            errors.push(format!(
                "{} commands must be stored in deterministic sorted order",
                check.check_id
            ));
        }
        for command in &check.commands {
            if !resolvable_commands.contains(command) {
                errors.push(format!(
                    "{} command `{}` is not resolvable from known makes/control-plane inventory",
                    check.check_id, command
                ));
            }
        }
        if check.reports.is_empty() {
            errors.push(format!(
                "{} must declare at least one report artifact",
                check.check_id
            ));
            missing_rows.push(
                serde_json::json!({"kind":"check","id":check.check_id,"missing":["reports"]}),
            );
        }
        if check.report_ids.is_empty() {
            errors.push(format!(
                "{} must declare at least one report id",
                check.check_id
            ));
            missing_rows.push(
                serde_json::json!({"kind":"check","id":check.check_id,"missing":["report_ids"]}),
            );
        }
        if check.report_ids.len() != check.reports.len() {
            errors.push(format!(
                "{} report_ids count must match reports count",
                check.check_id
            ));
        }
        for report_id in &check.report_ids {
            referenced_check_schema_ids.insert(report_id.clone());
            if !check_schema_ids.contains(report_id.as_str()) {
                errors.push(format!(
                    "{} references unknown check report id `{}`",
                    check.check_id, report_id
                ));
            }
        }
        for report in &check.reports {
            if !report.contains(&check.check_id) {
                errors.push(format!(
                    "{} report path `{}` must include the check id",
                    check.check_id, report
                ));
            }
        }
        if check.suite_membership.is_empty() {
            errors.push(format!("{} must declare suite_membership", check.check_id));
            missing_rows.push(serde_json::json!({"kind":"check","id":check.check_id,"missing":["suite_membership"]}));
        }
        if !matches!(
            check.severity.as_str(),
            "blocker" | "major" | "minor" | "info"
        ) {
            errors.push(format!(
                "{} severity `{}` is invalid",
                check.check_id, check.severity
            ));
        }
        if !matches!(check.stage.as_str(), "local" | "pr" | "merge" | "release") {
            errors.push(format!(
                "{} stage `{}` is invalid",
                check.check_id, check.stage
            ));
        }
        if !matches!(check.runtime_cost.as_str(), "low" | "medium" | "high") {
            errors.push(format!(
                "{} runtime_cost `{}` is invalid",
                check.check_id, check.runtime_cost
            ));
        }
        if !matches!(
            check.determinism.as_str(),
            "strict" | "bounded" | "best-effort"
        ) {
            errors.push(format!(
                "{} determinism `{}` is invalid",
                check.check_id, check.determinism
            ));
        }
        if let Some(tags) = &check.tags {
            if tags.is_empty() {
                errors.push(format!(
                    "{} tags must not be empty when present",
                    check.check_id
                ));
            }
            for tag in tags {
                if !allowed_tags.contains(tag) {
                    errors.push(format!("{} uses unknown tag `{}`", check.check_id, tag));
                }
            }
        }
        if let Some(depends_on) = &check.depends_on {
            if depends_on.iter().any(|value| value.trim().is_empty()) {
                errors.push(format!(
                    "{} depends_on must not contain empty ids",
                    check.check_id
                ));
            }
        }
        if let Some(replaces) = &check.replaces {
            if replaces.iter().any(|value| value.trim().is_empty()) {
                errors.push(format!(
                    "{} replaces must not contain empty ids",
                    check.check_id
                ));
            }
        }
        if let Some(since_version) = &check.since_version {
            if since_version.trim().is_empty() {
                errors.push(format!(
                    "{} since_version must not be blank",
                    check.check_id
                ));
            }
        }
        if let Some(retries) = check.retries {
            let has_flaky_tag = check
                .tags
                .as_ref()
                .map(|tags| tags.iter().any(|tag| tag == "flaky"))
                .unwrap_or(false);
            if retries > 1 && !has_flaky_tag {
                errors.push(format!(
                    "{} retries `{}` exceeds 1 without flaky tag",
                    check.check_id, retries
                ));
            }
        }
        if let Some(overlaps_with) = &check.overlaps_with {
            if overlaps_with.iter().any(|value| value.trim().is_empty()) {
                errors.push(format!(
                    "{} overlaps_with must not contain blank ids",
                    check.check_id
                ));
            }
        }
        if let Some(requires_tools) = &check.requires_tools {
            if requires_tools.iter().any(|value| value.trim().is_empty()) {
                errors.push(format!(
                    "{} requires_tools must not contain blank tool names",
                    check.check_id
                ));
            }
        }
        if let Some(policy) = &check.missing_tools_policy {
            if !matches!(policy.as_str(), "fail" | "skip") {
                errors.push(format!(
                    "{} missing_tools_policy `{}` is invalid",
                    check.check_id, policy
                ));
            }
        }
        if !check.suite_membership.iter().any(|suite| suite == "checks") {
            errors.push(format!(
                "{} suite_membership must include `checks`",
                check.check_id
            ));
        }
        if check.suite_membership.len() != 1 {
            errors.push(format!(
                "{} suite_membership must remain singular; use overlaps_with for deliberate overlap",
                check.check_id
            ));
        }
        if let Some(cpu_hint) = &check.cpu_hint {
            if !matches!(cpu_hint.as_str(), "light" | "moderate" | "heavy") {
                errors.push(format!(
                    "{} cpu_hint `{}` is invalid",
                    check.check_id, cpu_hint
                ));
            }
        } else {
            errors.push(format!("{} missing cpu_hint", check.check_id));
            missing_rows.push(
                serde_json::json!({"kind":"check","id":check.check_id,"missing":["cpu_hint"]}),
            );
        }
        if let Some(mem_hint) = &check.mem_hint {
            if !matches!(mem_hint.as_str(), "low" | "medium" | "high") {
                errors.push(format!(
                    "{} mem_hint `{}` is invalid",
                    check.check_id, mem_hint
                ));
            }
        } else {
            errors.push(format!("{} missing mem_hint", check.check_id));
            missing_rows.push(
                serde_json::json!({"kind":"check","id":check.check_id,"missing":["mem_hint"]}),
            );
        }
        let has_flaky_tag = check
            .tags
            .as_ref()
            .map(|tags| tags.iter().any(|tag| tag == "flaky"))
            .unwrap_or(false);
        if has_flaky_tag
            && !active_exception_scopes.contains(&format!("check:{}", check.check_id))
            && !active_exception_scopes.contains(&format!("contract:{}", check.check_id))
        {
            errors.push(format!(
                "{} uses flaky tag without an active exception entry",
                check.check_id
            ));
        }
    }
    for entry in &checks_schema_index.schemas {
        if !referenced_check_schema_ids.contains(&entry.report_id) {
            errors.push(format!(
                "check schema `{}` is not referenced by any governed check",
                entry.report_id
            ));
        }
    }

    if contracts_registry.schema_version != 1
        || contracts_registry.registry_id != "contracts-registry"
    {
        errors.push(
            "contracts registry must declare schema_version=1 and registry_id=contracts-registry"
                .to_string(),
        );
    }
    for contract in &contracts_registry.contracts {
        if !contract_ids.insert(contract.contract_id.clone()) {
            errors.push(format!(
                "duplicate contract id `{}` in contracts registry",
                contract.contract_id
            ));
        }
        if contract.summary.trim().is_empty() {
            errors.push(format!("{} missing summary", contract.contract_id));
        }
        if !owner_format_valid(&contract.owner) {
            errors.push(format!(
                "{} owner `{}` has invalid format",
                contract.contract_id, contract.owner
            ));
        }
        if contract.owner.contains('+') || contract.owner.contains(',') {
            errors.push(format!(
                "{} must have exactly one owner",
                contract.contract_id
            ));
        }
        if !matches!(contract.mode.as_str(), "pure" | "effect") {
            errors.push(format!(
                "{} mode `{}` is invalid",
                contract.contract_id, contract.mode
            ));
        }
        if !known_contract_groups.contains(&contract.group) {
            errors.push(format!(
                "{} group `{}` is invalid",
                contract.contract_id, contract.group
            ));
        }
        if contract.runner.trim().is_empty() {
            errors.push(format!("{} missing runner", contract.contract_id));
            missing_rows.push(serde_json::json!({"kind":"contract","id":contract.contract_id,"missing":["runner"]}));
        } else if !resolvable_commands.contains(&contract.runner) {
            errors.push(format!(
                "{} runner `{}` is not resolvable from known makes/control-plane inventory",
                contract.contract_id, contract.runner
            ));
        }
        if contract.reports.is_empty() {
            errors.push(format!(
                "{} must declare at least one report artifact",
                contract.contract_id
            ));
            missing_rows.push(serde_json::json!({"kind":"contract","id":contract.contract_id,"missing":["reports"]}));
        }
        if contract.suite_membership.is_empty() {
            errors.push(format!(
                "{} must declare suite_membership",
                contract.contract_id
            ));
            missing_rows.push(serde_json::json!({"kind":"contract","id":contract.contract_id,"missing":["suite_membership"]}));
        }
        if let Some(tags) = &contract.tags {
            for tag in tags {
                if !allowed_tags.contains(tag) {
                    errors.push(format!(
                        "{} uses unknown tag `{}`",
                        contract.contract_id, tag
                    ));
                }
            }
        }
        if let Some(retries) = contract.retries {
            let has_flaky_tag = contract
                .tags
                .as_ref()
                .map(|tags| tags.iter().any(|tag| tag == "flaky"))
                .unwrap_or(false);
            if retries > 1 && !has_flaky_tag {
                errors.push(format!(
                    "{} retries `{}` exceeds 1 without flaky tag",
                    contract.contract_id, retries
                ));
            }
        }
        if let Some(overlaps_with) = &contract.overlaps_with {
            if overlaps_with.iter().any(|value| value.trim().is_empty()) {
                errors.push(format!(
                    "{} overlaps_with must not contain blank ids",
                    contract.contract_id
                ));
            }
        }
        if let Some(requires_tools) = &contract.requires_tools {
            if requires_tools.iter().any(|value| value.trim().is_empty()) {
                errors.push(format!(
                    "{} requires_tools must not contain blank tool names",
                    contract.contract_id
                ));
            }
        }
        if let Some(policy) = &contract.missing_tools_policy {
            if !matches!(policy.as_str(), "fail" | "skip") {
                errors.push(format!(
                    "{} missing_tools_policy `{}` is invalid",
                    contract.contract_id, policy
                ));
            }
        }
        if !contract
            .suite_membership
            .iter()
            .any(|suite| suite == "contracts")
        {
            errors.push(format!(
                "{} suite_membership must include `contracts`",
                contract.contract_id
            ));
        }
        if contract.suite_membership.len() != 1 {
            errors.push(format!(
                "{} suite_membership must remain singular; use overlaps_with for deliberate overlap",
                contract.contract_id
            ));
        }
    }

    if !checks_docs.contains("## Checks")
        || !checks_docs.contains("## Contracts")
        || !checks_docs.contains("## Pure And Effect")
    {
        errors.push(
            "docs/_internal/governance/checks-and-contracts.md must define checks, contracts, and pure/effect semantics"
                .to_string(),
        );
    }
    if !checks_docs.contains("idempotent") {
        errors.push(
            "docs/_internal/governance/checks-and-contracts.md must define the idempotent checks rule"
                .to_string(),
        );
    }
    if !checks_docs.contains("## Suite Boundaries") || !checks_docs.contains("## Validation System")
    {
        errors.push(
            "docs/_internal/governance/checks-and-contracts.md must define suite boundaries and the validation system table"
                .to_string(),
        );
    }
    if !suite_membership_policy.contains("## Membership boundary")
        || !suite_membership_policy.contains("## Allowed overlap")
        || !suite_membership_policy.contains("## How to move an entry")
    {
        errors.push(
            "docs/_internal/governance/suite-membership-policy.md must define membership boundary, allowed overlap, and move procedure"
                .to_string(),
        );
    }

    let checks_suite_ids = checks_suite
        .entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    let contracts_suite_ids = contracts_suite
        .entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    let checks_suite_tag_count: usize = checks_suite
        .entries
        .iter()
        .map(|entry| entry.tags.len())
        .sum();
    let contracts_suite_tag_count: usize = contracts_suite
        .entries
        .iter()
        .map(|entry| entry.tags.len())
        .sum();
    let tests_suite_tag_count: usize = tests_suite
        .entries
        .iter()
        .map(|entry| entry.tags.len())
        .sum();

    for id in &check_ids {
        if !checks_suite_ids.contains(id) {
            errors.push(format!("checks suite missing registry id `{id}`"));
        }
    }
    for id in &contract_ids {
        if !contracts_suite_ids.contains(id) {
            errors.push(format!("contracts suite missing registry id `{id}`"));
        }
    }
    for check in &checks_registry.checks {
        if let Some(overlaps_with) = &check.overlaps_with {
            for overlap in overlaps_with {
                if !check_ids.contains(overlap) && !contract_ids.contains(overlap) {
                    errors.push(format!(
                        "{} overlaps_with unknown id `{}`",
                        check.check_id, overlap
                    ));
                }
            }
        }
    }
    for contract in &contracts_registry.contracts {
        if let Some(overlaps_with) = &contract.overlaps_with {
            for overlap in overlaps_with {
                if !check_ids.contains(overlap) && !contract_ids.contains(overlap) {
                    errors.push(format!(
                        "{} overlaps_with unknown id `{}`",
                        contract.contract_id, overlap
                    ));
                }
            }
        }
    }
    for baseline in &suite_baseline_policy.suites {
        let (current_total, current_groups) = match baseline.suite_id.as_str() {
            "checks" => (
                checks_suite_ids.len() as u64,
                checks_registry.checks.iter().fold(
                    BTreeMap::<String, u64>::new(),
                    |mut acc, check| {
                        *acc.entry(check.group.clone()).or_insert(0) += 1;
                        acc
                    },
                ),
            ),
            "contracts" => (
                contracts_suite_ids.len() as u64,
                contracts_registry.contracts.iter().fold(
                    BTreeMap::<String, u64>::new(),
                    |mut acc, contract| {
                        *acc.entry(contract.group.clone()).or_insert(0) += 1;
                        acc
                    },
                ),
            ),
            other => {
                errors.push(format!("suite baseline references unknown suite `{other}`"));
                continue;
            }
        };
        if current_total < baseline.expected_total {
            errors.push(format!(
                "suite `{}` shrank below baseline: expected at least {}, found {}",
                baseline.suite_id, baseline.expected_total, current_total
            ));
        }
        for (group, minimum) in &baseline.minimum_group_counts {
            let actual = current_groups.get(group).copied().unwrap_or(0);
            if actual < *minimum {
                errors.push(format!(
                    "suite `{}` group `{}` fell below baseline minimum: expected at least {}, found {}",
                    baseline.suite_id, group, minimum, actual
                ));
            }
        }
    }
    for row in &deprecation_rows {
        let Some(surface) = row.get("surface").and_then(serde_yaml::Value::as_str) else {
            continue;
        };
        if surface != "check-id" {
            continue;
        }
        let Some(old_name) = row.get("old_name").and_then(serde_yaml::Value::as_str) else {
            continue;
        };
        let Some(removal_target) = row
            .get("removal_target")
            .and_then(serde_yaml::Value::as_str)
        else {
            continue;
        };
        if !is_iso_date(removal_target) {
            errors.push(format!(
                "check deprecation `{old_name}` has invalid removal_target `{removal_target}`"
            ));
            continue;
        }
        if old_name.starts_with("CHECK-")
            && check_ids.contains(old_name)
            && removal_target < "2026-03-03"
        {
            errors.push(format!(
                "deprecated check `{old_name}` is past removal_target `{removal_target}` and must be removed or renewed"
            ));
        }
    }
    for entry in &checks_suite.entries {
        if !check_ids.contains(&entry.id) {
            errors.push(format!(
                "checks suite entry references unknown check `{}`",
                entry.id
            ));
        }
    }
    for entry in &contracts_suite.entries {
        if !contract_ids.contains(&entry.id) {
            errors.push(format!(
                "contracts suite entry references unknown contract `{}`",
                entry.id
            ));
        }
    }

    for group in &check_groups.groups {
        if group.summary.trim().is_empty() {
            errors.push(format!("check group `{}` must define summary", group.id));
        }
        if !checks_registry
            .checks
            .iter()
            .any(|check| check.group == group.id)
        {
            errors.push(format!(
                "check group `{}` has no checks and should be removed",
                group.id
            ));
        }
    }
    for group in &contract_groups.groups {
        if group.summary.trim().is_empty() {
            errors.push(format!("contract group `{}` must define summary", group.id));
        }
        if !contracts_registry
            .contracts
            .iter()
            .any(|contract| contract.group == group.id)
        {
            errors.push(format!(
                "contract group `{}` has no contracts and should be removed",
                group.id
            ));
        }
    }

    let required_fields_total =
        (checks_registry.checks.len() * 5) + (contracts_registry.contracts.len() * 4);
    let required_fields_missing = missing_rows
        .iter()
        .map(|row| {
            row.get("missing")
                .and_then(serde_json::Value::as_array)
                .map_or(0, |items| items.len())
        })
        .sum::<usize>();
    let required_fields_present = required_fields_total.saturating_sub(required_fields_missing);
    let coverage_percent = if required_fields_total == 0 {
        100
    } else {
        ((required_fields_present * 100) / required_fields_total) as u64
    };
    let missing_fields_report = serde_json::json!({
        "report_id": "registry-missing-fields",
        "version": 1,
        "inputs": {
            "checks_registry": "configs/sources/governance/governance/checks.registry.json",
            "contracts_registry": "configs/sources/governance/governance/contracts.registry.json",
            "completeness_policy": "configs/sources/governance/governance/registry-completeness-policy.json"
        },
        "status": if missing_rows.is_empty() { "ok" } else { "failed" },
        "summary": {
            "rows": missing_rows.len(),
            "required_field_coverage_percent": coverage_percent,
            "threshold_percent": completeness_policy.required_field_coverage_percent,
            "required_rule_count": completeness_policy.required_rules.len()
        },
        "rows": missing_rows,
        "errors": if coverage_percent < completeness_policy.required_field_coverage_percent {
            vec![format!(
                "registry completeness below threshold: {} < {}",
                coverage_percent, completeness_policy.required_field_coverage_percent
            )]
        } else {
            Vec::<String>::new()
        }
    });
    validate_named_report(
        root,
        "registry-missing-fields.schema.json",
        &missing_fields_report,
    )?;
    write_pretty_json(&registry_missing_fields_path(root), &missing_fields_report)?;
    if coverage_percent < completeness_policy.required_field_coverage_percent {
        errors.push(format!(
            "registry completeness below threshold: {} < {}",
            coverage_percent, completeness_policy.required_field_coverage_percent
        ));
    }

    errors.sort();
    errors.dedup();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "checks_inventory",
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "counts": {
            "checks": check_ids.len(),
            "contracts": contract_ids.len(),
            "checks_suite_entries": checks_suite.entries.len(),
            "contracts_suite_entries": contracts_suite.entries.len(),
            "tests_suite_entries": tests_suite.entries.len(),
            "checks_suite_tags": checks_suite_tag_count,
            "contracts_suite_tags": contracts_suite_tag_count,
            "tests_suite_tags": tests_suite_tag_count,
            "required_field_coverage_percent": coverage_percent
        },
        "artifacts": {
            "checks_registry": checks_registry_path(root).strip_prefix(root).unwrap_or(&checks_registry_path(root)).display().to_string(),
            "contracts_registry": contracts_registry_path(root).strip_prefix(root).unwrap_or(&contracts_registry_path(root)).display().to_string(),
            "checks_suite": checks_suite_path(root).strip_prefix(root).unwrap_or(&checks_suite_path(root)).display().to_string(),
            "contracts_suite": contracts_suite_path(root).strip_prefix(root).unwrap_or(&contracts_suite_path(root)).display().to_string(),
            "tests_suite": tests_suite_path(root).strip_prefix(root).unwrap_or(&tests_suite_path(root)).display().to_string(),
            "guide": "docs/_internal/governance/checks-and-contracts.md",
            "suites_index": suites_index_path(root).strip_prefix(root).unwrap_or(&suites_index_path(root)).display().to_string(),
            "tags_taxonomy": tags_taxonomy_path(root).strip_prefix(root).unwrap_or(&tags_taxonomy_path(root)).display().to_string(),
            "check_groups": check_groups_path(root).strip_prefix(root).unwrap_or(&check_groups_path(root)).display().to_string(),
            "contract_groups": contract_groups_path(root).strip_prefix(root).unwrap_or(&contract_groups_path(root)).display().to_string(),
            "missing_fields_report": registry_missing_fields_path(root).strip_prefix(root).unwrap_or(&registry_missing_fields_path(root)).display().to_string()
        },
        "errors": errors
    });
    write_pretty_json(&checks_inventory_path(root), &payload)?;
    Ok(payload)
}

fn load_compatibility_policy(root: &Path) -> Result<CompatibilityPolicyRegistry, String> {
    let path = compatibility_policy_path(root);
    serde_yaml::from_str(
        &fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn load_deprecations_registry(root: &Path) -> Result<DeprecationsRegistry, String> {
    let path = deprecations_registry_path(root);
    serde_yaml::from_str(
        &fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
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
        current = match current
            .get("properties")
            .and_then(|value| value.get(segment))
        {
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
            &fs::read_to_string(&path)
                .map_err(|err| format!("read {} failed: {err}", path.display()))?,
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
        &fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn semver_major(version: &str) -> Option<u64> {
    version.split('.').next()?.parse().ok()
}

fn release_breaking_notes_meta_path(root: &Path) -> PathBuf {
    root.join("ops/release/notes/breaking.json")
}

fn release_breaking_notes_schema_path(root: &Path) -> PathBuf {
    root.join("ops/release/notes/breaking.schema.json")
}

fn load_breaking_notes_meta(root: &Path) -> Result<serde_json::Value, String> {
    let path = release_breaking_notes_meta_path(root);
    let schema_path = release_breaking_notes_schema_path(root);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let schema = read_json_value(&schema_path)?;
    if schema
        .get("properties")
        .and_then(|value| value.get("schema_version"))
        .and_then(|value| value.get("const"))
        .and_then(serde_json::Value::as_u64)
        != value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
    {
        return Err(format!(
            "{} schema_version does not match {}",
            path.display(),
            schema_path.display()
        ));
    }
    if value
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .is_none()
    {
        return Err(format!(
            "{} front matter must declare entries array",
            path.display()
        ));
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

fn render_institutional_delta_markdown(inputs: &serde_json::Value) -> String {
    let mut out = String::from("# Institutional Delta\n\n");
    out.push_str(
        "Deterministic release delta generated from governed compatibility artifacts.\n\n",
    );
    out.push_str("## Breaking changes\n\n");
    let breaking = inputs
        .get("breaking_changes")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if breaking.is_empty() {
        out.push_str("- None.\n");
    } else {
        for row in breaking {
            let id = row
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let change = row
                .get("change")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            out.push_str(&format!("- `{id}`: {change}\n"));
        }
    }
    out.push_str("\n## Active deprecations\n\n");
    let deprecations = inputs
        .get("deprecations")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if deprecations.is_empty() {
        out.push_str("- None.\n");
    } else {
        for row in deprecations {
            let id = row
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let old_name = row
                .get("old_name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let new_name = row
                .get("new_name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let removal_target = row
                .get("removal_target")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            out.push_str(&format!(
                "- `{id}`: `{old_name}` -> `{new_name}` (removal target `{removal_target}`)\n"
            ));
        }
    }
    out
}

fn adr_field_from_markdown(markdown: &str, key: &str) -> Option<String> {
    let prefix = format!("- {key}:");
    markdown.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)
            .map(|value| value.trim().to_string())
    })
}

fn governance_adr_index_payload(root: &Path) -> Result<serde_json::Value, String> {
    let decisions_dir = root.join("docs/governance/decisions");
    let mut rows = Vec::new();
    if decisions_dir.exists() {
        for entry in fs::read_dir(&decisions_dir)
            .map_err(|e| format!("read {} failed: {e}", decisions_dir.display()))?
        {
            let entry =
                entry.map_err(|e| format!("read {} entry failed: {e}", decisions_dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            let text = fs::read_to_string(&path)
                .map_err(|e| format!("read {} failed: {e}", path.display()))?;
            let status = adr_field_from_markdown(&text, "Status").unwrap_or_default();
            let date = adr_field_from_markdown(&text, "Date").unwrap_or_default();
            let owners = adr_field_from_markdown(&text, "Owners").unwrap_or_default();
            let title = text
                .lines()
                .find_map(|line| line.strip_prefix("# ").map(ToString::to_string))
                .unwrap_or_else(|| rel.clone());
            rows.push(serde_json::json!({
                "id": path.file_stem().and_then(|value| value.to_str()).unwrap_or_default(),
                "title": title,
                "status": status,
                "date": date,
                "owners": owners,
                "path": rel,
            }));
        }
    }
    rows.sort_by(|left, right| {
        left["id"]
            .as_str()
            .unwrap_or_default()
            .cmp(right["id"].as_str().unwrap_or_default())
    });
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "governance_adr_index",
        "count": rows.len(),
        "entries": rows,
    }))
}

fn required_governance_docs() -> &'static [&'static str] {
    &[
        "docs/governance/governance-charter.md",
        "docs/governance/project-governance-model.md",
        "docs/governance/maintainers-and-roles.md",
        "docs/governance/decision-process.md",
        "docs/governance/contributor-onboarding-workflow.md",
        "docs/governance/adr-registry.md",
        "docs/governance/adr-template.md",
    ]
}

fn governance_docs_validation_errors(root: &Path) -> Vec<String> {
    required_governance_docs()
        .iter()
        .filter_map(|rel| {
            let path = root.join(rel);
            (!path.exists()).then(|| format!("required governance document missing: {rel}"))
        })
        .collect()
}

fn contributor_guideline_validation_errors(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let onboarding_path = root.join("docs/governance/contributor-onboarding-workflow.md");
    let text = match fs::read_to_string(&onboarding_path) {
        Ok(value) => value,
        Err(_) => {
            errors.push(format!(
                "contributor onboarding workflow missing: {}",
                onboarding_path
                    .strip_prefix(root)
                    .unwrap_or(&onboarding_path)
                    .display()
            ));
            return errors;
        }
    };
    for required in [
        "governance check --format json",
        "governance validate --format json",
        "maintainer signoff",
    ] {
        if !text.contains(required) {
            errors.push(format!(
                "contributor onboarding workflow missing required guidance: {required}"
            ));
        }
    }
    errors
}

pub(crate) fn run_governance_command(
    _quiet: bool,
    command: GovernanceCommand,
) -> Result<(String, i32), String> {
    match command {
        GovernanceCommand::Version {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let version = governance_version_value(&root)?;
            let source_path = governance_registry_path(&root);
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_version",
                "governance_version": version,
                "source": source_path.strip_prefix(&root).unwrap_or(&source_path).display().to_string(),
            });
            let rendered = emit_payload(format, out, &payload)?;
            Ok((rendered, 0))
        }
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
            if let Ok(registry) = governance_enforcement::load_registry(&root) {
                if let Some(rule) = registry.rules.iter().find(|row| row.id == id) {
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "kind": "governance_rule_explain",
                        "status": "ok",
                        "rule": rule,
                    });
                    let rendered = emit_payload(format, out, &payload)?;
                    return Ok((rendered, 0));
                }
            }
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
        GovernanceCommand::Check {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let registry = governance_enforcement::load_registry(&root)?;
            let evaluation = governance_enforcement::evaluate_registry(&root, &registry);
            let enforcement_path = governance_enforcement_path(&root);
            let coverage_path = governance_enforcement_coverage_path(&root);
            let metrics_path = governance_enforcement_metrics_path(&root);
            let coverage_payload = governance_enforcement_coverage_payload(&registry);
            if let Some(parent) = enforcement_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
            }
            fs::write(
                &enforcement_path,
                serde_json::to_string_pretty(&evaluation)
                    .map_err(|e| format!("encode governance enforcement report failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", enforcement_path.display()))?;
            fs::write(
                &coverage_path,
                serde_json::to_string_pretty(&coverage_payload)
                    .map_err(|e| format!("encode governance enforcement coverage failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", coverage_path.display()))?;
            let mut high = 0usize;
            let mut medium = 0usize;
            let mut low = 0usize;
            for violation in &evaluation.violations {
                match violation.severity {
                    governance_enforcement::GovernanceSeverity::High => high += 1,
                    governance_enforcement::GovernanceSeverity::Medium => medium += 1,
                    governance_enforcement::GovernanceSeverity::Low => low += 1,
                }
            }
            let metrics_payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_enforcement_metrics",
                "rule_count": evaluation.rule_count,
                "violation_count": evaluation.violations.len(),
                "status": evaluation.status,
                "violation_breakdown": {
                    "high": high,
                    "medium": medium,
                    "low": low
                }
            });
            fs::write(
                &metrics_path,
                serde_json::to_string_pretty(&metrics_payload)
                    .map_err(|e| format!("encode governance enforcement metrics failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", metrics_path.display()))?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_check",
                "status": evaluation.status,
                "evaluation": evaluation,
                "coverage": coverage_payload,
                "metrics": metrics_payload,
                "artifacts": {
                    "governance_enforcement": enforcement_path.strip_prefix(&root).unwrap_or(&enforcement_path).display().to_string(),
                    "governance_enforcement_coverage": coverage_path.strip_prefix(&root).unwrap_or(&coverage_path).display().to_string(),
                    "governance_enforcement_metrics": metrics_path.strip_prefix(&root).unwrap_or(&metrics_path).display().to_string(),
                }
            });
            let rendered = emit_payload(format, out, &payload)?;
            let code = if payload["status"] == "ok" { 0 } else { 1 };
            Ok((rendered, code))
        }
        GovernanceCommand::Rules {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let mut registry = governance_enforcement::load_registry(&root)?;
            registry.rules.sort_by(|a, b| a.id.cmp(&b.id));
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_rules",
                "status": "ok",
                "registry": registry,
            });
            let rendered = emit_payload(format, out, &payload)?;
            Ok((rendered, 0))
        }
        GovernanceCommand::Report {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let docs_errors = governance_docs_validation_errors(&root);
            let contributor_errors = contributor_guideline_validation_errors(&root);
            let adr_index = governance_adr_index_payload(&root)?;
            let enforcement_registry = governance_enforcement::load_registry(&root)?;
            let enforcement =
                governance_enforcement::evaluate_registry(&root, &enforcement_registry);
            let (checks_rendered, checks_code) =
                crate::run_checks_automation_boundaries(crate::AutomationBoundariesOptions {
                    repo_root: Some(root.clone()),
                    format: FormatArg::Json,
                    out: None,
                })?;
            let checks_report: serde_json::Value = serde_json::from_str(&checks_rendered)
                .map_err(|e| format!("parse checks automation report failed: {e}"))?;
            let tutorials_purity = checks_report["checks"]
                .as_array()
                .and_then(|rows| {
                    rows.iter().find(|row| {
                        row["id"].as_str() == Some("automation.tutorials.forbidden-patterns")
                    })
                })
                .cloned()
                .unwrap_or_else(|| serde_json::json!({"id":"automation.tutorials.forbidden-patterns","status":"missing"}));
            let clients_purity = checks_report["checks"]
                .as_array()
                .and_then(|rows| {
                    rows.iter().find(|row| {
                        row["id"].as_str() == Some("automation.clients.forbidden-patterns")
                    })
                })
                .cloned()
                .unwrap_or_else(|| serde_json::json!({"id":"automation.clients.forbidden-patterns","status":"missing"}));
            let directory_purity = checks_report["checks"]
                .as_array()
                .and_then(|rows| {
                    rows.iter().find(|row| {
                        row["id"].as_str() == Some("automation.ops.directory-purity")
                    })
                })
                .cloned()
                .unwrap_or_else(|| serde_json::json!({"id":"automation.ops.directory-purity","status":"missing"}));
            let packages_boundary = checks_report["checks"]
                .as_array()
                .and_then(|rows| {
                    rows.iter().find(|row| {
                        row["id"].as_str() == Some("automation.packages.boundary-compliance")
                    })
                })
                .cloned()
                .unwrap_or_else(|| serde_json::json!({"id":"automation.packages.boundary-compliance","status":"missing"}));
            let repo_purity = serde_json::json!({
                "status": if checks_code == 0 { "ok" } else { "failed" },
                "automation_boundary_violations": checks_report["violations"].as_array().map_or(0, |rows| rows.len()),
                "checks_exit_code": checks_code,
            });
            let status = if docs_errors.is_empty()
                && contributor_errors.is_empty()
                && enforcement.status == "ok"
                && checks_code == 0
            {
                "ok"
            } else {
                "failed"
            };
            let report = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_health_report",
                "status": status,
                "summary": {
                    "required_governance_documents": required_governance_docs().len(),
                    "missing_governance_documents": docs_errors.len(),
                    "contributor_guideline_findings": contributor_errors.len(),
                    "enforcement_rule_count": enforcement.rule_count,
                    "enforcement_violation_count": enforcement.violations.len(),
                    "adr_count": adr_index["count"].clone(),
                    "automation_boundary_violations": checks_report["violations"].as_array().map_or(0, |rows| rows.len()),
                },
                "governance_docs_validation": {
                    "status": if docs_errors.is_empty() { "ok" } else { "failed" },
                    "errors": docs_errors,
                },
                "contributor_guidelines_validation": {
                    "status": if contributor_errors.is_empty() { "ok" } else { "failed" },
                    "errors": contributor_errors,
                },
                "enforcement": enforcement,
                "adr_index": adr_index,
                "sections": {
                    "Automation purity": checks_report,
                    "Tutorials purity": tutorials_purity,
                    "Clients tooling purity": clients_purity,
                    "Packages boundary compliance": packages_boundary,
                    "Directory Purity": directory_purity,
                    "Repo purity": repo_purity,
                }
            });
            let report_path = governance_health_report_path(&root);
            write_pretty_json(&report_path, &report)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_report",
                "status": report["status"].clone(),
                "report_path": report_path.strip_prefix(&root).unwrap_or(&report_path).display().to_string(),
                "summary": report["summary"].clone(),
            });
            let rendered = emit_payload(format, out, &payload)?;
            let exit_code = if payload["status"] == "ok" { 0 } else { 1 };
            Ok((rendered, exit_code))
        }
        GovernanceCommand::DoctrineReport {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let (checks_rendered, checks_code) =
                crate::run_checks_automation_boundaries(crate::AutomationBoundariesOptions {
                    repo_root: Some(root.clone()),
                    format: FormatArg::Json,
                    out: None,
                })?;
            let (contracts_rendered, contracts_code) =
                crate::run_contract_automation_boundaries(crate::AutomationBoundariesOptions {
                    repo_root: Some(root.clone()),
                    format: FormatArg::Json,
                    out: None,
                })?;

            let checks_report: serde_json::Value = serde_json::from_str(&checks_rendered)
                .map_err(|e| format!("parse checks automation report failed: {e}"))?;
            let contracts_report: serde_json::Value = serde_json::from_str(&contracts_rendered)
                .map_err(|e| format!("parse contract automation report failed: {e}"))?;

            let status = if checks_code == 0 && contracts_code == 0 {
                "ok"
            } else {
                "failed"
            };
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "doctrine_compliance_report",
                "status": status,
                "summary": {
                    "checks_exit_code": checks_code,
                    "contracts_exit_code": contracts_code,
                    "checks_violations": checks_report["violations"].as_array().map_or(0, |rows| rows.len()),
                    "contracts_violations": contracts_report["violations"].as_array().map_or(0, |rows| rows.len()),
                },
                "checks_automation_boundaries": checks_report,
                "contract_automation_boundaries": contracts_report,
            });
            let report_path = root.join("artifacts/governance/doctrine-compliance-report.json");
            write_pretty_json(&report_path, &payload)?;
            let envelope = serde_json::json!({
                "schema_version": 1,
                "kind": "doctrine_compliance",
                "status": status,
                "report_path": report_path.strip_prefix(&root).unwrap_or(&report_path).display().to_string(),
                "summary": payload["summary"].clone(),
            });
            let rendered = emit_payload(format, out, &envelope)?;
            Ok((rendered, if status == "ok" { 0 } else { 1 }))
        }
        GovernanceCommand::Validate {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let objects = collect_governance_objects(&root)?;
            let validation = validate_governance_objects(&root, &objects);
            let checks_inventory = validate_checks_inventory(&root)?;
            let (graph_path, summary_path) = governance_summary_paths(&root);
            let coverage_path = governance_coverage_path(&root);
            let orphan_path = governance_orphan_report_path(&root);
            let index_path = governance_index_path(&root);
            let contract_coverage_path = governance_contract_coverage_path(&root);
            let lane_coverage_path = governance_lane_coverage_path(&root);
            let orphan_checks_path = governance_orphan_checks_path(&root);
            let policy_surface_path = governance_policy_surface_path(&root);
            let drift_path = governance_drift_path(&root);
            let enforcement_path = governance_enforcement_path(&root);
            let enforcement_coverage_path = governance_enforcement_coverage_path(&root);
            let enforcement_metrics_path = governance_enforcement_metrics_path(&root);
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
            let enforcement_registry = governance_enforcement::load_registry(&root)?;
            let enforcement_result =
                governance_enforcement::evaluate_registry(&root, &enforcement_registry);
            let enforcement_coverage =
                governance_enforcement_coverage_payload(&enforcement_registry);
            fs::write(
                &enforcement_path,
                serde_json::to_string_pretty(&enforcement_result)
                    .map_err(|e| format!("encode governance enforcement report failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", enforcement_path.display()))?;
            fs::write(
                &enforcement_coverage_path,
                serde_json::to_string_pretty(&enforcement_coverage)
                    .map_err(|e| format!("encode governance enforcement coverage failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", enforcement_coverage_path.display()))?;
            let enforcement_metrics = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_enforcement_metrics",
                "rule_count": enforcement_result.rule_count,
                "violation_count": enforcement_result.violations.len(),
                "status": enforcement_result.status,
            });
            fs::write(
                &enforcement_metrics_path,
                serde_json::to_string_pretty(&enforcement_metrics)
                    .map_err(|e| format!("encode governance enforcement metrics failed: {e}"))?,
            )
            .map_err(|e| format!("write {} failed: {e}", enforcement_metrics_path.display()))?;
            let mut governance_errors = validation.errors;
            let governance_docs_errors = governance_docs_validation_errors(&root);
            let contributor_guideline_errors = contributor_guideline_validation_errors(&root);
            governance_errors.extend(governance_docs_errors.clone());
            governance_errors.extend(contributor_guideline_errors.clone());
            governance_errors.extend(
                checks_inventory["errors"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_str().map(ToString::to_string)),
            );
            governance_errors.extend(validate_governance_registry(&root)?);
            governance_errors.extend(
                enforcement_result
                    .violations
                    .iter()
                    .map(|v| format!("[{}] {}", v.rule_id, v.message)),
            );

            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_validate",
                "status": if governance_errors.is_empty()
                    && checks_inventory["errors"].as_array().is_none_or(|rows| rows.is_empty()) {
                    "ok"
                } else {
                    "failed"
                },
                "objects": collect_governance_objects(&root)?,
                "errors": governance_errors,
                "checks_inventory": checks_inventory,
                "governance_docs_validation": {
                    "status": if governance_docs_errors.is_empty() { "ok" } else { "failed" },
                    "errors": governance_docs_errors,
                    "required_documents": required_governance_docs(),
                },
                "contributor_guidelines_validation": {
                    "status": if contributor_guideline_errors.is_empty() { "ok" } else { "failed" },
                    "errors": contributor_guideline_errors,
                },
                "enforcement": enforcement_result,
                "enforcement_coverage": enforcement_coverage,
                "enforcement_metrics": enforcement_metrics,
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
                    "governance_enforcement": enforcement_path.strip_prefix(&root).unwrap_or(&enforcement_path).display().to_string(),
                    "governance_enforcement_coverage": enforcement_coverage_path.strip_prefix(&root).unwrap_or(&enforcement_coverage_path).display().to_string(),
                    "governance_enforcement_metrics": enforcement_metrics_path.strip_prefix(&root).unwrap_or(&enforcement_metrics_path).display().to_string(),
                    "checks_inventory": checks_inventory_path(&root).strip_prefix(&root).unwrap_or(&checks_inventory_path(&root)).display().to_string(),
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
            GovernanceExceptionsCommand::List {
                repo_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let registry_path = exceptions_registry_path(&root);
                let registry: ExceptionsRegistry = read_yaml_file(&registry_path)?;
                let today = current_utc_day()?;
                let rows = registry
                    .exceptions
                    .into_iter()
                    .map(|item| {
                        let expires = date_days(&item.expires_at).ok();
                        serde_json::json!({
                            "id": item.id,
                            "scope": {"kind": item.scope.kind, "id": item.scope.id},
                            "severity": item.severity,
                            "owner": item.owner,
                            "created_at": item.created_at,
                            "expires_at": item.expires_at,
                            "days_to_expiry": expires.map(|value| value - today),
                            "reason": item.reason,
                        })
                    })
                    .collect::<Vec<_>>();
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "governance_exceptions_list",
                    "status": "ok",
                    "count": rows.len(),
                    "exceptions": rows,
                    "registry": registry_path.strip_prefix(&root).unwrap_or(&registry_path).display().to_string(),
                });
                let rendered = emit_payload(format, out, &payload)?;
                Ok((rendered, 0))
            }
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
                let no_exception_zones: BTreeSet<String> =
                    registry.policy.no_exception_zones.iter().cloned().collect();
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
                        errors.push(format!(
                            "{} has invalid created_at `{}`",
                            item.id, item.created_at
                        ));
                    }
                    if !is_iso_date(&item.expires_at) {
                        errors.push(format!(
                            "{} has invalid expires_at `{}`",
                            item.id, item.expires_at
                        ));
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
                        other => {
                            errors.push(format!("{} uses invalid scope.kind `{}`", item.id, other))
                        }
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
                        "sources": ["configs/sources/governance/governance/exceptions.yaml"]
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
                validate_named_report(
                    &root,
                    "exceptions-expiry-warning.schema.json",
                    &warning_report,
                )?;
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
                            "configs/sources/governance/governance/exceptions.yaml",
                            "configs/sources/governance/governance/exceptions-archive.yaml"
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
                let release_manifest_path = root.join("ops/release/evidence/manifest.json");
                let rel_exc_001 = if release_manifest_path.exists() {
                    let manifest: serde_json::Value =
                        serde_json::from_str(&fs::read_to_string(&release_manifest_path).map_err(
                            |err| format!("read {} failed: {err}", release_manifest_path.display()),
                        )?)
                        .map_err(|err| {
                            format!("parse {} failed: {err}", release_manifest_path.display())
                        })?;
                    manifest
                        .get("governance_assets")
                        .and_then(|value| value.get("exceptions_registry"))
                        .and_then(|value| value.get("path"))
                        .and_then(serde_json::Value::as_str)
                        == Some("configs/sources/governance/governance/exceptions.yaml")
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
                            "configs/sources/governance/governance/exceptions.yaml",
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
                let values_schema =
                    read_json_value(&root.join("ops/k8s/charts/bijux-atlas/values.schema.json"))?;
                let env_schema = read_json_value(&root.join("configs/schemas/contracts/env.schema.json"))?;
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
                            "configs/sources/governance/governance/deprecations.yaml",
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
                validate_named_report(
                    &root,
                    "compat-warnings.schema.json",
                    &compat_warnings_report,
                )?;
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
                        errors.push(format!(
                            "compatibility policy missing rule set `{rule_set}`"
                        ));
                    }
                    if !compatibility.deprecation_window_days.contains_key(rule_set) {
                        errors.push(format!(
                            "compatibility policy missing deprecation window for `{rule_set}`"
                        ));
                    }
                }
                for (name, rule_set) in &compatibility.compatibility_rules {
                    if rule_set.breaking_changes.is_empty() {
                        errors.push(format!(
                            "compatibility rule set `{name}` missing breaking_changes"
                        ));
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
                            let old_supported =
                                env_allowlist_contains(&env_schema, &entry.old_name);
                            let new_supported =
                                env_allowlist_contains(&env_schema, &entry.new_name);
                            let docs_updated = !entry.new_name.is_empty();
                            checks.insert(
                                "allowlist_support".to_string(),
                                old_supported && new_supported,
                            );
                            checks.insert("docs_updated".to_string(), docs_updated);
                            if !old_supported || !new_supported {
                                errors.push(format!(
                                    "{} env rename requires old and new keys in env schema",
                                    entry.id
                                ));
                            }
                        }
                        "chart-value" => {
                            let old_supported =
                                schema_supports_path(&values_schema, &entry.old_name);
                            let new_supported =
                                schema_supports_path(&values_schema, &entry.new_name);
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
                            errors
                                .push(format!("{} has unsupported surface `{}`", entry.id, other));
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
                            "configs/sources/governance/governance/compatibility.yaml",
                            "configs/sources/governance/governance/deprecations.yaml",
                            "configs/schemas/contracts/env.schema.json",
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
                    let migration_path =
                        root.join(format!("docs/reference/reports/migrations/{}.md", entry.id));
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
                                file.contains("prod") || file.contains("profile-baseline.yaml")
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
                            .is_some_and(|file| {
                                file.ends_with("/ci.yaml") || file.ends_with("ci.yaml")
                            })
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
                        "breaking changes exist but ops/release/notes/breaking.json has no entries"
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
                            "configs/sources/governance/governance/deprecations.yaml",
                            "artifacts/governance/compat-warnings.json",
                            "docs/redirects.json",
                            "ops/k8s/charts/bijux-atlas/Chart.yaml",
                            "ops/release/notes/breaking.json"
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
        GovernanceCommand::Adr { command } => match command {
            GovernanceAdrCommand::Index {
                repo_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let index_path = governance_adr_index_path(&root);
                let payload = governance_adr_index_payload(&root)?;
                write_pretty_json(&index_path, &payload)?;
                let output = serde_json::json!({
                    "schema_version": 1,
                    "kind": "governance_adr_index",
                    "status": "ok",
                    "count": payload["count"].clone(),
                    "entries": payload["entries"].clone(),
                    "artifact": index_path.strip_prefix(&root).unwrap_or(&index_path).display().to_string(),
                });
                let rendered = emit_payload(format, out, &output)?;
                Ok((rendered, 0))
            }
        },
        GovernanceCommand::Doctor {
            repo_root,
            format,
            out,
        } => {
            let root = resolve_repo_root(repo_root)?;
            let registry_path = exceptions_registry_path(&root);
            let registry_text = fs::read_to_string(&registry_path)
                .map_err(|err| format!("read {} failed: {err}", registry_path.display()))?;
            let exceptions: ExceptionsRegistry = serde_yaml::from_str(&registry_text)
                .map_err(|err| format!("parse {} failed: {err}", registry_path.display()))?;
            let deprecations = load_deprecations_registry(&root)?;
            let breaking_report = if breaking_changes_path(&root).exists() {
                read_json_value(&breaking_changes_path(&root))?
            } else {
                serde_json::json!({"rows":[]})
            };
            let release_manifest_path = root.join("ops/release/evidence/manifest.json");
            let today = current_utc_day()?;

            let active_exceptions = exceptions
                .exceptions
                .iter()
                .filter(|item| {
                    date_days(&item.expires_at)
                        .map(|day| day >= today)
                        .unwrap_or(false)
                })
                .map(|item| {
                    serde_json::json!({
                        "kind": "exception",
                        "id": item.id,
                        "scope": format!("{}:{}", item.scope.kind, item.scope.id),
                        "expires_at": item.expires_at
                    })
                })
                .collect::<Vec<_>>();

            let active_deprecations = deprecations
                .deprecations
                .iter()
                .filter(|item| {
                    date_days(&item.removal_target)
                        .map(|day| day >= today)
                        .unwrap_or(false)
                })
                .map(|item| {
                    serde_json::json!({
                        "kind": "deprecation",
                        "id": item.id,
                        "surface": item.surface,
                        "old_name": item.old_name,
                        "new_name": item.new_name,
                        "removal_target": item.removal_target
                    })
                })
                .collect::<Vec<_>>();

            let upcoming_removals = deprecations
                .deprecations
                .iter()
                .filter_map(|item| {
                    let removal = date_days(&item.removal_target).ok()?;
                    let days = removal - today;
                    (0..30).contains(&days).then(|| {
                        serde_json::json!({
                            "kind": "upcoming-removal",
                            "id": item.id,
                            "surface": item.surface,
                            "removal_target": item.removal_target,
                            "days_until_removal": days
                        })
                    })
                })
                .collect::<Vec<_>>();

            let mut rows = Vec::new();
            rows.extend(active_deprecations.iter().cloned());
            rows.extend(upcoming_removals.iter().cloned());
            rows.extend(active_exceptions.iter().cloned());

            let delta_inputs = serde_json::json!({
                "schema_version": 1,
                "generated_from": {
                    "breaking_changes": "artifacts/governance/breaking-changes.json",
                    "deprecations": "configs/sources/governance/governance/deprecations.yaml"
                },
                "breaking_changes": breaking_report.get("rows").cloned().unwrap_or_else(|| serde_json::json!([])),
                "deprecations": active_deprecations
            });
            let delta_inputs_path = institutional_delta_inputs_path(&root);
            let delta_inputs_schema = read_json_value(
                &root.join("configs/schemas/contracts/reports/institutional-delta-inputs.schema.json"),
            )?;
            if delta_inputs_schema
                .get("properties")
                .and_then(|value| value.get("schema_version"))
                .and_then(|value| value.get("const"))
                .and_then(serde_json::Value::as_u64)
                != delta_inputs
                    .get("schema_version")
                    .and_then(serde_json::Value::as_u64)
            {
                return Err("institutional delta inputs schema version mismatch".to_string());
            }
            if delta_inputs
                .get("breaking_changes")
                .and_then(serde_json::Value::as_array)
                .is_none()
                || delta_inputs
                    .get("deprecations")
                    .and_then(serde_json::Value::as_array)
                    .is_none()
            {
                return Err("institutional delta inputs must declare breaking_changes and deprecations arrays".to_string());
            }
            write_pretty_json(&delta_inputs_path, &delta_inputs)?;
            let delta_markdown = render_institutional_delta_markdown(&delta_inputs);
            let delta_markdown_path = institutional_delta_markdown_path(&root);
            write_text(&delta_markdown_path, &delta_markdown)?;

            let rel_gov_001 = if release_manifest_path.exists() {
                let manifest = read_json_value(&release_manifest_path)?;
                manifest
                    .get("governance_assets")
                    .and_then(|value| value.get("governance_doctor"))
                    .and_then(|value| value.get("path"))
                    .and_then(serde_json::Value::as_str)
                    == Some("artifacts/governance/governance-doctor.json")
            } else {
                true
            };
            let rel_gov_002 = if release_manifest_path.exists() {
                delta_markdown_path.exists()
            } else {
                true
            };

            let report = serde_json::json!({
                "report_id": "governance-doctor",
                "version": 1,
                "inputs": {
                    "generator": "bijux-dev-atlas governance doctor",
                    "sources": [
                        "configs/sources/governance/governance/exceptions.yaml",
                        "configs/sources/governance/governance/deprecations.yaml",
                        "artifacts/governance/breaking-changes.json"
                    ]
                },
                "status": "ok",
                "summary": {
                    "active_deprecations": active_deprecations.len(),
                    "upcoming_removals": upcoming_removals.len(),
                    "active_exceptions": active_exceptions.len()
                },
                "rows": rows,
                "contracts": {
                    "GOV-DOC-001": true,
                    "REL-GOV-001": rel_gov_001,
                    "REL-GOV-002": rel_gov_002
                },
                "errors": []
            });
            validate_named_report(&root, "governance-doctor.schema.json", &report)?;
            let report_path = governance_doctor_path(&root);
            write_pretty_json(&report_path, &report)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "governance_doctor",
                "status": "ok",
                "report_path": report_path.strip_prefix(&root).unwrap_or(&report_path).display().to_string(),
                "institutional_delta_path": delta_markdown_path.strip_prefix(&root).unwrap_or(&delta_markdown_path).display().to_string(),
                "contracts": report["contracts"].clone(),
                "errors": []
            });
            let rendered = emit_payload(format, out, &payload)?;
            Ok((rendered, 0))
        }
    }
}

pub(crate) fn run_registry_command(quiet: bool, command: RegistryCommand) -> i32 {
    let result = (|| -> Result<(String, i32), String> {
        match command {
            RegistryCommand::Status {
                repo_root,
                format,
                missing,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let payload = registry_status_payload(&root)?;
                validate_named_report(&root, "registry-status.schema.json", &payload)?;
                write_pretty_json(&registry_status_path(&root), &payload)?;
                fs::write(
                    registry_status_markdown_path(&root),
                    registry_status_markdown(&payload),
                )
                .map_err(|err| {
                    format!(
                        "write {} failed: {err}",
                        registry_status_markdown_path(&root).display()
                    )
                })?;
                let filtered_rows = if let Some(filter) = missing {
                    let needle = registry_missing_field_name(filter);
                    payload["rows"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|row| {
                            row["missing"].as_array().is_some_and(|items| {
                                items.iter().any(|item| item.as_str() == Some(needle))
                            })
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    payload["rows"].as_array().cloned().unwrap_or_default()
                };
                let response = serde_json::json!({
                    "schema_version": 1,
                    "kind": "registry_status",
                    "status": payload["status"].clone(),
                    "summary": payload["summary"].clone(),
                    "filter": missing.map(registry_missing_field_name),
                    "rows": filtered_rows,
                    "report_path": registry_status_path(&root).strip_prefix(&root).unwrap_or(&registry_status_path(&root)).display().to_string()
                });
                let rendered = emit_payload(format, out, &response)?;
                let exit_code = if response["status"] == "ok" { 0 } else { 1 };
                Ok((rendered, exit_code))
            }
            RegistryCommand::Doctor {
                repo_root,
                fix_suggestions,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let registry_status = registry_status_payload(&root)?;
                validate_named_report(&root, "registry-status.schema.json", &registry_status)?;
                write_pretty_json(&registry_status_path(&root), &registry_status)?;
                fs::write(
                    registry_status_markdown_path(&root),
                    registry_status_markdown(&registry_status),
                )
                .map_err(|err| {
                    format!(
                        "write {} failed: {err}",
                        registry_status_markdown_path(&root).display()
                    )
                })?;
                let work_remaining = serde_json::json!({
                    "schema_version": 1,
                    "kind": "registry_work_remaining",
                    "status": registry_status["status"].clone(),
                    "rows": registry_status["work_remaining"].clone()
                });
                write_pretty_json(&registry_work_remaining_path(&root), &work_remaining)?;
                let suggestions = if fix_suggestions {
                    registry_status["work_remaining"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|row| {
                            let missing = row["missing"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(serde_json::Value::as_str)
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!(
                                "{} {}: fill {}",
                                row["kind"].as_str().unwrap_or_default(),
                                row["id"].as_str().unwrap_or_default(),
                                missing
                            )
                        })
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "registry_doctor",
                    "status": registry_status["status"].clone(),
                    "summary": registry_status["summary"].clone(),
                    "report_path": registry_status_path(&root).strip_prefix(&root).unwrap_or(&registry_status_path(&root)).display().to_string(),
                    "work_remaining_path": registry_work_remaining_path(&root).strip_prefix(&root).unwrap_or(&registry_work_remaining_path(&root)).display().to_string(),
                    "fix_suggestions": suggestions,
                    "errors": registry_status["errors"].clone()
                });
                let rendered = emit_payload(format, out, &payload)?;
                let exit_code = if payload["status"] == "ok" { 0 } else { 1 };
                Ok((rendered, exit_code))
            }
        }
    })();

    match result {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 {
                    let _ = writeln!(io::stdout(), "{rendered}");
                } else {
                    let _ = writeln!(io::stderr(), "{rendered}");
                }
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas registry failed: {err}");
            1
        }
    }
}
