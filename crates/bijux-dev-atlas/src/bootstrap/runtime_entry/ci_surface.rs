use super::*;
#[path = "ci_surface_registry.rs"]
mod ci_registry;

use ci_registry::{
    ci_registry_unplanned_entries, load_ci_lane_surface, load_ci_lanes_registry,
    load_ci_policy_registry, workflow_step_rows,
};

struct CiVerifyRunOptions {
    format: FormatArg,
    out: Option<PathBuf>,
    allow_subprocess: bool,
    allow_git: bool,
    allow_write: bool,
    allow_network: bool,
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

fn render_ci_lanes_list(
    repo_root: &Path,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let mut lanes = load_ci_lanes_registry(repo_root)?.lanes;
    lanes.sort_by(|a, b| a.id.cmp(&b.id));
    let rows = lanes
        .into_iter()
        .map(|lane| {
            serde_json::json!({
                "id": lane.id,
                "description": lane.description,
                "mode": lane.mode,
                "timeout_class": lane.timeout_class,
                "concurrency_class": lane.concurrency_class
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "ci_lanes_list",
        "status": "ok",
        "rows": rows
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

fn render_ci_lanes_explain(
    repo_root: &Path,
    lane_id: &str,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let lanes = load_ci_lanes_registry(repo_root)?;
    let Some(lane) = lanes.lanes.into_iter().find(|row| row.id == lane_id) else {
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "ci_lanes_explain",
            "status": "not_found",
            "lane_id": lane_id
        });
        let rendered = emit_payload(format, out, &payload)?;
        return Ok((rendered, 1));
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "ci_lanes_explain",
        "status": "ok",
        "lane": lane
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

fn validate_ci_lanes_registry(repo_root: &Path) -> Result<serde_json::Value, String> {
    let registry = load_ci_lanes_registry(repo_root)?;
    let lane_surface = load_ci_lane_surface(repo_root)?;
    let mut errors = Vec::<String>::new();
    let mut seen = std::collections::BTreeSet::<String>::new();
    let mut seen_aliases = std::collections::BTreeSet::<String>::new();
    let mut ordered = Vec::<String>::new();
    for required in ["small", "medium", "large"] {
        if !registry.concurrency_classes.contains_key(required) {
            errors.push(format!("missing concurrency class `{required}`"));
        }
        if !registry.timeout_classes.contains_key(required) {
            errors.push(format!("missing timeout class `{required}`"));
        }
    }
    let mut lane_id_set = std::collections::BTreeSet::<String>::new();
    for lane in &registry.lanes {
        lane_id_set.insert(lane.id.clone());
    }
    for lane in &registry.lanes {
        if !seen.insert(lane.id.clone()) {
            errors.push(format!("duplicate lane id `{}`", lane.id));
        }
        ordered.push(lane.id.clone());
        if lane.aliases != {
            let mut sorted = lane.aliases.clone();
            sorted.sort();
            sorted
        } {
            errors.push(format!("lane `{}` aliases must be deterministically ordered", lane.id));
        }
        for alias in &lane.aliases {
            if alias == &lane.id {
                errors.push(format!(
                    "lane `{}` alias `{}` must differ from canonical id",
                    lane.id, alias
                ));
            }
            if lane_id_set.contains(alias) {
                errors.push(format!(
                    "lane `{}` alias `{}` collides with another lane id",
                    lane.id, alias
                ));
            }
            if !seen_aliases.insert(alias.clone()) {
                errors.push(format!("duplicate lane alias `{alias}`"));
            }
        }
        if lane.description.trim().is_empty() {
            errors.push(format!("lane `{}` is missing description", lane.id));
        }
        if lane.artifacts_expected.is_empty() {
            errors.push(format!("lane `{}` must declare artifacts_expected", lane.id));
        }
        if lane.evidence_bundle.trim().is_empty() {
            errors.push(format!("lane `{}` must declare evidence_bundle", lane.id));
        }
        if !lane.evidence_bundle.starts_with("artifacts/") {
            errors.push(format!(
                "lane `{}` evidence_bundle must live under artifacts/: `{}`",
                lane.id, lane.evidence_bundle
            ));
        }
        if !lane.command.starts_with("bijux dev atlas ") {
            errors.push(format!(
                "lane `{}` command must start with `bijux dev atlas `",
                lane.id
            ));
        }
        if !registry
            .concurrency_classes
            .contains_key(lane.concurrency_class.as_str())
        {
            errors.push(format!(
                "lane `{}` references undefined concurrency_class `{}`",
                lane.id, lane.concurrency_class
            ));
        }
        if !registry
            .timeout_classes
            .contains_key(lane.timeout_class.as_str())
        {
            errors.push(format!(
                "lane `{}` references undefined timeout_class `{}`",
                lane.id, lane.timeout_class
            ));
        }
    }
    let mut sorted = ordered.clone();
    sorted.sort();
    if ordered != sorted {
        errors.push("lane ids must be deterministically ordered".to_string());
    }
    let workflow_to_lanes = lane_surface
        .lanes
        .iter()
        .map(|row| (row.workflow.as_str(), row.lane.as_str()))
        .collect::<Vec<_>>();
    for (class, class_def) in &registry.concurrency_classes {
        let mut workflow_counts = std::collections::BTreeMap::<String, usize>::new();
        for (workflow, lane) in &workflow_to_lanes {
            if let Some(def) = registry.lanes.iter().find(|item| item.id == *lane) {
                if def.concurrency_class == *class {
                    *workflow_counts.entry((*workflow).to_string()).or_default() += 1;
                }
            }
        }
        for (workflow, count) in workflow_counts {
            if (count as u64) > class_def.max_parallelism {
                errors.push(format!(
                    "workflow `{workflow}` exceeds concurrency class `{class}` max_parallelism: {count} > {}",
                    class_def.max_parallelism
                ));
            }
        }
    }
    let workflow_timeouts = collect_workflow_timeout_minutes(repo_root)?;
    for lane in &lane_surface.lanes {
        let Some(lane_def) = registry.lanes.iter().find(|item| item.id == lane.lane) else {
            continue;
        };
        let Some(timeout_def) = registry.timeout_classes.get(lane_def.timeout_class.as_str()) else {
            continue;
        };
        let timeout = workflow_timeouts
            .get(lane.workflow.as_str())
            .copied()
            .unwrap_or(0);
        if timeout > timeout_def.max_minutes {
            errors.push(format!(
                "lane `{}` workflow `{}` timeout {} exceeds timeout class `{}` max {}",
                lane.lane, lane.workflow, timeout, lane_def.timeout_class, timeout_def.max_minutes
            ));
        }
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "ci_lanes_validate",
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "errors": errors,
        "summary": {
            "lanes": registry.lanes.len()
        }
    }))
}

fn collect_workflow_timeout_minutes(
    repo_root: &Path,
) -> Result<std::collections::BTreeMap<String, u64>, String> {
    let mut result = std::collections::BTreeMap::<String, u64>::new();
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
        let mut max_timeout = 0_u64;
        for (_, job_value) in jobs {
            let timeout = job_value
                .get("timeout-minutes")
                .and_then(serde_yaml::Value::as_u64)
                .unwrap_or(0);
            if timeout > max_timeout {
                max_timeout = timeout;
            }
        }
        result.insert(workflow_rel, max_timeout);
    }
    Ok(result)
}

fn render_ci_lanes_validate(
    repo_root: &Path,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let payload = validate_ci_lanes_registry(repo_root)?;
    let code = if payload["status"] == "ok" { 0 } else { 1 };
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, code))
}

fn render_ci_env_contract_validate(
    repo_root: &Path,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let path = repo_root.join("configs/ci/env-contract.json");
    let text =
        fs::read_to_string(&path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let status = if value
        .get("required_job_env_keys")
        .and_then(|v| v.as_array())
        .is_some()
    {
        "ok"
    } else {
        "failed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "ci_env_contract_validate",
        "status": status,
        "path": "configs/ci/env-contract.json"
    });
    let code = if status == "ok" { 0 } else { 1 };
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, code))
}

