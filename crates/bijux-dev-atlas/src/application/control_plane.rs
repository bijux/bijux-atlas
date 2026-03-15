// SPDX-License-Identifier: Apache-2.0

use crate::cli::PoliciesCommand;
use crate::*;
use bijux_dev_atlas::model::CONTRACT_SCHEMA_VERSION;
use bijux_dev_atlas::policies::{canonical_policy_json, DevAtlasPolicySet};
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub(crate) fn run_policies_command(quiet: bool, command: PoliciesCommand) -> i32 {
    let result = match command {
        PoliciesCommand::List {
            repo_root,
            format,
            out,
        } => run_policies_list(repo_root, format, out),
        PoliciesCommand::Explain {
            policy_id,
            repo_root,
            format,
            out,
        } => run_policies_explain(policy_id, repo_root, format, out),
        PoliciesCommand::Report {
            repo_root,
            format,
            out,
        } => run_policies_report(repo_root, format, out),
        PoliciesCommand::Print {
            repo_root,
            format,
            out,
        } => run_policies_print(repo_root, format, out),
        PoliciesCommand::Validate {
            repo_root,
            format,
            out,
        } => run_policies_validate(repo_root, format, out),
    };
    match result {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas policies failed: {err}");
            1
        }
    }
}

#[path = "control_plane/docker.rs"]
mod docker;
#[path = "control_plane/policies.rs"]
mod policies;
pub(crate) use docker::run_docker_command;
use policies::{run_policies_explain, run_policies_list, run_policies_report};
pub(crate) fn run_print_boundaries_command() -> Result<(String, i32), String> {
    let payload = serde_json::json!({
        "schema_version": CONTRACT_SCHEMA_VERSION,
        "effects": [
            {"id": "fs_read", "default_allowed": true, "description": "read repository files"},
            {"id": "fs_write", "default_allowed": false, "description": "write files under artifacts only"},
            {"id": "subprocess", "default_allowed": false, "description": "execute external processes"},
            {"id": "git", "default_allowed": false, "description": "invoke git commands"},
            {"id": "network", "default_allowed": false, "description": "perform network requests"}
        ],
        "text": "effect boundaries printed"
    });
    Ok((
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        0,
    ))
}

