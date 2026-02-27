// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ContractsCommand, ContractsFormatArg, ContractsModeArg, ContractsOpsDomainArg,
    ContractsSnapshotDomainArg, PoliciesCommand,
};
use crate::*;
use bijux_dev_atlas::contracts;
use bijux_dev_atlas::model::CONTRACT_SCHEMA_VERSION;
use bijux_dev_atlas::policies::{canonical_policy_json, DevAtlasPolicySet};
use std::fs;
use std::io::{self, Write};

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

mod control_plane_docker;
pub(crate) use control_plane_docker::run_docker_command;
use control_plane_docker::{run_policies_explain, run_policies_list, run_policies_report};

pub(crate) fn run_contracts_command(quiet: bool, command: ContractsCommand) -> i32 {
    fn require_artifacts_root_in_ci(
        artifacts_root: &Option<PathBuf>,
    ) -> Result<(), String> {
        if std::env::var_os("CI").is_some() && artifacts_root.is_none() {
            return Err(
                "CI contracts runs require --artifacts-root for deterministic evidence output"
                    .to_string(),
            );
        }
        Ok(())
    }

    fn require_effect_allowances(
        mode: ContractsModeArg,
        allow_subprocess: bool,
        allow_network: bool,
    ) -> Result<(), String> {
        if mode == ContractsModeArg::Effect && (!allow_subprocess || !allow_network) {
            return Err(
                "effect mode requires both --allow-subprocess and --allow-network".to_string(),
            );
        }
        Ok(())
    }

    fn ops_domain_filter(domain: ContractsOpsDomainArg) -> String {
        match domain {
            ContractsOpsDomainArg::Root => "OPS-ROOT-*".to_string(),
            ContractsOpsDomainArg::Datasets => "OPS-DATASETS-*".to_string(),
            ContractsOpsDomainArg::E2e => "OPS-E2E-*".to_string(),
            ContractsOpsDomainArg::Env => "OPS-ENV-*".to_string(),
            ContractsOpsDomainArg::Inventory => "OPS-INV-*".to_string(),
            ContractsOpsDomainArg::K8s => "OPS-K8S-*".to_string(),
            ContractsOpsDomainArg::Load => "OPS-LOAD-*".to_string(),
            ContractsOpsDomainArg::Observe => "OPS-OBS-*".to_string(),
            ContractsOpsDomainArg::Report => "OPS-REPORT-*".to_string(),
            ContractsOpsDomainArg::Schema => "OPS-SCHEMA-*".to_string(),
            ContractsOpsDomainArg::Stack => "OPS-STACK-*".to_string(),
        }
    }

    fn ops_mapped_gates(repo_root: &Path, contract_id: &str) -> Vec<String> {
        let path = repo_root.join("ops/inventory/contract-gate-map.json");
        let Ok(text) = fs::read_to_string(path) else {
            return Vec::new();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            return Vec::new();
        };
        json.get("mappings")
            .and_then(|v| v.as_array())
            .and_then(|rows| {
                rows.iter().find(|item| {
                    item.get("contract_id")
                        .and_then(|v| v.as_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case(contract_id))
                })
            })
            .and_then(|item| item.get("gate_ids"))
            .and_then(|v| v.as_array())
            .map(|gate_ids| {
                gate_ids
                    .iter()
                    .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }

    let run = (|| -> Result<(String, i32), String> {
        match command {
            ContractsCommand::Docker(args) => {
                let repo_root = resolve_repo_root(args.repo_root)?;
                let registry = contracts::docker::contracts(&repo_root)?;
                if let Some(contract_id) = args.explain {
                    let Some(contract) = registry
                        .iter()
                        .find(|entry| entry.id.0.eq_ignore_ascii_case(&contract_id))
                    else {
                        return Err(format!("unknown docker contract id `{contract_id}`"));
                    };
                    let explanation = contracts::docker::contract_explain(&contract.id.0);
                    let rendered = if args.json || args.format == ContractsFormatArg::Json {
                        serde_json::to_string_pretty(&serde_json::json!({
                            "schema_version": 1,
                            "domain": "docker",
                            "contract_id": contract.id.0,
                            "title": contract.title,
                            "tests": contract.tests.iter().map(|case| serde_json::json!({
                                "test_id": case.id.0,
                                "title": case.title
                            })).collect::<Vec<_>>(),
                            "mapped_gate": "bijux dev atlas contracts docker --mode static",
                            "explain": explanation
                        }))
                        .map_err(|e| format!("encode contracts explain failed: {e}"))?
                    } else {
                        let mut out = String::new();
                        out.push_str(&format!("{} {}\n", contract.id.0, contract.title));
                        out.push_str("Tests:\n");
                        for case in &contract.tests {
                            out.push_str(&format!("- {}: {}\n", case.id.0, case.title));
                        }
                        out.push_str("\nHow to fix:\n");
                        out.push_str(explanation.as_str());
                        out.push_str("\n\nMapped gate:\n");
                        out.push_str("bijux dev atlas contracts docker --mode static");
                        out.push('\n');
                        out
                    };
                    return Ok((rendered, 0));
                }
                require_artifacts_root_in_ci(&args.artifacts_root)?;
                require_effect_allowances(args.mode, args.allow_subprocess, args.allow_network)?;
                if args.list {
                    let rendered = if args.json || args.format == ContractsFormatArg::Json {
                        serde_json::to_string_pretty(&serde_json::json!({
                            "schema_version": 1,
                            "domain": "docker",
                            "contracts": registry.iter().map(|contract| serde_json::json!({
                                "id": contract.id.0,
                                "title": contract.title,
                                "tests": contract.tests.iter().map(|case| serde_json::json!({
                                    "test_id": case.id.0,
                                    "title": case.title
                                })).collect::<Vec<_>>()
                            })).collect::<Vec<_>>()
                        }))
                        .map_err(|e| format!("encode contracts list failed: {e}"))?
                    } else {
                        let mut out = String::new();
                        out.push_str("Contracts: docker\n");
                        for contract in &registry {
                            out.push_str(&format!("{} {}\n", contract.id.0, contract.title));
                            for case in &contract.tests {
                                out.push_str(&format!("  - {} {}\n", case.id.0, case.title));
                            }
                        }
                        out
                    };
                    return Ok((rendered, 0));
                }
                let mode = match args.mode {
                    ContractsModeArg::Static => contracts::Mode::Static,
                    ContractsModeArg::Effect => contracts::Mode::Effect,
                };
                let options = contracts::RunOptions {
                    mode,
                    allow_subprocess: args.allow_subprocess,
                    allow_network: args.allow_network,
                    skip_missing_tools: args.skip_missing_tools,
                    timeout_seconds: args.timeout_seconds,
                    fail_fast: args.fail_fast,
                    contract_filter: args.filter_contract,
                    test_filter: args.filter_test,
                    list_only: args.list,
                    artifacts_root: args.artifacts_root,
                };
                let report =
                    contracts::run("docker", contracts::docker::contracts, &repo_root, &options)?;
                let rendered = if args.json || args.format == ContractsFormatArg::Json {
                    serde_json::to_string_pretty(&contracts::to_json(&report))
                        .map_err(|e| format!("encode contracts report failed: {e}"))?
                } else if args.format == ContractsFormatArg::Junit {
                    contracts::to_junit(&report)?
                } else {
                    contracts::to_pretty(&report)
                };
                Ok((rendered, report.exit_code()))
            }
            ContractsCommand::Ops(args) => {
                let repo_root = resolve_repo_root(args.repo_root)?;
                let registry = contracts::ops::contracts(&repo_root)?;
                if let Some(contract_id) = args.explain {
                    let Some(contract) = registry
                        .iter()
                        .find(|entry| entry.id.0.eq_ignore_ascii_case(&contract_id))
                    else {
                        return Err(format!("unknown ops contract id `{contract_id}`"));
                    };
                    let explanation = contracts::ops::contract_explain(&contract.id.0);
                    let mapped_gates = ops_mapped_gates(&repo_root, &contract.id.0);
                    let rendered = if args.json || args.format == ContractsFormatArg::Json {
                        serde_json::to_string_pretty(&serde_json::json!({
                            "schema_version": 1,
                            "domain": "ops",
                            "contract_id": contract.id.0,
                            "title": contract.title,
                            "tests": contract.tests.iter().map(|case| serde_json::json!({
                                "test_id": case.id.0,
                                "title": case.title
                            })).collect::<Vec<_>>(),
                            "mapped_gate": contracts::ops::contract_gate_command(&contract.id.0),
                            "mapped_gates": mapped_gates,
                            "explain": explanation
                        }))
                        .map_err(|e| format!("encode contracts explain failed: {e}"))?
                    } else {
                        let mut out = String::new();
                        out.push_str(&format!("{} {}\n", contract.id.0, contract.title));
                        out.push_str("Tests:\n");
                        for case in &contract.tests {
                            out.push_str(&format!("- {}: {}\n", case.id.0, case.title));
                        }
                        out.push_str("\nHow to fix:\n");
                        out.push_str(explanation);
                        out.push_str("\n\nMapped gate:\n");
                        out.push_str(contracts::ops::contract_gate_command(&contract.id.0));
                        if !mapped_gates.is_empty() {
                            out.push_str("\nMapped gate ids:\n");
                            for gate_id in &mapped_gates {
                                out.push_str(&format!("- {gate_id}\n"));
                            }
                        }
                        out.push('\n');
                        out
                    };
                    return Ok((rendered, 0));
                }
                require_artifacts_root_in_ci(&args.artifacts_root)?;
                require_effect_allowances(args.mode, args.allow_subprocess, args.allow_network)?;
                if args.list {
                    let rendered = if args.json || args.format == ContractsFormatArg::Json {
                        serde_json::to_string_pretty(&serde_json::json!({
                            "schema_version": 1,
                            "domain": "ops",
                            "contracts": registry.iter().map(|contract| serde_json::json!({
                                "id": contract.id.0,
                                "title": contract.title,
                                "tests": contract.tests.iter().map(|case| serde_json::json!({
                                    "test_id": case.id.0,
                                    "title": case.title
                                })).collect::<Vec<_>>()
                            })).collect::<Vec<_>>()
                        }))
                        .map_err(|e| format!("encode contracts list failed: {e}"))?
                    } else {
                        let mut out = String::new();
                        out.push_str("Contracts: ops\n");
                        for contract in &registry {
                            out.push_str(&format!("{} {}\n", contract.id.0, contract.title));
                            for case in &contract.tests {
                                out.push_str(&format!("  - {} {}\n", case.id.0, case.title));
                            }
                        }
                        out
                    };
                    return Ok((rendered, 0));
                }
                let mode = match args.mode {
                    ContractsModeArg::Static => contracts::Mode::Static,
                    ContractsModeArg::Effect => contracts::Mode::Effect,
                };
                let contract_filter = if let Some(domain) = args.domain {
                    Some(ops_domain_filter(domain))
                } else {
                    args.filter_contract
                };
                let options = contracts::RunOptions {
                    mode,
                    allow_subprocess: args.allow_subprocess,
                    allow_network: args.allow_network,
                    skip_missing_tools: args.skip_missing_tools,
                    timeout_seconds: args.timeout_seconds,
                    fail_fast: args.fail_fast,
                    contract_filter,
                    test_filter: args.filter_test,
                    list_only: args.list,
                    artifacts_root: args.artifacts_root,
                };
                let report =
                    contracts::run("ops", contracts::ops::contracts, &repo_root, &options)?;
                let rendered = if args.json || args.format == ContractsFormatArg::Json {
                    serde_json::to_string_pretty(&contracts::to_json(&report))
                        .map_err(|e| format!("encode contracts report failed: {e}"))?
                } else if args.format == ContractsFormatArg::Junit {
                    contracts::to_junit(&report)?
                } else {
                    contracts::to_pretty(&report)
                };
                Ok((rendered, report.exit_code()))
            }
            ContractsCommand::Snapshot(args) => {
                let repo_root = resolve_repo_root(args.repo_root)?;
                let (domain, mut registry, default_rel) = match args.domain {
                    ContractsSnapshotDomainArg::Docker => (
                        "docker",
                        contracts::docker::contracts(&repo_root)?,
                        PathBuf::from("docker/_generated/contracts-registry-snapshot.json"),
                    ),
                    ContractsSnapshotDomainArg::Ops => (
                        "ops",
                        contracts::ops::contracts(&repo_root)?,
                        PathBuf::from("ops/_generated/control-plane-surface-list.json"),
                    ),
                };
                let payload = if domain == "ops" {
                    contracts::ops::render_registry_snapshot_json(&repo_root)?
                } else {
                    registry.sort_by_key(|c| c.id.0.clone());
                    let contracts = registry
                        .into_iter()
                        .map(|mut c| {
                            c.tests.sort_by_key(|t| t.id.0.clone());
                            serde_json::json!({
                                "id": c.id.0,
                                "title": c.title,
                                "tests": c.tests.into_iter().map(|t| serde_json::json!({
                                    "test_id": t.id.0,
                                    "title": t.title
                                })).collect::<Vec<_>>()
                            })
                        })
                        .collect::<Vec<_>>();
                    serde_json::json!({
                        "schema_version": 1,
                        "domain": domain,
                        "contracts": contracts,
                    })
                };
                let rendered = serde_json::to_string_pretty(&payload)
                    .map_err(|e| format!("encode contracts snapshot failed: {e}"))?;
                let out_path = args.out.unwrap_or_else(|| repo_root.join(default_rel));
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
                }
                fs::write(&out_path, format!("{rendered}\n"))
                    .map_err(|e| format!("write {} failed: {e}", out_path.display()))?;
                Ok((rendered, 0))
            }
        }
    })();

    match run {
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
            let _ = writeln!(io::stderr(), "bijux-dev-atlas contracts failed: {err}");
            1
        }
    }
}

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
    let payload = serde_json::json!({
        "schema_version": 1,
        "name": "bijux-dev-atlas",
        "commands": [
            {"name": "version", "kind": "leaf"},
            {"name": "help", "kind": "leaf"},
            {"name": "check", "kind": "group", "subcommands": ["registry", "list", "explain", "doctor", "run"]},
            {"name": "ops", "kind": "group"},
            {"name": "docs", "kind": "group"},
            {"name": "contracts", "kind": "group"},
            {"name": "configs", "kind": "group"},
            {"name": "build", "kind": "group"},
            {"name": "policies", "kind": "group"},
            {"name": "docker", "kind": "group"},
            {"name": "workflows", "kind": "group"},
            {"name": "gates", "kind": "group"},
            {"name": "capabilities", "kind": "leaf"}
        ]
    });
    let rendered = match format {
        FormatArg::Text => payload["commands"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|row| row.get("name").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => emit_payload(format, out.clone(), &payload)?,
    };
    if matches!(format, FormatArg::Text) {
        write_output_if_requested(out, &rendered)?;
    }
    Ok((rendered, 0))
}
