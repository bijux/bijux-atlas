use super::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiPolicyRegistry {
    schema_version: u64,
    pub(super) entries: Vec<CiPolicyEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiPolicyEntry {
    pub(super) policy_id: String,
    pub(super) workflow: String,
    pub(super) job: String,
    pub(super) step: String,
    pub(super) classification: String,
    owner: String,
    pub(super) status: String,
    #[serde(default)]
    pub(super) control_plane_command: String,
    #[serde(default)]
    pub(super) authoritative_implementation: String,
    pub(super) replacement_plan: String,
    pub(super) docs: String,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CiPolicyExceptions {
    schema_version: u64,
    exceptions: Vec<CiPolicyException>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CiPolicyException {
    policy_id: String,
    workflow: String,
    job: String,
    step: String,
    owner: String,
    reason: String,
    expires_on: String,
    renewal_reference: String,
    governance_tier: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiLaneSurfaceRegistry {
    schema_version: u64,
    pub(super) lanes: Vec<CiLaneSurfaceEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiLaneSurfaceEntry {
    pub(super) lane: String,
    pub(super) workflow: String,
    pub(super) commands: Vec<CiLaneCommandEntry>,
    pub(super) reports: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiLaneCommandEntry {
    pub(super) id: String,
    pub(super) kind: String,
    command: String,
    #[serde(default)]
    pub(super) suite: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WorkflowStepPatterns {
    schema_version: u64,
    patterns: Vec<WorkflowStepPattern>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WorkflowStepPattern {
    pattern_id: String,
    step_kind: String,
    classification: String,
    match_mode: String,
    needle: String,
    allowed: bool,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WorkflowAllowlist {
    schema_version: u64,
    entries: Vec<WorkflowAllowlistEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WorkflowAllowlistEntry {
    workflow: String,
    job: String,
    step: String,
    owner: String,
    reason: String,
    expires_on: String,
    renewal_reference: String,
}

#[derive(Debug, Serialize)]
pub(super) struct WorkflowLintRow {
    pub(super) workflow: String,
    pub(super) job: String,
    pub(super) step: String,
    step_kind: String,
    pub(super) classification: String,
    pub(super) allowed: bool,
    matched_pattern: String,
    pub(super) registry_policy_id: String,
    allowlist_expires_on: String,
}

pub(super) fn load_ci_policy_registry(repo_root: &Path) -> Result<CiPolicyRegistry, String> {
    let path = repo_root.join("configs/sources/repository/ci/policy-outside-control-plane.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_ci_policy_exceptions(repo_root: &Path) -> Result<CiPolicyExceptions, String> {
    let path = repo_root.join("configs/sources/repository/ci/policy-exceptions.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

pub(super) fn load_ci_lane_surface(repo_root: &Path) -> Result<CiLaneSurfaceRegistry, String> {
    let path = repo_root.join("configs/sources/repository/ci/lane-surface.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiLaneRegistry {
    pub(super) schema_version: u64,
    #[serde(default)]
    pub(super) concurrency_classes: std::collections::BTreeMap<String, CiConcurrencyClass>,
    #[serde(default)]
    pub(super) timeout_classes: std::collections::BTreeMap<String, CiTimeoutClass>,
    pub(super) lanes: Vec<CiLaneRegistryEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiConcurrencyClass {
    pub(super) max_parallelism: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiTimeoutClass {
    pub(super) max_minutes: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct CiLaneRegistryEntry {
    pub(super) id: String,
    pub(super) description: String,
    pub(super) mode: String,
    #[serde(default)]
    pub(super) aliases: Vec<String>,
    pub(super) required_env: Vec<String>,
    pub(super) artifacts_expected: Vec<String>,
    pub(super) evidence_bundle: String,
    pub(super) timeout_class: String,
    pub(super) concurrency_class: String,
    pub(super) command: String,
}

pub(super) fn load_ci_lanes_registry(repo_root: &Path) -> Result<CiLaneRegistry, String> {
    let path = repo_root.join("configs/sources/repository/ci/lanes.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_workflow_step_patterns(repo_root: &Path) -> Result<WorkflowStepPatterns, String> {
    let path = repo_root.join("configs/sources/repository/ci/workflow-step-patterns.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_workflow_allowlist(repo_root: &Path) -> Result<WorkflowAllowlist, String> {
    let path = repo_root.join("configs/sources/repository/ci/workflow-allowlist.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn workflow_pattern_matches(pattern: &WorkflowStepPattern, value: &str) -> bool {
    match pattern.match_mode.as_str() {
        "contains" => value.contains(&pattern.needle),
        "starts_with" => value.starts_with(&pattern.needle),
        "equals" => value == pattern.needle,
        _ => false,
    }
}

fn normalized_run_body(run: &str) -> String {
    run.lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && *line != "set -euo pipefail"
                && !line.starts_with('#')
                && *line != "then"
                && *line != "else"
                && *line != "fi"
                && *line != "do"
                && *line != "done"
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn workflow_step_rows(repo_root: &Path) -> Result<Vec<WorkflowLintRow>, String> {
    let patterns = load_workflow_step_patterns(repo_root)?;
    let allowlist = load_workflow_allowlist(repo_root)?;
    let registry = load_ci_policy_registry(repo_root)?;
    let mut rows = Vec::<WorkflowLintRow>::new();
    for workflow in walk_files_local(&repo_root.join(".github/workflows"))
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("yml"))
    {
        let workflow_rel = workflow
            .strip_prefix(repo_root)
            .unwrap_or(&workflow)
            .display()
            .to_string();
        let value: serde_yaml::Value = serde_yaml::from_str(
            &fs::read_to_string(&workflow)
                .map_err(|err| format!("read {} failed: {err}", workflow.display()))?,
        )
        .map_err(|err| format!("parse {} failed: {err}", workflow.display()))?;
        let jobs = value
            .get("jobs")
            .and_then(serde_yaml::Value::as_mapping)
            .cloned()
            .unwrap_or_default();
        for (job_key, job_value) in jobs {
            let Some(job_name) = job_key.as_str() else {
                continue;
            };
            let steps = job_value
                .get("steps")
                .and_then(serde_yaml::Value::as_sequence)
                .cloned()
                .unwrap_or_default();
            for step in steps {
                let step_name = step
                    .get("name")
                    .and_then(serde_yaml::Value::as_str)
                    .or_else(|| step.get("id").and_then(serde_yaml::Value::as_str))
                    .or_else(|| step.get("uses").and_then(serde_yaml::Value::as_str))
                    .unwrap_or("unnamed-step")
                    .to_string();
                let (step_kind, step_value) =
                    if let Some(uses) = step.get("uses").and_then(serde_yaml::Value::as_str) {
                        ("uses".to_string(), uses.to_string())
                    } else if let Some(run) = step.get("run").and_then(serde_yaml::Value::as_str) {
                        ("run".to_string(), normalized_run_body(run))
                    } else {
                        ("invalid-step".to_string(), String::new())
                    };
                let matched_pattern = patterns
                    .patterns
                    .iter()
                    .find(|pattern| {
                        pattern.step_kind == step_kind
                            && workflow_pattern_matches(pattern, &step_value)
                    })
                    .cloned();
                let allowlist_entry = allowlist.entries.iter().find(|entry| {
                    entry.workflow == workflow_rel
                        && entry.job == job_name
                        && entry.step == step_name
                });
                let registry_entry = registry.entries.iter().find(|entry| {
                    entry.workflow == workflow_rel
                        && entry.job == job_name
                        && entry.step == step_name
                });
                let classification = matched_pattern
                    .as_ref()
                    .map(|pattern| pattern.classification.clone())
                    .or_else(|| registry_entry.map(|entry| entry.classification.clone()))
                    .unwrap_or_else(|| "unclassified".to_string());
                let allowed = matched_pattern
                    .as_ref()
                    .is_some_and(|pattern| pattern.allowed)
                    || allowlist_entry.is_some();
                rows.push(WorkflowLintRow {
                    workflow: workflow_rel.clone(),
                    job: job_name.to_string(),
                    step: step_name,
                    step_kind,
                    classification,
                    allowed,
                    matched_pattern: matched_pattern
                        .map(|pattern| pattern.pattern_id)
                        .unwrap_or_default(),
                    registry_policy_id: registry_entry
                        .map(|entry| entry.policy_id.clone())
                        .unwrap_or_default(),
                    allowlist_expires_on: allowlist_entry
                        .map(|entry| entry.expires_on.clone())
                        .unwrap_or_default(),
                });
            }
        }
    }
    rows.sort_by(|a, b| {
        a.workflow
            .cmp(&b.workflow)
            .then_with(|| a.job.cmp(&b.job))
            .then_with(|| a.step.cmp(&b.step))
    });
    Ok(rows)
}

fn current_utc_date_string() -> Result<String, String> {
    let output = ProcessCommand::new("date")
        .args(["-u", "+%F"])
        .output()
        .map_err(|err| format!("date -u failed: {err}"))?;
    if !output.status.success() {
        return Err("date -u +%F failed".to_string());
    }
    let rendered = String::from_utf8(output.stdout).map_err(|err| err.to_string())?;
    Ok(rendered.trim().to_string())
}

pub(super) fn ci_exception_is_expired(raw: &str) -> Result<bool, String> {
    if raw.len() != 10
        || !raw.chars().enumerate().all(|(idx, ch)| match idx {
            4 | 7 => ch == '-',
            _ => ch.is_ascii_digit(),
        })
    {
        return Err(format!("invalid expiry `{raw}`"));
    }
    let today = current_utc_date_string()?;
    Ok(raw < today.as_str())
}

pub(super) struct CiRegistryPolicyDrift {
    pub unplanned: Vec<CiPolicyEntry>,
    pub uniqueness_errors: Vec<String>,
    pub docs_errors: Vec<String>,
    pub exception_errors: Vec<String>,
}

pub(super) fn ci_registry_unplanned_entries(
    repo_root: &Path,
) -> Result<CiRegistryPolicyDrift, String> {
    let registry = load_ci_policy_registry(repo_root)?;
    let exceptions = load_ci_policy_exceptions(repo_root)?;
    let mut unplanned = Vec::new();
    let mut uniqueness_errors = Vec::new();
    let mut docs_errors = Vec::new();
    let mut exception_errors = Vec::new();
    let mut seen_policy_ids = std::collections::BTreeSet::<String>::new();
    for entry in &registry.entries {
        if entry.classification == "policy"
            && entry.status != "atlas"
            && entry.status != "planned"
            && entry.status != "exception"
        {
            unplanned.push(entry.clone());
        }
        if entry.status == "atlas" && entry.control_plane_command.trim().is_empty() {
            unplanned.push(entry.clone());
        }
        if !seen_policy_ids.insert(entry.policy_id.clone()) {
            uniqueness_errors.push(format!(
                "policy registry contains duplicate policy_id `{}`",
                entry.policy_id
            ));
        }
        if entry.authoritative_implementation.trim().is_empty() {
            uniqueness_errors.push(format!(
                "policy registry entry `{}` is missing authoritative_implementation",
                entry.policy_id
            ));
        }
        if entry.docs.trim().is_empty() || !repo_root.join(&entry.docs).exists() {
            docs_errors.push(format!(
                "policy `{}` references missing docs `{}`",
                entry.policy_id, entry.docs
            ));
        }
        if entry.replacement_plan.trim().is_empty() {
            unplanned.push(entry.clone());
        }
    }
    for exception in exceptions.exceptions {
        if exception.governance_tier != "temporary"
            && exception.governance_tier != "governance-approved"
        {
            exception_errors.push(format!(
                "policy exception `{}` has unknown governance_tier `{}`",
                exception.policy_id, exception.governance_tier
            ));
        }
        if exception.governance_tier != "governance-approved"
            && exception.expires_on == "9999-12-31"
        {
            exception_errors.push(format!(
                "policy exception `{}` uses a permanent expiry without governance approval",
                exception.policy_id
            ));
        }
        if ci_exception_is_expired(&exception.expires_on)? {
            exception_errors.push(format!(
                "policy exception `{}` expired on {}",
                exception.policy_id, exception.expires_on
            ));
        }
        if exception.renewal_reference.trim().is_empty() {
            exception_errors.push(format!(
                "policy exception `{}` is missing renewal_reference",
                exception.policy_id
            ));
        }
    }
    Ok(CiRegistryPolicyDrift {
        unplanned,
        uniqueness_errors,
        docs_errors,
        exception_errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate_dir
            .parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| panic!("repo root"))
            .to_path_buf()
    }

    #[test]
    fn ci_policy_registry_entries_are_all_planned_or_atlas() {
        let root = repo_root();
        let policy_drift = ci_registry_unplanned_entries(&root)
            .unwrap_or_else(|err| panic!("ci registry validation: {err}"));
        assert!(
            policy_drift.unplanned.is_empty(),
            "unexpected unplanned entries: {:?}",
            policy_drift.unplanned
        );
        assert!(
            policy_drift.uniqueness_errors.is_empty(),
            "unexpected uniqueness errors: {:?}",
            policy_drift.uniqueness_errors
        );
        assert!(
            policy_drift.docs_errors.is_empty(),
            "unexpected docs errors: {:?}",
            policy_drift.docs_errors
        );
        assert!(
            policy_drift.exception_errors.is_empty(),
            "unexpected exception errors: {:?}",
            policy_drift.exception_errors
        );
    }

    #[test]
    fn ci_exception_expiry_parser_handles_past_and_future_dates() {
        assert!(
            ci_exception_is_expired("2000-01-01").unwrap_or_else(|err| panic!("past date: {err}"))
        );
        assert!(!ci_exception_is_expired("2999-01-01")
            .unwrap_or_else(|err| panic!("future date: {err}")));
        assert!(ci_exception_is_expired("invalid").is_err());
    }
}
