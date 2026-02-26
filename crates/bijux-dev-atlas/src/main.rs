// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

#[path = "commands/build.rs"]
mod build_commands;
mod cli;
#[path = "commands/configs.rs"]
mod configs_commands;
#[path = "commands/control_plane.rs"]
mod control_plane_commands;
#[path = "commands/dispatch.rs"]
mod dispatch;
mod docs_command_runtime;
#[path = "commands/docs.rs"]
mod docs_commands;
#[cfg(test)]
mod main_tests;
mod ops_command_support;
#[path = "commands/ops.rs"]
mod ops_commands;
mod ops_runtime_execution;
mod ops_support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::cli::{
    ConfigsCommand, ConfigsCommonArgs, DocsCommand, DocsCommonArgs, DomainArg, FormatArg,
    GatesCommand, OpsCommand, OpsCommonArgs, OpsGenerateCommand, OpsPinsCommand, OpsRenderTarget,
    OpsStatusTarget, WorkflowsCommand,
};
#[cfg(test)]
pub(crate) use crate::cli::Cli;
use bijux_dev_atlas_adapters::{Capabilities, RealFs, RealProcessRunner};
use bijux_dev_atlas_core::ops_inventory::{ops_inventory_summary, validate_ops_inventory};
use bijux_dev_atlas_core::{
    exit_code_for_report, explain_output, load_registry, registry_doctor, render_json,
    render_jsonl, render_text_with_durations, run_checks, select_checks, RunOptions, RunRequest,
    Selectors,
};
use bijux_dev_atlas_model::{CheckId, CheckSpec, DomainId, RunId, SuiteId, Tag};
pub(crate) use build_commands::run_build_command;
#[cfg(test)]
pub(crate) use configs_commands::parse_config_file;
pub(crate) use configs_commands::{
    configs_context, configs_diff_payload, configs_lint_payload, configs_validate_payload,
    run_configs_command,
};
pub(crate) use control_plane_commands::{
    run_capabilities_command, run_docker_command, run_help_inventory_command, run_policies_command,
    run_print_boundaries_command, run_print_policies, run_version_command,
};
pub(crate) use docs_command_runtime::{docs_lint_payload, run_docs_command};
#[cfg(test)]
pub(crate) use docs_commands::mkdocs_nav_refs;
pub(crate) use docs_commands::{
    docs_context, docs_links_payload, docs_validate_payload, walk_files_local,
};
pub(crate) use ops_commands::{emit_payload, normalize_tool_version_with_regex, run_ops_command};
pub(crate) use ops_support::{
    OpsCommandError, OpsFs, OpsProcess, StackProfile, StackProfiles, SurfacesInventory,
    ToolDefinition, ToolchainInventory,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use sha2::{Digest, Sha256};

const UMBRELLA_MIN_VERSION: &str = "0.1.0";
const UMBRELLA_MAX_EXCLUSIVE_VERSION: &str = "0.2.0";

impl From<DomainArg> for DomainId {
    fn from(value: DomainArg) -> Self {
        match value {
            DomainArg::Root => Self::Root,
            DomainArg::Workflows => Self::Workflows,
            DomainArg::Configs => Self::Configs,
            DomainArg::Docker => Self::Docker,
            DomainArg::Crates => Self::Crates,
            DomainArg::Ops => Self::Ops,
            DomainArg::Repo => Self::Repo,
            DomainArg::Docs => Self::Docs,
            DomainArg::Make => Self::Make,
        }
    }
}

fn discover_repo_root(start: &Path) -> Result<PathBuf, String> {
    let mut current = start.canonicalize().map_err(|err| err.to_string())?;
    loop {
        if current.join("ops/inventory/registry.toml").exists() {
            return Ok(current);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return Err(
                "could not discover repo root (no ops/inventory/registry.toml found)".to_string(),
            );
        }
    }
}

fn resolve_repo_root(arg: Option<PathBuf>) -> Result<PathBuf, String> {
    match arg {
        Some(path) => discover_repo_root(&path),
        None => {
            let cwd = std::env::current_dir().map_err(|err| err.to_string())?;
            discover_repo_root(&cwd)
        }
    }
}

pub(crate) fn plugin_metadata_json() -> String {
    serde_json::json!({
        "schema_version": "v1",
        "name": "bijux-dev-atlas",
        "version": env!("CARGO_PKG_VERSION"),
        "build_hash": "dev",
        "compatible_umbrella": format!(">={UMBRELLA_MIN_VERSION},<{UMBRELLA_MAX_EXCLUSIVE_VERSION}"),
        "compatible_umbrella_min": UMBRELLA_MIN_VERSION,
        "compatible_umbrella_max_exclusive": UMBRELLA_MAX_EXCLUSIVE_VERSION,
    })
    .to_string()
}

fn parse_selectors(
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
) -> Result<Selectors, String> {
    let normalized_suite = suite
        .as_deref()
        .map(normalize_suite_name)
        .transpose()?
        .map(std::string::ToString::to_string);
    Ok(Selectors {
        suite: normalized_suite
            .as_ref()
            .map(|v| SuiteId::parse(v))
            .transpose()?,
        domain: domain.map(Into::into),
        tag: tag.map(|v| Tag::parse(&v)).transpose()?,
        id_glob: id,
        include_internal,
        include_slow,
    })
}

fn normalize_suite_name(raw: &str) -> Result<&str, String> {
    match raw {
        "ci-fast" => Ok("ci_fast"),
        "ci" => Ok("ci"),
        "local" => Ok("local"),
        "deep" => Ok("deep"),
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
                        println!("{rendered}");
                    } else {
                        eprintln!("{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas workflows validate failed: {err}");
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
                    println!("{rendered}");
                }
                code
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas gates list failed: {err}");
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
                        println!("{rendered}");
                    } else {
                        eprintln!("{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas gates run failed: {err}");
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

pub(crate) fn run_check_registry_doctor(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let report = registry_doctor(&root);
    let status = if report.errors.is_empty() {
        "ok"
    } else {
        "failed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "repo_root": root.display().to_string(),
        "errors": report.errors,
    });
    let rendered = match format {
        FormatArg::Text => format!(
            "status: {status}\nerrors: {}",
            payload["errors"].as_array().map_or(0, Vec::len)
        ),
        FormatArg::Json => serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => serde_json::to_string(&payload).map_err(|err| err.to_string())?,
    };
    write_output_if_requested(out, &rendered)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn main() {
    std::process::exit(cli::run());
}