fn render_ci_simulate(
    repo_root: &Path,
    lane: Option<String>,
    matrix: bool,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let registry = load_ci_lanes_registry(repo_root)?;
    let selected = if matrix {
        registry.lanes
    } else if let Some(id) = lane {
        let Some(row) = registry.lanes.into_iter().find(|row| row.id == id) else {
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "ci_simulate",
                "status": "not_found",
                "lane": id
            });
            let rendered = emit_payload(format, out, &payload)?;
            return Ok((rendered, 1));
        };
        vec![row]
    } else {
        return Err("ci simulate requires --lane <id> or --matrix".to_string());
    };
    let mut rows = Vec::<serde_json::Value>::new();
    let mut missing_artifacts = Vec::<String>::new();
    let mut missing_evidence_bundles = Vec::<String>::new();
    for row in selected {
        let mut missing = Vec::<String>::new();
        for artifact in &row.artifacts_expected {
            if !repo_root.join(artifact).exists() {
                missing.push(artifact.clone());
            }
        }
        let evidence_missing = !repo_root.join(&row.evidence_bundle).exists();
        if evidence_missing {
            missing_evidence_bundles.push(format!("{}:{}", row.id, row.evidence_bundle));
        }
        if !missing.is_empty() {
            missing_artifacts.extend(missing.iter().map(|item| format!("{}:{item}", row.id)));
        }
        rows.push(serde_json::json!({
            "lane": row.id,
            "mode": row.mode,
            "command": row.command,
            "artifacts_expected": row.artifacts_expected,
            "evidence_bundle": row.evidence_bundle,
            "missing_artifacts": missing,
            "missing_evidence_bundle": evidence_missing,
            "status": if missing.is_empty() && !evidence_missing { "ok" } else { "incomplete" }
        }));
    }
    rows.sort_by(|a, b| a["lane"].as_str().cmp(&b["lane"].as_str()));
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "ci_simulate",
        "status": if missing_artifacts.is_empty() && missing_evidence_bundles.is_empty() { "ok" } else { "failed" },
        "rows": rows,
        "summary": {
            "lanes": rows.len(),
            "missing_artifacts": missing_artifacts.len(),
            "missing_evidence_bundles": missing_evidence_bundles.len()
        },
        "artifact_completeness": {
            "missing_artifacts": missing_artifacts,
            "missing_evidence_bundles": missing_evidence_bundles
        }
    });
    let code = if payload["status"] == "ok" { 0 } else { 1 };
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, code))
}

