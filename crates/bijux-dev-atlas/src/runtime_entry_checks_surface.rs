fn normalize_suite_name(raw: &str) -> Result<&str, String> {
    match raw {
        "ci-fast" => Ok("ci_fast"),
        "ci" => Ok("ci"),
        "local" => Ok("local"),
        "deep" => Ok("deep"),
        "required" => Ok("ci_pr"),
        "repo:required" => Ok("repo_required"),
        "repo:doctor" => Ok("repo_required"),
        "docs:required" => Ok("docs_required"),
        "configs:required" => Ok("configs_required"),
        "make:required" => Ok("make_required"),
        other => Ok(other),
    }
}

fn write_output_if_requested(out: Option<PathBuf>, rendered: &str) -> Result<(), String> {
    if let Some(path) = out {
        std::fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("cannot write {}: {err}", path.display()))?;
    }
    Ok(())
}

fn render_list_output(checks: &[CheckSpec], format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => {
            let mut lines = Vec::new();
            let mut current_domain = String::new();
            for check in checks {
                let domain = format!("{:?}", check.domain).to_ascii_lowercase();
                if domain != current_domain {
                    if !current_domain.is_empty() {
                        lines.push(String::new());
                    }
                    lines.push(format!("[{domain}]"));
                    current_domain = domain;
                }
                let tags = check
                    .tags
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                let suites = check
                    .suites
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(format!(
                    "{}\tbudget_ms={}\ttags={}\tsuites={}\t{}",
                    check.id, check.budget_ms, tags, suites, check.title
                ));
            }
            Ok(lines.join("\n"))
        }
        FormatArg::Json => {
            let rows: Vec<serde_json::Value> = checks
                .iter()
                .map(|check| {
                    serde_json::json!({
                        "id": check.id.as_str(),
                        "domain": format!("{:?}", check.domain).to_ascii_lowercase(),
                        "tags": check.tags.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
                        "suites": check.suites.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
                        "budget_ms": check.budget_ms,
                        "title": check.title,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&serde_json::json!({"checks": rows}))
                .map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for list".to_string()),
    }
}

fn render_explain_output(explain_text: String, format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => Ok(explain_text),
        FormatArg::Json => {
            let mut map = serde_json::Map::new();
            for line in explain_text.lines() {
                if let Some((key, value)) = line.split_once(": ") {
                    map.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
            serde_json::to_string_pretty(&serde_json::Value::Object(map))
                .map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for explain".to_string()),
    }
}

pub(crate) struct CheckListOptions {
    repo_root: Option<PathBuf>,
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
    format: FormatArg,
    out: Option<PathBuf>,
}

pub(crate) fn run_check_list(options: CheckListOptions) -> Result<(String, i32), String> {
    let root = resolve_repo_root(options.repo_root)?;
    let selectors = parse_selectors(
        options.suite,
        options.domain,
        options.tag,
        options.id,
        options.include_internal,
        options.include_slow,
    )?;
    let registry = load_registry(&root)?;
    let checks = select_checks(&registry, &selectors)?;
    let rendered = render_list_output(&checks, options.format)?;
    write_output_if_requested(options.out, &rendered)?;
    Ok((rendered, 0))
}

pub(crate) fn run_check_explain(
    check_id: String,
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry = load_registry(&root)?;
    let id = CheckId::parse(&check_id)?;
    let rendered = render_explain_output(explain_output(&registry, &id)?, format)?;
    write_output_if_requested(out, &rendered)?;
    Ok((rendered, 0))
}

pub(crate) struct CheckRunOptions {
    repo_root: Option<PathBuf>,
    artifacts_root: Option<PathBuf>,
    run_id: Option<String>,
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
    allow_subprocess: bool,
    allow_git: bool,
    allow_write: bool,
    allow_network: bool,
    fail_fast: bool,
    max_failures: Option<usize>,
    format: FormatArg,
    out: Option<PathBuf>,
    durations: usize,
}

#[derive(Debug, Serialize)]
struct DocsPageRow {
    path: String,
    in_nav: bool,
}

#[derive(Debug)]
struct DocsContext {
    repo_root: PathBuf,
    docs_root: PathBuf,
    artifacts_root: PathBuf,
    run_id: RunId,
}

#[derive(Default)]
struct DocsIssues {
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug)]
struct ConfigsContext {
    repo_root: PathBuf,
    configs_root: PathBuf,
    artifacts_root: PathBuf,
    run_id: RunId,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CiPolicyRegistry {
    schema_version: u64,
    entries: Vec<CiPolicyEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CiPolicyEntry {
    policy_id: String,
    workflow: String,
    job: String,
    step: String,
    classification: String,
    owner: String,
    status: String,
    #[serde(default)]
    control_plane_command: String,
    #[serde(default)]
    authoritative_implementation: String,
    replacement_plan: String,
    docs: String,
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
struct CiLaneSurfaceRegistry {
    schema_version: u64,
    lanes: Vec<CiLaneSurfaceEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CiLaneSurfaceEntry {
    lane: String,
    workflow: String,
    commands: Vec<CiLaneCommandEntry>,
    reports: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CiLaneCommandEntry {
    id: String,
    kind: String,
    command: String,
    #[serde(default)]
    suite: String,
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
struct WorkflowLintRow {
    workflow: String,
    job: String,
    step: String,
    step_kind: String,
    classification: String,
    allowed: bool,
    matched_pattern: String,
    registry_policy_id: String,
    allowlist_expires_on: String,
}

fn load_ci_policy_registry(repo_root: &Path) -> Result<CiPolicyRegistry, String> {
    let path = repo_root.join("configs/ci/policy-outside-control-plane.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_ci_policy_exceptions(repo_root: &Path) -> Result<CiPolicyExceptions, String> {
    let path = repo_root.join("configs/ci/policy-exceptions.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_ci_lane_surface(repo_root: &Path) -> Result<CiLaneSurfaceRegistry, String> {
    let path = repo_root.join("configs/ci/lane-surface.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_workflow_step_patterns(repo_root: &Path) -> Result<WorkflowStepPatterns, String> {
    let path = repo_root.join("configs/ci/workflow-step-patterns.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_workflow_allowlist(repo_root: &Path) -> Result<WorkflowAllowlist, String> {
    let path = repo_root.join("configs/ci/workflow-allowlist.json");
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

fn workflow_step_rows(repo_root: &Path) -> Result<Vec<WorkflowLintRow>, String> {
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
                let (step_kind, step_value) = if let Some(uses) =
                    step.get("uses").and_then(serde_yaml::Value::as_str)
                {
                    ("uses".to_string(), uses.to_string())
                } else if let Some(run) = step.get("run").and_then(serde_yaml::Value::as_str) {
                    ("run".to_string(), normalized_run_body(run))
                } else {
                    ("unknown".to_string(), String::new())
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
                    entry.workflow == workflow_rel && entry.job == job_name && entry.step == step_name
                });
                let registry_entry = registry.entries.iter().find(|entry| {
                    entry.workflow == workflow_rel && entry.job == job_name && entry.step == step_name
                });
                let classification = matched_pattern
                    .as_ref()
                    .map(|pattern| pattern.classification.clone())
                    .or_else(|| registry_entry.map(|entry| entry.classification.clone()))
                    .unwrap_or_else(|| "unknown".to_string());
                let allowed = matched_pattern.as_ref().is_some_and(|pattern| pattern.allowed)
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
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system clock before unix epoch: {err}"))?;
    let days = (duration.as_secs() / 86_400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

fn ci_exception_is_expired(raw: &str) -> Result<bool, String> {
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

fn ci_registry_unplanned_entries(
    repo_root: &Path,
) -> Result<(Vec<CiPolicyEntry>, Vec<String>, Vec<String>, Vec<String>), String> {
    let registry = load_ci_policy_registry(repo_root)?;
    let exceptions = load_ci_policy_exceptions(repo_root)?;
    let mut unplanned = Vec::new();
    let mut uniqueness_errors = Vec::new();
    let mut docs_errors = Vec::new();
    let mut exception_errors = Vec::new();
    let mut seen_policy_ids = std::collections::BTreeSet::<String>::new();
    for entry in &registry.entries {
        if entry.classification == "policy" && entry.status != "atlas" && entry.status != "planned" && entry.status != "exception" {
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
        if exception.governance_tier != "governance-approved" && exception.expires_on == "9999-12-31"
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
    Ok((unplanned, uniqueness_errors, docs_errors, exception_errors))
}

fn render_ci_explain(
    repo_root: &Path,
    lane: &str,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let lane_surface = load_ci_lane_surface(repo_root)?;
    let registry = load_registry(repo_root)?;
    let Some(lane_entry) = lane_surface.lanes.into_iter().find(|row| row.lane == lane) else {
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "ci_explain",
            "status": "not_found",
            "lane": lane
        });
        let rendered = emit_payload(format, out, &payload)?;
        return Ok((rendered, 1));
    };
    let mut checks = Vec::<serde_json::Value>::new();
    for command in &lane_entry.commands {
        if command.kind == "suite" && !command.suite.is_empty() {
            let selectors = parse_selectors(
                Some(command.suite.clone()),
                None,
                None,
                None,
                true,
                true,
            )?;
            let selected = select_checks(&registry, &selectors)?;
            for check in selected {
                checks.push(serde_json::json!({
                    "suite": command.suite,
                    "id": check.id.as_str(),
                    "domain": format!("{:?}", check.domain).to_ascii_lowercase(),
                    "title": check.title
                }));
            }
        }
    }
    checks.sort_by(|a, b| {
        a["id"]
            .as_str()
            .cmp(&b["id"].as_str())
            .then_with(|| a["suite"].as_str().cmp(&b["suite"].as_str()))
    });
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "ci_explain",
        "status": "ok",
        "lane": lane_entry.lane,
        "workflow": lane_entry.workflow,
        "commands": lane_entry.commands,
        "reports": lane_entry.reports,
        "checks": checks
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

fn render_ci_report(
    repo_root: &Path,
    kind: &str,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let policy_registry = load_ci_policy_registry(repo_root)?;
    let lane_surface = load_ci_lane_surface(repo_root)?;
    let (unplanned, uniqueness_errors, docs_errors, exception_errors) =
        ci_registry_unplanned_entries(repo_root)?;
    let payload = match kind {
        "lane-parity" => {
            let mut lane_rows = Vec::<serde_json::Value>::new();
            for lane in lane_surface.lanes {
                let command_ids = lane
                    .commands
                    .iter()
                    .map(|row| row.id.clone())
                    .collect::<Vec<_>>();
                lane_rows.push(serde_json::json!({
                    "lane": lane.lane,
                    "workflow": lane.workflow,
                    "command_ids": command_ids,
                    "report_count": lane.reports.len()
                }));
            }
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_lane_parity",
                "lanes": lane_rows
            })
        }
        "policy-diff" => {
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_policy_diff",
                "summary": {
                    "entries": policy_registry.entries.len(),
                    "unplanned": unplanned.len(),
                    "uniqueness_errors": uniqueness_errors.len(),
                    "docs_errors": docs_errors.len(),
                    "exception_errors": exception_errors.len()
                },
                "unplanned": unplanned,
                "uniqueness_errors": uniqueness_errors,
                "docs_errors": docs_errors,
                "exception_errors": exception_errors
            })
        }
        "atlas-authority" => {
            let atlas = policy_registry
                .entries
                .into_iter()
                .filter(|entry| entry.status == "atlas")
                .collect::<Vec<_>>();
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_atlas_authority",
                "all_policy_implementations_live_in_atlas": atlas.iter().all(|entry| !entry.control_plane_command.is_empty()),
                "atlas_entries": atlas
            })
        }
        "workflow-lint" => {
            let rows = workflow_step_rows(repo_root)?;
            let violations = rows
                .iter()
                .filter(|row| !row.allowed)
                .map(|row| {
                    serde_json::json!({
                        "workflow": row.workflow,
                        "job": row.job,
                        "step": row.step,
                        "classification": row.classification,
                        "registry_policy_id": row.registry_policy_id
                    })
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_workflow_lint",
                "summary": {
                    "steps": rows.len(),
                    "violations": violations.len()
                },
                "rows": rows,
                "violations": violations
            })
        }
        other => {
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "ci_report",
                "status": "unknown_kind",
                "requested_kind": other
            });
            let rendered = emit_payload(format, out, &payload)?;
            return Ok((rendered, 1));
        }
    };
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

fn run_ci_verify_gate(
    repo_root: &Path,
    gate: &str,
    format: FormatArg,
    out: Option<PathBuf>,
    allow_subprocess: bool,
    allow_git: bool,
    allow_write: bool,
    allow_network: bool,
) -> Result<(String, i32), String> {
    let payload = match gate {
        "workflow-policy" => {
            let workflow = repo_root.join(".github/workflows/ci-pr.yml");
            let text = fs::read_to_string(&workflow)
                .map_err(|err| format!("read {} failed: {err}", workflow.display()))?;
            let mut errors = Vec::<String>::new();
            for required in [
                ".github/dependabot.yml",
                ".github/CODEOWNERS",
                "configs/ci/policy-outside-control-plane.json",
                "configs/ci/lane-surface.json",
            ] {
                if !repo_root.join(required).exists() {
                    errors.push(format!("missing required workflow policy file `{required}`"));
                }
            }
            if !text.contains("actions/checkout@") {
                errors.push("workflow-policy job must keep checkout".to_string());
            }
            let (unplanned, uniqueness_errors, docs_errors, exception_errors) =
                ci_registry_unplanned_entries(repo_root)?;
            errors.extend(
                unplanned
                    .into_iter()
                    .map(|entry| format!("unplanned ci policy entry `{}`", entry.policy_id)),
            );
            errors.extend(uniqueness_errors);
            errors.extend(docs_errors);
            errors.extend(exception_errors);
            let lint_rows = workflow_step_rows(repo_root)?;
            errors.extend(lint_rows.into_iter().filter(|row| !row.allowed).map(|row| {
                format!(
                    "workflow step `{}` in {}/{} is not matched by an allowed pattern or active allowlist",
                    row.step, row.workflow, row.job
                )
            }));
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify_workflow_policy",
                "status": if errors.is_empty() { "ok" } else { "failed" },
                "errors": errors
            })
        }
        "workflow-lint" => {
            let rows = workflow_step_rows(repo_root)?;
            let errors = rows
                .iter()
                .filter(|row| !row.allowed)
                .map(|row| {
                    format!(
                        "workflow step `{}` in {}/{} is not allowed by workflow-step-patterns or workflow-allowlist",
                        row.step, row.workflow, row.job
                    )
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify_workflow_lint",
                "status": if errors.is_empty() { "ok" } else { "failed" },
                "rows": rows,
                "errors": errors
            })
        }
        "dependency-lock" => {
            if !allow_git {
                return Err("ci verify dependency-lock requires --allow-git".to_string());
            }
            let output = ProcessCommand::new("git")
                .current_dir(repo_root)
                .args(["diff", "--name-only"])
                .output()
                .map_err(|err| format!("git diff failed: {err}"))?;
            if !output.status.success() {
                return Err("git diff --name-only failed".to_string());
            }
            let changed = String::from_utf8(output.stdout).map_err(|err| err.to_string())?;
            let unexpected = changed
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && *line != "Cargo.lock")
                .map(str::to_string)
                .collect::<Vec<_>>();
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify_dependency_lock",
                "status": if unexpected.is_empty() { "ok" } else { "failed" },
                "unexpected_paths": unexpected
            })
        }
        "release-candidate" => {
            if !allow_git {
                return Err("ci verify release-candidate requires --allow-git".to_string());
            }
            let mut errors = Vec::<String>::new();
            for args in [
                vec!["diff", "--quiet"],
                vec!["diff", "--cached", "--quiet"],
                vec!["diff", "--quiet", "Cargo.lock"],
            ] {
                let status = ProcessCommand::new("git")
                    .current_dir(repo_root)
                    .args(&args)
                    .status()
                    .map_err(|err| format!("git {:?} failed: {err}", args))?;
                if !status.success() {
                    errors.push(format!("git {:?} reported a dirty tree", args));
                }
            }
            let metadata = ProcessCommand::new("cargo")
                .current_dir(repo_root)
                .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
                .output()
                .map_err(|err| format!("cargo metadata failed: {err}"))?;
            if !metadata.status.success() {
                errors.push("cargo metadata --locked failed".to_string());
            }
            let mut package_version = String::new();
            if metadata.status.success() {
                let value: serde_json::Value =
                    serde_json::from_slice(&metadata.stdout).map_err(|err| err.to_string())?;
                package_version = value["packages"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|pkg| pkg["name"].as_str() == Some("bijux-dev-atlas"))
                    .and_then(|pkg| pkg["version"].as_str())
                    .unwrap_or_default()
                    .to_string();
            }
            let git_ref = std::env::var("GITHUB_REF").unwrap_or_default();
            if let Some(tag) = git_ref.strip_prefix("refs/tags/") {
                if format!("v{package_version}") != tag {
                    errors.push(format!(
                        "tag `{tag}` does not match bijux-dev-atlas version `v{package_version}`"
                    ));
                }
            }
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify_release_candidate",
                "status": if errors.is_empty() { "ok" } else { "failed" },
                "package_version": package_version,
                "git_ref": git_ref,
                "errors": errors
            })
        }
        "docs-preview" => {
            if !allow_subprocess || !allow_write {
                return Err("ci verify docs-preview requires --allow-subprocess and --allow-write".to_string());
            }
            let docs_common = DocsCommonArgs {
                repo_root: Some(repo_root.to_path_buf()),
                artifacts_root: None,
                run_id: None,
                format: FormatArg::Json,
                out: None,
                allow_subprocess,
                allow_write,
                allow_network,
                strict: true,
                include_drafts: false,
            };
            let build_code = run_docs_command(true, DocsCommand::Build(docs_common));
            let site_payload = bijux_dev_atlas::docs::site_output::site_output_report(repo_root)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify_docs_preview",
                "status": if build_code == 0 && site_payload["status"].as_str() == Some("pass") { "ok" } else { "failed" },
                "build_exit_code": build_code,
                "site_output": site_payload
            });
            payload
        }
        "docs-diff" => {
            if !allow_git || !allow_write {
                return Err("ci verify docs-diff requires --allow-git and --allow-write".to_string());
            }
            let output = ProcessCommand::new("git")
                .current_dir(repo_root)
                .args([
                    "diff",
                    "--name-only",
                    "HEAD~1...HEAD",
                    "--",
                    "docs/**",
                    "configs/docs/**",
                    "ops/report/docs/**",
                    "docker/**",
                    "make/**",
                ])
                .output()
                .map_err(|err| format!("git diff for docs failed: {err}"))?;
            let changed = String::from_utf8(output.stdout)
                .map_err(|err| err.to_string())?
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>();
            serde_json::json!({
                "schema_version": 1,
                "kind": "docs_diff_summary_v1",
                "changed_count": changed.len(),
                "changed_paths": changed
            })
        }
        "docs-quality" => {
            if !allow_write {
                return Err("ci verify docs-quality requires --allow-write".to_string());
            }
            let path = repo_root.join("docs/_internal/generated/docs-test-coverage.json");
            if !path.exists() {
                serde_json::json!({
                    "schema_version": 1,
                    "kind": "ci_verify_docs_quality",
                    "status": "failed",
                    "errors": ["missing docs/_internal/generated/docs-test-coverage.json"]
                })
            } else {
                let payload: serde_json::Value = serde_json::from_str(
                    &fs::read_to_string(&path)
                        .map_err(|err| format!("read {} failed: {err}", path.display()))?,
                )
                .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
                serde_json::json!({
                    "schema_version": 1,
                    "kind": "ci_verify_docs_quality",
                    "status": "ok",
                    "coverage": payload
                })
            }
        }
        "rust-fmt" => {
            if !allow_subprocess {
                return Err("ci verify rust-fmt requires --allow-subprocess".to_string());
            }
            let output = ProcessCommand::new("cargo")
                .current_dir(repo_root)
                .args(["fmt", "--all", "--", "--check", "--config-path", "configs/rust/rustfmt.toml"])
                .output()
                .map_err(|err| format!("cargo fmt failed: {err}"))?;
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify_rust_fmt",
                "status": if output.status.success() { "ok" } else { "failed" },
                "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
                "stderr": String::from_utf8_lossy(&output.stderr).to_string()
            })
        }
        "rust-clippy" => {
            if !allow_subprocess {
                return Err("ci verify rust-clippy requires --allow-subprocess".to_string());
            }
            let output = ProcessCommand::new("cargo")
                .current_dir(repo_root)
                .env("CLIPPY_CONF_DIR", "configs/rust")
                .args(["clippy", "-q", "--workspace", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"])
                .output()
                .map_err(|err| format!("cargo clippy failed: {err}"))?;
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify_rust_clippy",
                "status": if output.status.success() { "ok" } else { "failed" },
                "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
                "stderr": String::from_utf8_lossy(&output.stderr).to_string()
            })
        }
        other => {
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify",
                "status": "unknown_gate",
                "gate": other
            });
            let rendered = emit_payload(format, out, &payload)?;
            return Ok((rendered, 1));
        }
    };
    let code = if payload["status"].as_str() == Some("ok") {
        0
    } else {
        1
    };
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, code))
}

