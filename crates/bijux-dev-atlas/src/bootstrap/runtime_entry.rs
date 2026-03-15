pub(crate) use std::fs;
pub(crate) use std::io::{self, IsTerminal, Write};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::process::Command as ProcessCommand;

#[cfg(test)]
pub(crate) use crate::cli::Cli;
pub(crate) use crate::cli::{
    CheckModeArg, CheckSeverityArg, ConfigsCommand, ConfigsCommonArgs, DemoCommand, DocsCommand,
    DocsCommonArgs, DomainArg, FormatArg, GatesCommand, OpsCommand, OpsCommonArgs, OpsRenderTarget,
    OpsStatusTarget, WorkflowsCommand,
};
pub(crate) use crate::api_commands::run_api_command;
pub(crate) use crate::artifacts_commands::run_artifacts_command;
pub(crate) use crate::audit_commands::run_audit_command;
pub(crate) use bijux_dev_atlas::core::ops_inventory::{ops_inventory_summary, validate_ops_inventory};
pub(crate) use bijux_dev_atlas::core::{
    exit_code_for_report, explain_output, load_registry, registry_doctor, render_json,
    render_jsonl, render_text_with_durations, run_checks, select_checks, RunOptions, RunRequest,
    Selectors,
};
pub(crate) use bijux_dev_atlas::model::{CheckId, CheckSpec, DomainId, RunId, SuiteId, Tag};
pub(crate) use bijux_dev_atlas::model::{CheckMode, CheckSeverity};
pub(crate) use bijux_dev_atlas::registry::{CheckCatalog, CheckCatalogEntry};
pub(crate) use bijux_dev_atlas::runtime::{Capabilities, RealFs, RealProcessRunner, WorkspaceRoot};
pub(crate) use bijux_dev_atlas::ui::terminal::report::render_check_run_report;
pub(crate) use crate::build_commands::run_build_command;
pub(crate) use crate::commands_data::run_data_command;
#[cfg(test)]
pub(crate) use crate::configs_commands::parse_config_file;
pub(crate) use crate::configs_commands::{
    configs_context, configs_diff_payload, configs_lint_payload, configs_validate_payload,
    run_configs_command,
};
pub(crate) use crate::control_plane_commands::{
    run_capabilities_command, run_docker_command, run_help_inventory_command,
    run_policies_command, run_print_boundaries_command, run_print_policies,
    run_version_command,
};
#[cfg(test)]
pub(crate) use crate::docs_commands::mkdocs_nav_refs;
pub(crate) use crate::docs_commands::{
    docs_context, docs_links_payload, docs_validate_payload, walk_files_local,
};
pub(crate) use crate::docs_commands::{docs_lint_payload, run_docs_command};
pub(crate) use crate::drift_commands::run_drift_command;
pub(crate) use crate::governance_commands::run_governance_command;
pub(crate) use crate::governance_commands::run_registry_command;
pub(crate) use crate::invariants_commands::run_invariants_command;
pub(crate) use crate::load_commands::run_load_command;
pub(crate) use crate::make_commands::run_make_command;
pub(crate) use crate::migrations_commands::run_migrations_command;
pub(crate) use crate::observe_commands::run_observe_command;
pub(crate) use crate::ops_commands::{
    emit_payload, normalize_tool_version_with_regex, run_ops_command,
};
pub(crate) use crate::ops_support::{
    OpsCommandError, OpsFs, OpsProcess, StackProfile, StackProfiles, SurfacesInventory,
    ToolDefinition, ToolchainInventory,
};
pub(crate) use crate::perf_commands::run_perf_command;
pub(crate) use regex::Regex;
pub(crate) use crate::release_commands::run_release_command;
pub(crate) use crate::reproduce_commands::run_reproduce_command;
pub(crate) use crate::runtime_commands::run_runtime_command;
pub(crate) use crate::security_commands::run_security_command;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use serde_yaml::Value as YamlValue;
pub(crate) use sha2::{Digest, Sha256};
pub(crate) use crate::suites_commands::{
    run_registry_check_by_id, run_registry_contract_by_id, run_suites_command,
};
pub(crate) use crate::system_commands::run_system_command;
pub(crate) use crate::tutorials_commands::run_tutorials_command;

const UMBRELLA_MIN_VERSION: &str = "0.3.0";
const UMBRELLA_MAX_EXCLUSIVE_VERSION: &str = "0.4.0";

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

pub(crate) fn resolve_repo_root(arg: Option<PathBuf>) -> Result<PathBuf, String> {
    WorkspaceRoot::from_cli_or_cwd(arg)
        .map(WorkspaceRoot::into_inner)
        .map_err(|err| err.to_string())
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn parse_selectors(
    suite: Option<String>,
    domain: Option<DomainArg>,
    severity: Option<CheckSeverityArg>,
    mode: Option<CheckModeArg>,
    tag: Option<String>,
    name: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
) -> Result<Selectors, String> {
    if let Some(pattern) = id.as_deref() {
        validate_id_glob_pattern(pattern)?;
    }
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
        severity: severity.map(|value| match value {
            CheckSeverityArg::Blocker => CheckSeverity::Blocker,
            CheckSeverityArg::High => CheckSeverity::High,
            CheckSeverityArg::Medium => CheckSeverity::Medium,
            CheckSeverityArg::Low => CheckSeverity::Low,
            CheckSeverityArg::Info => CheckSeverity::Info,
        }),
        mode: mode.map(|value| match value {
            CheckModeArg::Static => CheckMode::Static,
            CheckModeArg::Effect => CheckMode::Effect,
        }),
        tag: tag.map(|v| Tag::parse(&v)).transpose()?,
        title_substring: name,
        id_glob: id,
        include_internal,
        include_slow,
    })
}

fn validate_id_glob_pattern(pattern: &str) -> Result<(), String> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Err("invalid wildcard pattern ``: pattern cannot be empty".to_string());
    }
    if let Some(ch) = trimmed
        .chars()
        .find(|ch| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':' | '*' | '?')))
    {
        return Err(format!(
            "invalid wildcard pattern `{trimmed}`: unsupported metacharacter `{ch}`; use `*` and `?` only"
        ));
    }
    Ok(())
}

pub(crate) fn run_demo_command(quiet: bool, command: DemoCommand) -> i32 {
    let result = (|| -> Result<(String, i32), String> {
        match command {
            DemoCommand::Quickstart(args) => {
                let repo_root = resolve_repo_root(args.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "name": "demo_quickstart",
                    "text": "quickstart execution plan",
                    "duration_budget_minutes": 3,
                    "steps_budget": 4,
                    "steps": [
                        {"order": 1, "name": "stack_up", "command": "bijux dev atlas ops stack up --profile kind --allow-subprocess --allow-write --format json"},
                        {"order": 2, "name": "ingest_fixture", "command": "bijux atlas ingest run --input ops/datasets/fixtures/tiny --format json"},
                        {"order": 3, "name": "query_smoke", "command": "curl -fsS http://127.0.0.1:8080/api/v1/genes?limit=1"},
                        {"order": 4, "name": "metrics_smoke", "command": "curl -fsS http://127.0.0.1:8080/metrics"}
                    ],
                    "repo_root": repo_root.display().to_string()
                });
                Ok((emit_payload(args.format, args.out, &payload)?, 0))
            }
        }
    })();
    match result {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas demo failed: {err}");
            1
        }
    }
}

include!("runtime_entry_checks_surface.rs");
include!("runtime_entry_checks_governance.rs");

pub(crate) fn run() -> i32 {
    crate::cli::run()
}