fn render_ci_report(
    repo_root: &Path,
    kind: &str,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let policy_registry = load_ci_policy_registry(repo_root)?;
    let lane_surface = load_ci_lane_surface(repo_root)?;
    let policy_drift = ci_registry_unplanned_entries(repo_root)?;
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
                    "unplanned": policy_drift.unplanned.len(),
                    "uniqueness_errors": policy_drift.uniqueness_errors.len(),
                    "docs_errors": policy_drift.docs_errors.len(),
                    "exception_errors": policy_drift.exception_errors.len()
                },
                "unplanned": policy_drift.unplanned,
                "uniqueness_errors": policy_drift.uniqueness_errors,
                "docs_errors": policy_drift.docs_errors,
                "exception_errors": policy_drift.exception_errors
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

fn github_ref_from_process_env() -> String {
    std::env::vars_os()
        .find(|(key, _)| key == "GITHUB_REF")
        .and_then(|(_, value)| value.into_string().ok())
        .unwrap_or_default()
}

fn lane_id_or_alias(lane_registry: &ci_registry::CiLaneRegistry, id: &str) -> Option<String> {
    if lane_registry.lanes.iter().any(|lane| lane.id == id) {
        return Some(id.to_string());
    }
    lane_registry
        .lanes
        .iter()
        .find(|lane| lane.aliases.iter().any(|alias| alias == id))
        .map(|lane| lane.id.clone())
}