pub(crate) fn run_check_run(options: CheckRunOptions) -> Result<(String, i32), String> {
    let root = resolve_repo_root(options.repo_root)?;
    let selectors = parse_selectors(
        options.suite,
        options.domain,
        options.tag,
        options.id,
        options.include_internal,
        options.include_slow,
    )?;
    let request = RunRequest {
        repo_root: root.clone(),
        domain: selectors.domain,
        capabilities: Capabilities::from_cli_flags(
            options.allow_write,
            options.allow_subprocess,
            options.allow_git,
            options.allow_network,
        ),
        artifacts_root: options
            .artifacts_root
            .or_else(|| Some(root.join("artifacts"))),
        run_id: options.run_id.map(|rid| RunId::parse(&rid)).transpose()?,
        command: Some("bijux dev atlas check run".to_string()),
    };
    let run_options = RunOptions {
        fail_fast: options.fail_fast,
        max_failures: options.max_failures,
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &run_options,
    )?;
    let rendered = match options.format {
        FormatArg::Text => render_text_with_durations(&report, options.durations),
        FormatArg::Json => render_json(&report)?,
        FormatArg::Jsonl => render_jsonl(&report)?,
    };
    write_output_if_requested(options.out, &rendered)?;
    Ok((rendered, exit_code_for_report(&report)))
}