pub(crate) fn run_print_policies(repo_root: Option<PathBuf>) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let rendered = canonical_policy_json(&policies.to_document()).map_err(|err| err.to_string())?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_print(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let rendered = match format {
        FormatArg::Text => format!(
            "status: ok\nschema_version: {:?}\ncompatibility_rules: {}\ndocumented_defaults: {}",
            doc.schema_version,
            doc.compatibility.len(),
            doc.documented_defaults.len()
        ),
        FormatArg::Json => serde_json::to_string_pretty(&doc).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => serde_json::to_string(&doc).map_err(|err| err.to_string())?,
    };
    write_output_if_requested(out, &rendered)?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_validate(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "repo_root": root.display().to_string(),
        "policy_schema_version": doc.schema_version,
        "compatibility_rules": doc.compatibility.len(),
        "documented_defaults": doc.documented_defaults.len(),
        "capabilities": {
            "fs_write": false,
            "subprocess": false,
            "network": false,
            "git": false
        }
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_capabilities_command(
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "text": "capabilities default-deny; commands require explicit effect flags",
        "defaults": {
            "fs_write": false,
            "subprocess": false,
            "network": false,
            "git": false
        },
        "rules": [
            {"effect": "fs_write", "policy": "explicit flag required", "flags": ["--allow-write"]},
            {"effect": "subprocess", "policy": "explicit flag required", "flags": ["--allow-subprocess"]},
            {"effect": "network", "policy": "explicit flag required", "flags": ["--allow-network"]},
            {"effect": "git", "policy": "check run only", "flags": ["--allow-git"]}
        ],
        "command_groups": [
            {"name": "check", "writes": "flag-gated", "subprocess": "flag-gated", "network": "flag-gated"},
            {"name": "docs", "writes": "flag-gated", "subprocess": "flag-gated", "network": "default-deny"},
            {"name": "configs", "writes": "flag-gated", "subprocess": "flag-gated", "network": "default-deny"},
            {"name": "ops", "writes": "flag-gated", "subprocess": "flag-gated", "network": "default-deny"}
        ]
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_version_command(
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "name": "bijux-dev-atlas",
        "version": env!("CARGO_PKG_VERSION"),
        "git_hash": option_env!("BIJUX_GIT_HASH"),
    });
    let rendered = match format {
        FormatArg::Text => {
            let version = payload
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let git_hash = payload
                .get("git_hash")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!("bijux-dev-atlas {version}\ngit_hash: {git_hash}")
        }
        _ => emit_payload(format, out.clone(), &payload)?,
    };
    if matches!(format, FormatArg::Text) {
        write_output_if_requested(out, &rendered)?;
    }
    Ok((rendered, 0))
}

pub(crate) fn run_help_inventory_command(
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let payload = help_inventory_payload();
    validate_help_inventory_payload(&payload)?;
    let rendered = match format {
        FormatArg::Text => payload["commands"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|row| row.get("id").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => emit_payload(format, out.clone(), &payload)?,
    };
    if matches!(format, FormatArg::Text) {
        write_output_if_requested(out, &rendered)?;
    }
    Ok((rendered, 0))
}

pub(crate) fn help_inventory_payload() -> serde_json::Value {
    let commands = vec![
        serde_json::json!({
            "id": "build",
            "kind": "group",
            "purpose": "build binaries, plans, and local install artifacts",
            "effects": ["fs_read", "fs_write", "subprocess"],
            "inputs": ["repo_root", "artifacts_root", "build profile"],
            "outputs": ["build metadata", "dist artifacts", "local install output"],
            "report_ids": [],
            "hidden": true,
            "subcommands": ["bin", "plan", "verify", "meta", "dist", "clean", "doctor", "install-local"]
        }),
        serde_json::json!({
            "id": "capabilities",
            "kind": "leaf",
            "purpose": "describe the default-deny effect model and required flags",
            "effects": [],
            "inputs": [],
            "outputs": ["capabilities report"],
            "report_ids": [],
            "hidden": true
        }),
        serde_json::json!({
            "id": "check",
            "kind": "group",
            "purpose": "discover, explain, and execute governed checks",
            "effects": ["fs_read", "fs_write", "subprocess", "git", "network"],
            "inputs": ["suite selector", "check selector", "capability flags"],
            "outputs": ["check execution report"],
            "report_ids": ["artifact-report-validation"],
            "hidden": false,
            "subcommands": ["registry", "list", "explain", "doctor", "run"]
        }),
        serde_json::json!({
            "id": "ci",
            "kind": "group",
            "purpose": "run CI-oriented lane, explain, report, and verification surfaces",
            "effects": ["fs_read", "fs_write", "subprocess", "git", "network"],
            "inputs": ["lane", "gate", "format"],
            "outputs": ["ci reports", "lane explanations", "verification artifacts"],
            "report_ids": ["ci_workflow_lint"],
            "hidden": false
        }),
        serde_json::json!({
            "id": "configs",
            "kind": "group",
            "purpose": "inspect, validate, lint, and compile repository configuration inputs",
            "effects": ["fs_read", "fs_write", "subprocess", "network"],
            "inputs": ["repo_root", "artifacts_root", "config file"],
            "outputs": ["config validation reports", "compiled config output"],
            "report_ids": [],
            "hidden": false
        }),
        serde_json::json!({
            "id": "contracts",
            "kind": "group",
            "purpose": "run contract registries, snapshots, doctors, and effect-mode reports",
            "effects": ["fs_read", "fs_write", "subprocess", "network"],
            "inputs": ["domain", "mode", "run_id"],
            "outputs": ["contract reports", "registry snapshots"],
            "report_ids": ["ops-profiles", "helm-env"],
            "hidden": false
        }),
        serde_json::json!({
            "id": "docker",
            "kind": "group",
            "purpose": "run docker validation and release-oriented commands",
            "effects": ["fs_read", "fs_write", "subprocess", "network"],
            "inputs": ["repo_root", "artifacts_root", "policy selection"],
            "outputs": ["docker validation reports"],
            "report_ids": [],
            "hidden": true
        }),
        serde_json::json!({
            "id": "docs",
            "kind": "group",
            "purpose": "build, lint, validate, and regenerate governed documentation artifacts",
            "effects": ["fs_read", "fs_write", "subprocess", "network"],
            "inputs": ["repo_root", "artifacts_root", "run_id"],
            "outputs": ["docs reports", "generated references", "site output artifacts"],
            "report_ids": ["docs-site-output", "docs-build-closure-summary", "closure-index"],
            "hidden": false
        }),
        serde_json::json!({
            "id": "gates",
            "kind": "group",
            "purpose": "list and run curated gate suites",
            "effects": ["fs_read", "fs_write", "subprocess", "git", "network"],
            "inputs": ["suite", "capability flags"],
            "outputs": ["gate run reports"],
            "report_ids": [],
            "hidden": true
        }),
        serde_json::json!({
            "id": "governance",
            "kind": "group",
            "purpose": "inspect and validate governed repository objects and coverage artifacts",
            "effects": ["fs_read", "fs_write"],
            "inputs": ["repo_root", "governance object id"],
            "outputs": ["governance graph", "coverage", "orphan reports"],
            "report_ids": [],
            "hidden": false
        }),
        serde_json::json!({
            "id": "help",
            "kind": "leaf",
            "purpose": "emit the machine-readable command inventory and text summary",
            "effects": [],
            "inputs": [],
            "outputs": ["command inventory"],
            "report_ids": [],
            "hidden": true
        }),
        serde_json::json!({
            "id": "make",
            "kind": "group",
            "purpose": "inspect and validate thin make wrapper surfaces",
            "effects": ["fs_read", "subprocess"],
            "inputs": ["target", "repo_root"],
            "outputs": ["makes surface reports"],
            "report_ids": [],
            "hidden": true
        }),
        serde_json::json!({
            "id": "ops",
            "kind": "group",
            "purpose": "validate, render, plan, and execute governed ops workflows",
            "effects": ["fs_read", "fs_write", "subprocess", "network"],
            "inputs": ["profile", "ops_root", "artifacts_root", "run_id"],
            "outputs": ["ops run reports", "rendered manifests", "inventory summaries"],
            "report_ids": ["ops-profiles", "helm-env"],
            "hidden": false
        }),
        serde_json::json!({
            "id": "policies",
            "kind": "group",
            "purpose": "inspect, explain, validate, and print control-plane policy sets",
            "effects": ["fs_read"],
            "inputs": ["policy id", "repo_root"],
            "outputs": ["policy reports"],
            "report_ids": [],
            "hidden": false
        }),
        serde_json::json!({
            "id": "release",
            "kind": "group",
            "purpose": "run release-specific verification entrypoints",
            "effects": ["fs_read"],
            "inputs": ["profile", "repo_root"],
            "outputs": ["release check reports"],
            "report_ids": [],
            "hidden": true
        }),
        serde_json::json!({
            "id": "validate",
            "kind": "leaf",
            "purpose": "run the curated top-level validation entrypoint across checks and ops",
            "effects": ["fs_read", "subprocess"],
            "inputs": ["profile", "repo_root"],
            "outputs": ["aggregate validation report"],
            "report_ids": [],
            "hidden": false
        }),
        serde_json::json!({
            "id": "version",
            "kind": "leaf",
            "purpose": "print control-plane version and compatibility metadata",
            "effects": [],
            "inputs": [],
            "outputs": ["version report"],
            "report_ids": [],
            "hidden": true
        }),
        serde_json::json!({
            "id": "workflows",
            "kind": "group",
            "purpose": "run workflow-oriented control-plane surfaces that back CI lanes",
            "effects": ["fs_read", "fs_write", "subprocess", "git", "network"],
            "inputs": ["lane", "gate", "format"],
            "outputs": ["workflow reports", "doctor output"],
            "report_ids": ["ci_workflow_lint"],
            "hidden": true
        }),
    ];
    serde_json::json!({
        "schema_version": 1,
        "kind": "command_inventory",
        "name": "bijux-dev-atlas",
        "count": commands.len(),
        "commands": commands
    })
}

fn validate_help_inventory_payload(payload: &serde_json::Value) -> Result<(), String> {
    let commands = payload
        .get("commands")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "command inventory must define commands array".to_string())?;
    let mut ids = BTreeSet::new();
    let mut hidden_ids = BTreeSet::new();
    for row in commands {
        let id = row
            .get("id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "command inventory entry missing id".to_string())?;
        if !ids.insert(id.to_string()) {
            return Err(format!("command inventory contains duplicate id `{id}`"));
        }
        for field in [
            "kind",
            "purpose",
            "effects",
            "inputs",
            "outputs",
            "report_ids",
            "hidden",
        ] {
            if row.get(field).is_none() {
                return Err(format!("command inventory entry `{id}` missing `{field}`"));
            }
        }
        if row.get("hidden").and_then(|value| value.as_bool()) == Some(true) {
            hidden_ids.insert(id.to_string());
        }
    }
    let allowlisted_hidden_ids = load_hidden_command_allowlist()?;
    if hidden_ids != allowlisted_hidden_ids {
        let missing = hidden_ids
            .difference(&allowlisted_hidden_ids)
            .cloned()
            .collect::<Vec<_>>();
        let stale = allowlisted_hidden_ids
            .difference(&hidden_ids)
            .cloned()
            .collect::<Vec<_>>();
        return Err(format!(
            "hidden command allowlist mismatch: missing={missing:?} stale={stale:?}"
        ));
    }
    Ok(())
}

fn load_hidden_command_allowlist() -> Result<BTreeSet<String>, String> {
    let path = resolve_hidden_command_allowlist_path();
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    value
        .get("hidden_command_ids")
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("{} must define hidden_command_ids", path.display()))
        .map(|rows| {
            rows.iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
}

fn resolve_hidden_command_allowlist_path() -> PathBuf {
    let cwd_path = Path::new("configs/sources/runtime/cli/hidden-command-allowlist.json");
    if cwd_path.exists() {
        return cwd_path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| Path::new("."))
        .join("configs/sources/runtime/cli/hidden-command-allowlist.json")
}