fn lane_domain_from_id(lane_id: &str) -> &'static str {
    if lane_id.starts_with("docs-") {
        "docs"
    } else if lane_id.starts_with("ops-") {
        "ops"
    } else if lane_id.starts_with("ci-") {
        "ci"
    } else if lane_id.starts_with("release-") {
        "release"
    } else if lane_id.starts_with("dependency-") {
        "dependency"
    } else {
        "generic"
    }
}

fn domain_path_prefixes(domain: &str) -> &'static [&'static str] {
    match domain {
        "docs" => &["docs/", "mkdocs.yml", "configs/docs/"],
        "ops" => &["ops/", "configs/ops/", ".github/workflows/ops-"],
        "ci" => &[".github/workflows/", "configs/ci/", "crates/bijux-dev-atlas/"],
        "release" => &["ops/release/", "docs/", "configs/release/", ".github/workflows/release-"],
        "dependency" => &["Cargo.lock", ".github/workflows/dependency-"],
        _ => &[".github/workflows/"],
    }
}

fn scan_workflow_referenced_lane_ids(workflow_text: &str) -> Vec<String> {
    workflow_text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- lane:"))
        .map(str::trim)
        .map(|value| {
            value
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
                .collect::<String>()
        })
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
}

fn scan_workflow_secret_names(workflow_text: &str) -> Vec<String> {
    let mut out = Vec::<String>::new();
    let mut cursor = 0_usize;
    let marker = "secrets.";
    while let Some(pos) = workflow_text[cursor..].find(marker) {
        let absolute = cursor + pos;
        let start = absolute + marker.len();
        let tail = &workflow_text[start..];
        let name = tail
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .collect::<String>();
        let name_len = name.len();
        if !name.is_empty() {
            out.push(name);
        }
        cursor = start.saturating_add(name_len).saturating_add(1);
        if cursor >= workflow_text.len() {
            break;
        }
    }
    out.sort();
    out.dedup();
    out
}

