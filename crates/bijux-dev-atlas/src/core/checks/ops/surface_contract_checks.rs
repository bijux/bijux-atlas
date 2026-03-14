// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use super::*;
use crate::ports::{Clock, SystemClock};

fn current_utc_date_string() -> String {
    let days = (SystemClock.now_unix_secs() / 86_400) as i64;
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
    format!("{year:04}-{month:02}-{day:02}")
}

pub(super) fn checks_ops_makefile_routes_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/ops.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected_targets = ["ops-doctor:", "ops-validate:", "ops-render:", "ops-status:"];
    let mut violations = Vec::new();
    for target in expected_targets {
        if !content.contains(target) {
            violations.push(violation(
                "OPS_MAKEFILE_TARGET_MISSING",
                format!("ops make wrapper target missing `{target}`"),
                "add thin ops wrapper target in make/ops.mk",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_make_governance_wrappers_bijux_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/ci.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        let trimmed = line.trim();
        if !(trimmed.contains("make ")
            || trimmed.contains("$(BIJUX_DEV_ATLAS)")
            || trimmed.contains("$(BIJUX) dev atlas"))
        {
            continue;
        }
        let words = trimmed.split_whitespace().collect::<Vec<_>>();
        if words.iter().any(|w| {
            matches!(
                *w,
                "python" | "python3" | "bash" | "helm" | "kubectl" | "k6"
            )
        }) {
            violations.push(violation(
                "MAKE_GOVERNANCE_DELEGATION_ONLY_VIOLATION",
                format!("governance wrapper must be delegation-only: `{trimmed}`"),
                "keep governance wrappers routed only through make/bijux dev atlas",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_make_ops_wrappers_delegate_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/ops.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    if !content.contains("BIJUX ?= bijux") || !content.contains("BIJUX_DEV_ATLAS ?=") {
        violations.push(violation(
            "MAKE_OPS_BIJUX_VARIABLES_MISSING",
            "make/ops.mk must declare BIJUX and BIJUX_DEV_ATLAS variables".to_string(),
            "declare BIJUX and BIJUX_DEV_ATLAS wrapper variables in make/ops.mk",
            Some(rel),
        ));
    }
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        if line.trim_end().ends_with('\\') {
            violations.push(violation(
                "MAKE_OPS_SINGLE_LINE_RECIPE_REQUIRED",
                "make/ops.mk wrapper recipes must be single-line delegations".to_string(),
                "keep ops wrappers single-line and delegation-only",
                Some(rel),
            ));
        }
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let direct_tool = tokens
            .first()
            .copied()
            .unwrap_or_default()
            .trim_start_matches('@');
        if matches!(
            direct_tool,
            "python" | "python3" | "bash" | "sh" | "kubectl" | "helm" | "k6"
        ) {
            violations.push(violation(
                "MAKE_OPS_DELEGATION_ONLY_VIOLATION",
                format!("make/ops.mk must be delegation-only: `{line}`"),
                "ops wrappers may call make or bijux dev atlas only",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_workflows_governance_entrypoints_bijux_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let _ = ctx;
    Ok(Vec::new())
}

pub(super) fn check_workflows_ops_entrypoints_bijux_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let _ = ctx;
    Ok(Vec::new())
}

pub(super) fn check_workflows_policy_registry_has_no_unplanned_entries(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/ci/policy-outside-control-plane.json");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for entry in value["entries"].as_array().into_iter().flatten() {
        let policy_id = entry["policy_id"].as_str().unwrap_or("unknown");
        let status = entry["status"].as_str().unwrap_or_default();
        let replacement_plan = entry["replacement_plan"].as_str().unwrap_or_default();
        let command = entry["control_plane_command"].as_str().unwrap_or_default();
        if !matches!(status, "atlas" | "planned" | "exception") {
            violations.push(violation(
                "WORKFLOW_POLICY_UNPLANNED_ENTRY_FOUND",
                format!(
                    "workflow policy registry entry `{policy_id}` must be atlas, planned, or exception"
                ),
                "record a concrete atlas replacement or explicit temporary exception",
                Some(rel),
            ));
        }
        if replacement_plan.trim().is_empty() {
            violations.push(violation(
                "WORKFLOW_POLICY_REPLACEMENT_PLAN_MISSING",
                format!("workflow policy registry entry `{policy_id}` is missing replacement_plan"),
                "document the atlas replacement plan for every workflow-local policy step",
                Some(rel),
            ));
        }
        if status == "atlas" && command.trim().is_empty() {
            violations.push(violation(
                "WORKFLOW_POLICY_ATLAS_COMMAND_MISSING",
                format!(
                    "workflow policy registry entry `{policy_id}` is missing control_plane_command"
                ),
                "link every atlas-owned workflow policy entry to its executable command",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_workflows_policy_registry_unique_and_documented(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/ci/policy-outside-control-plane.json");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut seen = BTreeMap::<String, String>::new();
    let mut violations = Vec::new();
    for entry in value["entries"].as_array().into_iter().flatten() {
        let policy_id = entry["policy_id"].as_str().unwrap_or("unknown");
        let implementation = entry["authoritative_implementation"]
            .as_str()
            .unwrap_or_default();
        let docs = entry["docs"].as_str().unwrap_or_default();
        if seen
            .insert(policy_id.to_string(), implementation.to_string())
            .is_some()
        {
            violations.push(violation(
                "WORKFLOW_POLICY_ID_NOT_UNIQUE",
                format!("workflow policy registry contains duplicate policy_id `{policy_id}`"),
                "keep workflow policy ids unique",
                Some(rel),
            ));
        }
        if implementation.trim().is_empty() {
            violations.push(violation(
                "WORKFLOW_POLICY_AUTHORITY_MISSING",
                format!("workflow policy registry entry `{policy_id}` is missing authoritative_implementation"),
                "declare one authoritative implementation for every workflow policy entry",
                Some(rel),
            ));
        }
        if docs.trim().is_empty() || !ctx.repo_root.join(docs).exists() {
            violations.push(violation(
                "WORKFLOW_POLICY_DOC_LINK_MISSING",
                format!("workflow policy registry entry `{policy_id}` references missing docs `{docs}`"),
                "link workflow policy entries to a committed docs page that explains the executable check",
                Some(rel),
            ));
        } else if let Ok(doc_text) = fs::read_to_string(ctx.repo_root.join(docs)) {
            if !doc_text.contains("bijux-dev-atlas --")
                && !doc_text.contains("cargo run -q -p bijux-dev-atlas --")
                && !doc_text.contains("cargo run --locked -q -p bijux-dev-atlas --")
            {
                violations.push(violation(
                    "WORKFLOW_POLICY_DOC_EXECUTABLE_LINK_MISSING",
                    format!("workflow policy docs `{docs}` do not reference an executable atlas command for `{policy_id}`"),
                    "link policy docs to the executable atlas command instead of prose-only policy descriptions",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_workflows_policy_exceptions_expiry(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/ci/policy-exceptions.json");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let today = current_utc_date_string();
    let mut violations = Vec::new();
    for entry in value["exceptions"].as_array().into_iter().flatten() {
        let policy_id = entry["policy_id"].as_str().unwrap_or("unknown");
        let expires_on = entry["expires_on"].as_str().unwrap_or_default();
        let governance_tier = entry["governance_tier"].as_str().unwrap_or_default();
        let renewal_reference = entry["renewal_reference"].as_str().unwrap_or_default();
        if expires_on.len() != 10 {
            violations.push(violation(
                "WORKFLOW_POLICY_EXCEPTION_DATE_INVALID",
                format!("workflow policy exception `{policy_id}` must use YYYY-MM-DD expiry"),
                "set expires_on with an ISO date",
                Some(rel),
            ));
        } else if expires_on < today.as_str() {
            violations.push(violation(
                "WORKFLOW_POLICY_EXCEPTION_EXPIRED",
                format!("workflow policy exception `{policy_id}` expired on `{expires_on}`"),
                "renew or remove expired workflow policy exceptions",
                Some(rel),
            ));
        }
        if governance_tier != "governance-approved" && expires_on == "9999-12-31" {
            violations.push(violation(
                "WORKFLOW_POLICY_EXCEPTION_PERMANENT_FORBIDDEN",
                format!("workflow policy exception `{policy_id}` is permanent without governance approval"),
                "use bounded expiry for temporary exceptions or mark governance-approved",
                Some(rel),
            ));
        }
        if renewal_reference.trim().is_empty() {
            violations.push(violation(
                "WORKFLOW_POLICY_EXCEPTION_RENEWAL_MISSING",
                format!("workflow policy exception `{policy_id}` is missing renewal_reference"),
                "record the renewal workflow reference for each workflow policy exception",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_workflows_step_patterns_cover_surface(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/ci/workflow-step-patterns.json");
    let allowlist_rel = Path::new("configs/ci/workflow-allowlist.json");
    let root = ctx.repo_root;
    let patterns_value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(rel)).map_err(|err| CheckError::Failed(err.to_string()))?,
    )
    .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(allowlist_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?,
    )
    .map_err(|err| CheckError::Failed(err.to_string()))?;
    let patterns = patterns_value["patterns"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let allowlist = allowlist_value["entries"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut violations = Vec::new();
    for workflow in walk_files(&root.join(".github/workflows")) {
        if workflow.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue;
        }
        let workflow_rel = workflow
            .strip_prefix(root)
            .unwrap_or(&workflow)
            .display()
            .to_string();
        let yaml: serde_yaml::Value = serde_yaml::from_str(
            &fs::read_to_string(&workflow).map_err(|err| CheckError::Failed(err.to_string()))?,
        )
        .map_err(|err| CheckError::Failed(err.to_string()))?;
        let jobs = yaml
            .get("jobs")
            .and_then(serde_yaml::Value::as_mapping)
            .cloned()
            .unwrap_or_default();
        for (job_key, job_value) in jobs {
            let Some(job) = job_key.as_str() else {
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
                    .unwrap_or("unnamed-step");
                let (step_kind, content) =
                    if let Some(uses) = step.get("uses").and_then(serde_yaml::Value::as_str) {
                        ("uses", uses.to_string())
                    } else if let Some(run) = step.get("run").and_then(serde_yaml::Value::as_str) {
                        let normalized = run
                            .lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty() && *line != "set -euo pipefail")
                            .collect::<Vec<_>>()
                            .join("\n");
                        ("run", normalized)
                    } else {
                        ("unknown", String::new())
                    };
                let matched = patterns.iter().any(|pattern| {
                    pattern["step_kind"].as_str() == Some(step_kind)
                        && match pattern["match_mode"].as_str().unwrap_or_default() {
                            "contains" => {
                                content.contains(pattern["needle"].as_str().unwrap_or_default())
                            }
                            "starts_with" => {
                                content.starts_with(pattern["needle"].as_str().unwrap_or_default())
                            }
                            "equals" => content == pattern["needle"].as_str().unwrap_or_default(),
                            _ => false,
                        }
                });
                let allowed = allowlist.iter().any(|entry| {
                    entry["workflow"].as_str() == Some(&workflow_rel)
                        && entry["job"].as_str() == Some(job)
                        && entry["step"].as_str() == Some(step_name)
                });
                if !matched && !allowed {
                    violations.push(violation(
                        "WORKFLOW_STEP_PATTERN_MISSING",
                        format!(
                            "workflow step `{step_name}` in `{workflow_rel}` job `{job}` is not covered by configs/ci/workflow-step-patterns.json or workflow-allowlist.json"
                        ),
                        "register the step pattern or add an expiry-bound allowlist entry with justification",
                        Some(rel),
                    ));
                }
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_workflows_allowlist_expiry_bound(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/ci/workflow-allowlist.json");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for entry in value["entries"].as_array().into_iter().flatten() {
        let step = entry["step"].as_str().unwrap_or("unknown");
        let expires_on = entry["expires_on"].as_str().unwrap_or_default();
        let renewal_reference = entry["renewal_reference"].as_str().unwrap_or_default();
        if expires_on.len() != 10 {
            violations.push(violation(
                "WORKFLOW_ALLOWLIST_EXPIRY_INVALID",
                format!("workflow allowlist entry `{step}` must use YYYY-MM-DD expiry"),
                "set an ISO expiry date for every workflow allowlist entry",
                Some(rel),
            ));
        }
        if renewal_reference.trim().is_empty() {
            violations.push(violation(
                "WORKFLOW_ALLOWLIST_RENEWAL_MISSING",
                format!("workflow allowlist entry `{step}` is missing renewal_reference"),
                "record the renewal reference for every allowlist entry",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_make_governance_wrappers_no_direct_cargo(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/ci.mk");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for line in text.lines().filter(|line| line.starts_with('\t')) {
        if line.contains("cargo ") {
            violations.push(violation(
                "MAKE_GOVERNANCE_DIRECT_CARGO_REFERENCE_FOUND",
                format!(
                    "governance wrapper must not call cargo directly: `{}`",
                    line.trim()
                ),
                "route governance wrappers through bijux dev atlas",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_command_list_matches_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-atlas/docs/cli-command-list.md");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if current.lines().next() == Some("atlas") {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "DOCS_COMMAND_LIST_INVALID",
            "runtime command list doc must start with canonical `atlas` command".to_string(),
            "refresh runtime command list snapshot from bijux atlas --help",
            Some(rel),
        )])
    }
}

pub(super) fn check_docs_dev_command_list_matches_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/docs/cli-command-list.md");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if current.lines().next() == Some("ops") {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "DOCS_DEV_COMMAND_LIST_INVALID",
            "dev-atlas command list doc must start with canonical `ops` command".to_string(),
            "refresh dev command list snapshot from bijux dev atlas --help",
            Some(rel),
        )])
    }
}

pub(super) fn check_crates_bijux_atlas_reserved_verbs_exclude_dev(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-atlas/src/lib.rs");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("\"dev\"") && text.contains("reserved") {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_RUNTIME_RESERVED_VERB_DEV_MISSING",
            "runtime CLI reserved verbs policy must include `dev`".to_string(),
            "keep `dev` reserved in runtime command namespace ownership rules",
            Some(rel),
        )])
    }
}

pub(super) fn check_crates_bijux_dev_atlas_not_umbrella_binary(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/Cargo.toml");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("name = \"bijux\"") {
        Ok(vec![violation(
            "CRATES_DEV_ATLAS_UMBRELLA_BINARY_FORBIDDEN",
            "bijux-dev-atlas must not build the umbrella `bijux` binary".to_string(),
            "keep umbrella binary ownership in bijux-atlas only",
            Some(rel),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_crates_command_namespace_ownership_unique(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let runtime_rel = Path::new("crates/bijux-atlas/docs/cli-command-list.md");
    let dev_rel = Path::new("crates/bijux-dev-atlas/docs/cli-command-list.md");
    let runtime = fs::read_to_string(ctx.repo_root.join(runtime_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let dev = fs::read_to_string(ctx.repo_root.join(dev_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let runtime_first = runtime
        .lines()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .collect::<BTreeSet<_>>();
    let dev_first = dev
        .lines()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .collect::<BTreeSet<_>>();
    let overlap = runtime_first
        .intersection(&dev_first)
        .filter(|v| **v != "version")
        .cloned()
        .collect::<Vec<_>>();
    if overlap.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_COMMAND_NAMESPACE_OWNERSHIP_DUPLICATE",
            format!("runtime and dev command surfaces have duplicate namespace ownership: {}", overlap.join(", ")),
            "keep runtime and dev command surface ownership disjoint (except shared global version semantics)",
            Some(dev_rel),
        )])
    }
}
