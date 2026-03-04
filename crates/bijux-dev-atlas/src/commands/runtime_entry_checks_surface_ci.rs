use super::*;
#[path = "runtime_entry_checks_surface_ci_registry.rs"]
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
    let mut errors = Vec::<String>::new();
    let mut seen = std::collections::BTreeSet::<String>::new();
    let mut ordered = Vec::<String>::new();
    for lane in &registry.lanes {
        if !seen.insert(lane.id.clone()) {
            errors.push(format!("duplicate lane id `{}`", lane.id));
        }
        ordered.push(lane.id.clone());
        if lane.description.trim().is_empty() {
            errors.push(format!("lane `{}` is missing description", lane.id));
        }
        if lane.artifacts_expected.is_empty() {
            errors.push(format!("lane `{}` must declare artifacts_expected", lane.id));
        }
        if !lane.command.starts_with("bijux dev atlas ") {
            errors.push(format!(
                "lane `{}` command must start with `bijux dev atlas `",
                lane.id
            ));
        }
    }
    let mut sorted = ordered.clone();
    sorted.sort();
    if ordered != sorted {
        errors.push("lane ids must be deterministically ordered".to_string());
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
    for row in selected {
        let mut missing = Vec::<String>::new();
        for artifact in &row.artifacts_expected {
            if !repo_root.join(artifact).exists() {
                missing.push(artifact.clone());
            }
        }
        if !missing.is_empty() {
            missing_artifacts.extend(missing.iter().map(|item| format!("{}:{item}", row.id)));
        }
        rows.push(serde_json::json!({
            "lane": row.id,
            "mode": row.mode,
            "command": row.command,
            "artifacts_expected": row.artifacts_expected,
            "missing_artifacts": missing,
            "status": if missing.is_empty() { "ok" } else { "missing_artifacts" }
        }));
    }
    rows.sort_by(|a, b| a["lane"].as_str().cmp(&b["lane"].as_str()));
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "ci_simulate",
        "status": if missing_artifacts.is_empty() { "ok" } else { "failed" },
        "rows": rows,
        "summary": {
            "lanes": rows.len(),
            "missing_artifacts": missing_artifacts.len()
        },
        "missing_artifacts": missing_artifacts
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