fn run_ci_verify_gate(
    repo_root: &Path,
    gate: &str,
    options: CiVerifyRunOptions,
) -> Result<(String, i32), String> {
    let CiVerifyRunOptions {
        format,
        out,
        allow_subprocess,
        allow_git,
        allow_write,
        allow_network,
    } = options;
    let payload = match gate {
        "workflow-policy" => {
            let workflow = repo_root.join(".github/workflows/ci-pr.yml");
            let text = fs::read_to_string(&workflow)
                .map_err(|err| format!("read {} failed: {err}", workflow.display()))?;
            let mut errors = Vec::<String>::new();
            let lane_registry = load_ci_lanes_registry(repo_root)?;
            let lane_surface = load_ci_lane_surface(repo_root)?;
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
            let mut workflow_to_lanes =
                std::collections::BTreeMap::<String, std::collections::BTreeSet<String>>::new();
            for row in &lane_surface.lanes {
                workflow_to_lanes
                    .entry(row.workflow.clone())
                    .or_default()
                    .insert(row.lane.clone());
            }
            for workflow_path in workflow_to_lanes.keys() {
                let workflow_abs = repo_root.join(workflow_path);
                let workflow_text = fs::read_to_string(&workflow_abs)
                    .map_err(|err| format!("read {} failed: {err}", workflow_abs.display()))?;
                let workflow_value: serde_yaml::Value = serde_yaml::from_str(&workflow_text)
                    .map_err(|err| format!("parse {} failed: {err}", workflow_abs.display()))?;
                let top_env = workflow_value
                    .get("env")
                    .and_then(serde_yaml::Value::as_mapping)
                    .cloned()
                    .unwrap_or_default();
                let jobs = workflow_value
                    .get("jobs")
                    .and_then(serde_yaml::Value::as_mapping)
                    .cloned()
                    .unwrap_or_default();
                let workflow_lanes = workflow_to_lanes
                    .get(workflow_path.as_str())
                    .cloned()
                    .unwrap_or_default();
                for canonical_lane in workflow_lanes {
                    for required_env in lane_registry
                        .lanes
                        .iter()
                        .find(|lane| lane.id == canonical_lane)
                        .map(|lane| lane.required_env.clone())
                        .unwrap_or_default()
                    {
                        let declared = top_env
                            .iter()
                            .any(|(k, _)| k.as_str() == Some(required_env.as_str()))
                            || jobs.iter().any(|(_, job_value)| {
                                job_value
                                    .get("env")
                                    .and_then(serde_yaml::Value::as_mapping)
                                    .map(|env_map| {
                                        env_map
                                            .iter()
                                            .any(|(k, _)| k.as_str() == Some(required_env.as_str()))
                                    })
                                    .unwrap_or(false)
                            });
                        if !declared {
                            errors.push(format!(
                                "workflow `{workflow_path}` for lane `{canonical_lane}` is missing required env `{required_env}`"
                            ));
                        }
                    }
                }
                let referenced_lanes = scan_workflow_referenced_lane_ids(&workflow_text);
                for referenced_lane in referenced_lanes {
                    if lane_id_or_alias(&lane_registry, referenced_lane.as_str()).is_none() {
                        errors.push(format!(
                            "workflow `{workflow_path}` references unknown lane `{referenced_lane}`"
                        ));
                    }
                }
                let lane_domain = workflow_to_lanes
                    .get(workflow_path.as_str())
                    .and_then(|lanes| lanes.iter().next())
                    .map(|lane| lane_domain_from_id(lane.as_str()))
                    .unwrap_or("generic");
                let path_filters = workflow_value
                    .get("on")
                    .and_then(|on| on.get("pull_request"))
                    .and_then(|pr| pr.get("paths"))
                    .and_then(serde_yaml::Value::as_sequence)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect::<Vec<_>>();
                if !path_filters.is_empty() {
                    let allowed_prefixes = domain_path_prefixes(lane_domain);
                    let aligned = path_filters.iter().any(|entry| {
                        allowed_prefixes.iter().any(|prefix| entry.starts_with(prefix))
                    });
                    if !aligned {
                        errors.push(format!(
                            "workflow `{workflow_path}` path filters do not align with lane domain `{lane_domain}`"
                        ));
                    }
                }
                let secret_names = scan_workflow_secret_names(&workflow_text);
                if !secret_names.is_empty() {
                    let lane_allowed_for_secrets = workflow_to_lanes
                        .get(workflow_path.as_str())
                        .map(|lanes| {
                            lanes.iter().any(|lane| {
                                lane == "release-candidate"
                                    || lane == "ops-kind-integration"
                                    || lane == "dependency-lock"
                            })
                        })
                        .unwrap_or(false);
                    if !lane_allowed_for_secrets {
                        errors.push(format!(
                            "workflow `{workflow_path}` uses secrets but mapped lanes are not allowed for secret use"
                        ));
                    }
                    let secret_registry_path = repo_root.join("configs/ci/secret-registry.json");
                    if secret_registry_path.exists() {
                        let registry_text = fs::read_to_string(&secret_registry_path).map_err(|err| {
                            format!("read {} failed: {err}", secret_registry_path.display())
                        })?;
                        let registry_value: serde_json::Value =
                            serde_json::from_str(&registry_text).map_err(|err| {
                                format!("parse {} failed: {err}", secret_registry_path.display())
                            })?;
                        let allowed = registry_value["secrets"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .filter_map(|value| value.as_str().map(str::to_string))
                            .collect::<std::collections::BTreeSet<_>>();
                        for secret in secret_names {
                            if !allowed.contains(secret.as_str()) {
                                errors.push(format!(
                                    "workflow `{workflow_path}` references secret `{secret}` not listed in configs/ci/secret-registry.json"
                                ));
                            }
                        }
                    }
                }
            }
            let canonical_lane_ids = lane_registry
                .lanes
                .iter()
                .map(|lane| lane.id.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            for mapped_lane in lane_surface.lanes.iter().map(|row| row.lane.as_str()) {
                if !canonical_lane_ids.contains(mapped_lane) {
                    errors.push(format!(
                        "lane surface references lane `{mapped_lane}` not present in configs/ci/lanes.json"
                    ));
                }
            }
            let policy_drift = ci_registry_unplanned_entries(repo_root)?;
            errors.extend(
                policy_drift
                    .unplanned
                    .into_iter()
                    .map(|entry| format!("unplanned ci policy entry `{}`", entry.policy_id)),
            );
            errors.extend(policy_drift.uniqueness_errors);
            errors.extend(policy_drift.docs_errors);
            errors.extend(policy_drift.exception_errors);
            let lane_workflow_set = workflow_to_lanes.keys().cloned().collect::<std::collections::BTreeSet<_>>();
            let lint_rows = workflow_step_rows(repo_root)?;
            errors.extend(lint_rows.into_iter().filter(|row| !row.allowed).map(|row| {
                if lane_workflow_set.contains(row.workflow.as_str()) {
                    format!(
                        "workflow step `{}` in {}/{} is not matched by an allowed pattern or active allowlist",
                        row.step, row.workflow, row.job
                    )
                } else {
                    String::new()
                }
            }));
            errors.retain(|item| !item.is_empty());
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
            let git_ref = github_ref_from_process_env();
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
            serde_json::json!({
                "schema_version": 1,
                "kind": "ci_verify_docs_preview",
                "status": if build_code == 0 && site_payload["status"].as_str() == Some("pass") { "ok" } else { "failed" },
                "build_exit_code": build_code,
                "site_output": site_payload
            })
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
                    "ops/docker/**",
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

pub(super) fn run_workflows_command(quiet: bool, command: WorkflowsCommand) -> i32 {
    match command {
        WorkflowsCommand::Lanes { command } => match command {
            crate::cli::CiLanesCommand::List {
                repo_root,
                format,
                out,
            } => match resolve_repo_root(repo_root)
                .and_then(|root| render_ci_lanes_list(&root, format, out))
            {
                Ok((rendered, code)) => {
                    if !quiet && !rendered.is_empty() {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    }
                    code
                }
                Err(err) => {
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas ci lanes list failed: {err}");
                    1
                }
            },
            crate::cli::CiLanesCommand::Explain {
                lane_id,
                repo_root,
                format,
                out,
            } => match resolve_repo_root(repo_root)
                .and_then(|root| render_ci_lanes_explain(&root, &lane_id, format, out))
            {
                Ok((rendered, code)) => {
                    if !quiet && !rendered.is_empty() {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    }
                    code
                }
                Err(err) => {
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas ci lanes explain failed: {err}");
                    1
                }
            },
            crate::cli::CiLanesCommand::Validate {
                repo_root,
                format,
                out,
            } => match resolve_repo_root(repo_root)
                .and_then(|root| render_ci_lanes_validate(&root, format, out))
            {
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
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas ci lanes validate failed: {err}");
                    1
                }
            },
        },
        WorkflowsCommand::Simulate {
            repo_root,
            lane,
            matrix,
            format,
            out,
        } => match resolve_repo_root(repo_root)
            .and_then(|root| render_ci_simulate(&root, lane, matrix, format, out))
        {
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
                let _ = writeln!(io::stderr(), "bijux-dev-atlas ci simulate failed: {err}");
                1
            }
        },
        WorkflowsCommand::EnvContract { command } => match command {
            crate::cli::CiEnvContractCommand::Validate {
                repo_root,
                format,
                out,
            } => match resolve_repo_root(repo_root)
                .and_then(|root| render_ci_env_contract_validate(&root, format, out))
            {
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
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas ci env-contract validate failed: {err}");
                    1
                }
            },
        },
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
            severity: None,
            mode: None,
            tag: None,
            name: None,
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
            severity: None,
            mode: None,
            tag: None,
            name: None,
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
            run_ci_verify_gate(&root, &gate, CiVerifyRunOptions {
                format,
                out,
                allow_subprocess,
                allow_git,
                allow_write,
                allow_network,
            })
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

pub(super) fn run_gates_command(quiet: bool, command: GatesCommand) -> i32 {
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
            severity: None,
            mode: None,
            tag: None,
            name: None,
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
            severity: None,
            mode: None,
            tag: None,
            name: None,
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