pub(crate) fn run_workflows_command(quiet: bool, command: WorkflowsCommand) -> i32 {
    match command {
        WorkflowsCommand::Validate {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match run_check_run(CheckRunOptions {
            repo_root,
            artifacts_root: None,
            run_id: None,
            suite: None,
            domain: Some(DomainArg::Workflows),
            tag: None,
            id: None,
            include_internal,
            include_slow,
            allow_subprocess: false,
            allow_git: false,
            allow_write: false,
            allow_network: false,
            fail_fast: false,
            max_failures: None,
            format,
            out,
            durations: 0,
        }) {
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
                let _ = writeln!(
                    io::stderr(),
                    "bijux-dev-atlas workflows validate failed: {err}"
                );
                1
            }
        },
        WorkflowsCommand::Doctor {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match run_check_doctor(repo_root, include_internal, include_slow, format, out) {
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
                let _ = writeln!(
                    io::stderr(),
                    "bijux-dev-atlas workflows doctor failed: {err}"
                );
                1
            }
        },
        WorkflowsCommand::Surface {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match run_check_list(CheckListOptions {
            repo_root,
            suite: None,
            domain: Some(DomainArg::Workflows),
            tag: None,
            id: None,
            include_internal,
            include_slow,
            format,
            out,
        }) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(
                    io::stderr(),
                    "bijux-dev-atlas workflows surface failed: {err}"
                );
                1
            }
        },
        WorkflowsCommand::Explain {
            lane,
            repo_root,
            format,
            out,
        } => match resolve_repo_root(repo_root).and_then(|root| render_ci_explain(&root, &lane, format, out)) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas ci explain failed: {err}");
                1
            }
        },
        WorkflowsCommand::Report {
            repo_root,
            kind,
            format,
            out,
        } => match resolve_repo_root(repo_root).and_then(|root| render_ci_report(&root, &kind, format, out)) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas ci report failed: {err}");
                1
            }
        },
        WorkflowsCommand::Verify {
            gate,
            repo_root,
            format,
            out,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
        } => match resolve_repo_root(repo_root).and_then(|root| {
            run_ci_verify_gate(
                &root,
                &gate,
                format,
                out,
                allow_subprocess,
                allow_git,
                allow_write,
                allow_network,
            )
        }) {
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
                let _ = writeln!(io::stderr(), "bijux-dev-atlas ci verify failed: {err}");
                1
            }
        },
    }
}

