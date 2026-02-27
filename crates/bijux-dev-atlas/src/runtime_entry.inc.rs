use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

#[cfg(test)]
pub(crate) use crate::cli::Cli;
use crate::cli::{
    ConfigsCommand, ConfigsCommonArgs, DemoCommand, DocsCommand, DocsCommonArgs, DomainArg,
    FormatArg, GatesCommand, OpsCommand, OpsCommonArgs, OpsRenderTarget, OpsStatusTarget,
    WorkflowsCommand,
};
use bijux_dev_atlas::adapters::{Capabilities, RealFs, RealProcessRunner, WorkspaceRoot};
use bijux_dev_atlas::core::ops_inventory::{ops_inventory_summary, validate_ops_inventory};
use bijux_dev_atlas::core::{
    exit_code_for_report, explain_output, load_registry, registry_doctor, render_json,
    render_jsonl, render_text_with_durations, run_checks, select_checks, RunOptions, RunRequest,
    Selectors,
};
use bijux_dev_atlas::model::{CheckId, CheckSpec, DomainId, RunId, SuiteId, Tag};
pub(crate) use build_commands::run_build_command;
#[cfg(test)]
pub(crate) use configs_commands::parse_config_file;
pub(crate) use configs_commands::{
    configs_context, configs_diff_payload, configs_lint_payload, configs_validate_payload,
    run_configs_command,
};
pub(crate) use control_plane_commands::{
    run_capabilities_command, run_contracts_command, run_docker_command,
    run_help_inventory_command, run_policies_command, run_print_boundaries_command,
    run_print_policies, run_version_command,
};
#[cfg(test)]
pub(crate) use docs_commands::mkdocs_nav_refs;
pub(crate) use docs_commands::{
    docs_context, docs_links_payload, docs_validate_payload, walk_files_local,
};
pub(crate) use docs_commands::{docs_lint_payload, run_docs_command};
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

fn resolve_repo_root(arg: Option<PathBuf>) -> Result<PathBuf, String> {
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


include!("runtime_entry_checks.inc.rs");

pub(crate) fn run() -> i32 {
    cli::run()
}