pub(crate) fn run_gates_command(quiet: bool, command: GatesCommand) -> i32 {
    match command {
        GatesCommand::List {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match run_check_list(CheckListOptions {
            repo_root,
            suite: None,
            domain: None,
            tag: None,
            id: None,
            include_internal,
            include_slow,
            format,
            out,
        }) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas gates list failed: {err}");
                1
            }
        },
        GatesCommand::Run {
            repo_root,
            artifacts_root,
            run_id,
            suite,
            include_internal,
            include_slow,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
            fail_fast,
            max_failures,
            format,
            out,
            durations,
        } => match run_check_run(CheckRunOptions {
            repo_root,
            artifacts_root,
            run_id,
            suite: Some(suite),
            domain: None,
            tag: None,
            id: None,
            include_internal,
            include_slow,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
            fail_fast,
            max_failures,
            format,
            out,
            durations,
        }) {
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
                let _ = writeln!(io::stderr(), "bijux-dev-atlas gates run failed: {err}");
                1
            }
        },
    }
}

pub(crate) fn run_check_doctor(
    repo_root: Option<PathBuf>,
    include_internal: bool,
    include_slow: bool,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry_report = registry_doctor(&root);
    let inventory_errors = validate_ops_inventory(&root);
    let selectors = parse_selectors(
        Some("doctor".to_string()),
        None,
        None,
        None,
        include_internal,
        include_slow,
    )?;
    let request = RunRequest {
        repo_root: root.clone(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: Some(root.join("artifacts")),
        run_id: Some(RunId::from_seed("doctor_run")),
        command: Some("bijux dev atlas doctor".to_string()),
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &RunOptions::default(),
    )?;
    let docs_common = DocsCommonArgs {
        repo_root: Some(root.clone()),
        artifacts_root: Some(root.join("artifacts")),
        run_id: Some("doctor_docs".to_string()),
        format,
        out: None,
        allow_subprocess: false,
        allow_write: false,
        allow_network: false,
        strict: false,
        include_drafts: false,
    };
    let docs_ctx = docs_context(&docs_common)?;
    let docs_validate = docs_validate_payload(&docs_ctx, &docs_common)?;
    let docs_links = docs_links_payload(&docs_ctx, &docs_common)?;
    let docs_lint = docs_lint_payload(&docs_ctx, &docs_common)?;
    let configs_common = ConfigsCommonArgs {
        repo_root: Some(root.clone()),
        artifacts_root: Some(root.join("artifacts")),
        run_id: Some("doctor_configs".to_string()),
        format,
        out: None,
        allow_write: false,
        allow_subprocess: false,
        allow_network: false,
        strict: false,
    };
    let configs_ctx = configs_context(&configs_common)?;
    let configs_validate = configs_validate_payload(&configs_ctx, &configs_common)?;
    let configs_lint = configs_lint_payload(&configs_ctx, &configs_common)?;
    let configs_diff = configs_diff_payload(&configs_ctx, &configs_common)?;
    let check_exit = exit_code_for_report(&report);
    let inventory_error_count = inventory_errors.len();
    let ops_doctor_status = if inventory_errors.is_empty() && check_exit == 0 {
        "ok"
    } else {
        "failed"
    };
    let docs_error_count = docs_validate
        .get("errors")
        .and_then(|v| v.as_array())
        .map_or(0, Vec::len)
        + docs_links
            .get("errors")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len)
        + docs_lint
            .get("errors")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len);
    let configs_error_count = configs_validate
        .get("errors")
        .and_then(|v| v.as_array())
        .map_or(0, Vec::len)
        + configs_lint
            .get("errors")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len)
        + configs_diff
            .get("errors")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len);
    // Top-level doctor remains a stable fast governance health gate. Docs/configs summaries are
    // reported for visibility but do not fail the command by default because they contain broad
    // repo lint signals that are not part of the curated doctor contract.
    let status =
        if registry_report.errors.is_empty() && inventory_errors.is_empty() && check_exit == 0 {
            "ok"
        } else {
            "failed"
        };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "registry_errors": registry_report.errors,
        "inventory_errors": inventory_errors,
        "ops_doctor": {
            "status": ops_doctor_status,
            "inventory_errors": inventory_error_count,
            "checks_exit": check_exit
        },
        "docs_doctor": {
            "validate_errors": docs_validate.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "links_errors": docs_links.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "lint_errors": docs_lint.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "status": if docs_error_count == 0 { "ok" } else { "failed" }
        },
        "configs_doctor": {
            "validate_errors": configs_validate.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "lint_errors": configs_lint.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "diff_errors": configs_diff.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "status": if configs_error_count == 0 { "ok" } else { "failed" }
        },
        "control_plane_doctor": {
            "status": status,
            "ops": {"status": ops_doctor_status, "errors": inventory_error_count + usize::from(check_exit != 0)},
            "docs": {"status": if docs_error_count == 0 { "ok" } else { "failed" }, "errors": docs_error_count},
            "configs": {"status": if configs_error_count == 0 { "ok" } else { "failed" }, "errors": configs_error_count}
        },
        "check_report": report,
    });

    let evidence_dir = root.join("artifacts/atlas-dev/doctor");
    fs::create_dir_all(&evidence_dir)
        .map_err(|err| format!("failed to create {}: {err}", evidence_dir.display()))?;
    let evidence_path = evidence_dir.join("doctor.report.json");
    fs::write(
        &evidence_path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", evidence_path.display()))?;

    let rendered = match format {
        FormatArg::Text => format!(
            "status: {status}\nregistry_errors: {}\ninventory_errors: {}\ncheck_summary: passed={} failed={} skipped={} errors={} total={}\nevidence: {}",
            payload["registry_errors"].as_array().map_or(0, Vec::len),
            payload["inventory_errors"].as_array().map_or(0, Vec::len),
            report.summary.passed,
            report.summary.failed,
            report.summary.skipped,
            report.summary.errors,
            report.summary.total,
            evidence_path.display(),
        ),
        FormatArg::Json => serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => serde_json::to_string(&payload).map_err(|err| err.to_string())?,
    };
    write_output_if_requested(out, &rendered)?;
    let exit = if status == "ok" { 0 } else { 1 };
    Ok((rendered, exit))
}

#[cfg(test)]
mod ci_tests {
    use super::*;

    fn repo_root() -> PathBuf {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate_dir
            .parent()
            .and_then(Path::parent)
            .expect("repo root")
            .to_path_buf()
    }

    #[test]
    fn ci_policy_registry_entries_are_all_planned_or_atlas() {
        let root = repo_root();
        let (unplanned, uniqueness_errors, docs_errors, exception_errors) =
            ci_registry_unplanned_entries(&root).expect("ci registry validation");
        assert!(unplanned.is_empty(), "unexpected unplanned entries: {unplanned:?}");
        assert!(
            uniqueness_errors.is_empty(),
            "unexpected uniqueness errors: {uniqueness_errors:?}"
        );
        assert!(docs_errors.is_empty(), "unexpected docs errors: {docs_errors:?}");
        assert!(
            exception_errors.is_empty(),
            "unexpected exception errors: {exception_errors:?}"
        );
    }

    #[test]
    fn ci_exception_expiry_parser_handles_past_and_future_dates() {
        assert!(ci_exception_is_expired("2000-01-01").expect("past date"));
        assert!(!ci_exception_is_expired("2999-01-01").expect("future date"));
        assert!(ci_exception_is_expired("invalid").is_err());
    }
}
