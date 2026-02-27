// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ContractsCommand, ContractsCommonArgs, ContractsFormatArg, ContractsModeArg,
    ContractsOpsDomainArg, ContractsSnapshotDomainArg, PoliciesCommand,
};
use crate::*;
use bijux_dev_atlas::contracts;
use bijux_dev_atlas::model::CONTRACT_SCHEMA_VERSION;
use bijux_dev_atlas::policies::{canonical_policy_json, DevAtlasPolicySet};
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

mod control_plane_docker;
pub(crate) use control_plane_docker::run_docker_command;
use control_plane_docker::{run_policies_explain, run_policies_list, run_policies_report};

pub(crate) fn run_contracts_command(quiet: bool, command: ContractsCommand) -> i32 {
    fn require_artifacts_root_in_ci(artifacts_root: &Option<PathBuf>) -> Result<(), String> {
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

    fn common_format(common: &ContractsCommonArgs) -> ContractsFormatArg {
        if common.json {
            ContractsFormatArg::Json
        } else {
            common.format
        }
    }

    fn write_optional(path: &Option<PathBuf>, rendered: &str) -> Result<(), String> {
        if let Some(path) = path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
            }
            fs::write(path, rendered).map_err(|e| format!("write {} failed: {e}", path.display()))?;
        }
        Ok(())
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

    #[derive(Clone, Copy)]
    struct DomainDescriptor {
        name: &'static str,
        contracts_fn: fn(&Path) -> Result<Vec<contracts::Contract>, String>,
        explain_fn: fn(&str) -> String,
        gate_fn: fn(&str) -> &'static str,
    }

    fn domain_descriptor(name: &str) -> Option<DomainDescriptor> {
        match name {
            "docker" => Some(DomainDescriptor {
                name: "docker",
                contracts_fn: contracts::docker::contracts,
                explain_fn: contracts::docker::contract_explain,
                gate_fn: |_id| "bijux dev atlas contracts docker --mode static",
            }),
            "make" => Some(DomainDescriptor {
                name: "make",
                contracts_fn: contracts::make::contracts,
                explain_fn: contracts::make::contract_explain,
                gate_fn: contracts::make::contract_gate_command,
            }),
            "ops" => Some(DomainDescriptor {
                name: "ops",
                contracts_fn: contracts::ops::contracts,
                explain_fn: |id| contracts::ops::contract_explain(id).to_string(),
                gate_fn: contracts::ops::contract_gate_command,
            }),
            _ => None,
        }
    }

    fn all_domains(repo_root: &Path) -> Result<Vec<(DomainDescriptor, Vec<contracts::Contract>)>, String> {
        let mut out = Vec::new();
        for name in ["docker", "make", "ops"] {
            let descriptor = domain_descriptor(name).expect("known domain");
            out.push((descriptor, (descriptor.contracts_fn)(repo_root)?));
        }
        Ok(out)
    }

    fn registry_lints(repo_root: &Path) -> Result<Vec<contracts::RegistryLint>, String> {
        let mut rows = Vec::new();
        for (descriptor, registry) in all_domains(repo_root)? {
            rows.extend(contracts::registry_snapshot(descriptor.name, &registry));
        }
        Ok(contracts::lint_registry_rows(&rows))
    }

    fn render_registry_lints(
        lints: &[contracts::RegistryLint],
        format: ContractsFormatArg,
    ) -> Result<String, String> {
        if lints.is_empty() {
            return Ok(String::new());
        }
        match format {
            ContractsFormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "status": "invalid",
                "lints": lints.iter().map(|lint| serde_json::json!({
                    "code": lint.code,
                    "message": lint.message
                })).collect::<Vec<_>>()
            }))
            .map_err(|e| format!("encode contracts lint report failed: {e}")),
            _ => Ok(lints
                .iter()
                .map(|lint| format!("{}: {}", lint.code, lint.message))
                .collect::<Vec<_>>()
                .join("\n")),
        }
    }

    fn render_list(
        domains: &[(DomainDescriptor, Vec<contracts::Contract>)],
        include_tests: bool,
        format: ContractsFormatArg,
    ) -> Result<String, String> {
        let mut rows = Vec::new();
        for (descriptor, registry) in domains {
            rows.extend(contracts::registry_snapshot(descriptor.name, registry));
        }
        rows.sort_by(|a, b| a.domain.cmp(&b.domain).then(a.id.cmp(&b.id)));
        match format {
            ContractsFormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "contracts": rows.iter().map(|row| serde_json::json!({
                    "domain": row.domain,
                    "id": row.id,
                    "title": row.title,
                    "tests": row.test_ids.iter().map(|test_id| serde_json::json!({
                        "test_id": test_id
                    })).collect::<Vec<_>>()
                })).collect::<Vec<_>>()
            }))
            .map_err(|e| format!("encode contracts list failed: {e}")),
            _ => {
                let mut out = String::new();
                out.push_str("DOMAIN   CONTRACT ID        TITLE\n");
                for row in rows {
                    out.push_str(&format!("{:<8} {:<18} {}\n", row.domain, row.id, row.title));
                    if include_tests {
                        for test_id in row.test_ids {
                            out.push_str(&format!("         - {}\n", test_id));
                        }
                    }
                }
                Ok(out)
            }
        }
    }

    fn explain_test(
        domain: &str,
        contract: &contracts::Contract,
        test: &contracts::TestCase,
        format: ContractsFormatArg,
    ) -> Result<String, String> {
        let effects = match test.kind {
            contracts::TestKind::Pure => Vec::<&str>::new(),
            contracts::TestKind::Subprocess => vec!["subprocess"],
            contracts::TestKind::Network => vec!["network"],
        };
        let payload = serde_json::json!({
            "schema_version": 1,
            "domain": domain,
            "contract_id": contract.id.0,
            "contract_title": contract.title,
            "test_id": test.id.0,
            "test_title": test.title,
            "kind": format!("{:?}", test.kind).to_ascii_lowercase(),
            "inputs_read": ["repository workspace"],
            "outputs_written": ["artifacts root when configured"],
            "effects_required": effects,
        });
        match format {
            ContractsFormatArg::Json => serde_json::to_string_pretty(&payload)
                .map_err(|e| format!("encode test explanation failed: {e}")),
            _ => Ok(format!(
                "{} {}\n{} {}\nInputs read:\n- repository workspace\nOutputs written:\n- artifacts root when configured\nEffects required:\n{}",
                contract.id.0,
                contract.title,
                test.id.0,
                test.title,
                if effects.is_empty() {
                    "- none".to_string()
                } else {
                    effects
                        .into_iter()
                        .map(|effect| format!("- {effect}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            )),
        }
    }

    fn run_one(
        descriptor: &DomainDescriptor,
        repo_root: &Path,
        common: &ContractsCommonArgs,
        contract_filter: Option<String>,
    ) -> Result<contracts::RunReport, String> {
        let mode = match common.mode {
            ContractsModeArg::Static => contracts::Mode::Static,
            ContractsModeArg::Effect => contracts::Mode::Effect,
        };
        let options = contracts::RunOptions {
            mode,
            allow_subprocess: common.allow_subprocess,
            allow_network: common.allow_network,
            skip_missing_tools: common.skip_missing_tools,
            timeout_seconds: common.timeout_seconds,
            fail_fast: common.fail_fast,
            contract_filter,
            test_filter: common.filter_test.clone(),
            only_contracts: common.only_contracts.clone(),
            only_tests: common.only_tests.clone(),
            skip_contracts: common.skip_contracts.clone(),
            tags: common.tags.clone(),
            list_only: false,
            artifacts_root: common.artifacts_root.clone(),
        };
        contracts::run(descriptor.name, descriptor.contracts_fn, repo_root, &options)
    }

    let run = (|| -> Result<(String, i32), String> {
        if let ContractsCommand::Snapshot(args) = &command {
            let repo_root = resolve_repo_root(args.repo_root.clone())?;
            let domains = all_domains(&repo_root)?;
            let (domain_name, rows, default_rel) = match args.domain {
                ContractsSnapshotDomainArg::All => (
                    "all",
                    domains
                        .iter()
                        .flat_map(|(descriptor, registry)| {
                            contracts::registry_snapshot(descriptor.name, registry)
                        })
                        .collect::<Vec<_>>(),
                    PathBuf::from("artifacts/contracts/all/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Docker => (
                    "docker",
                    contracts::registry_snapshot(
                        "docker",
                        &domains
                            .iter()
                            .find(|(descriptor, _)| descriptor.name == "docker")
                            .expect("docker domain")
                            .1,
                    ),
                    PathBuf::from("docker/_generated/contracts-registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Make => (
                    "make",
                    contracts::registry_snapshot(
                        "make",
                        &domains
                            .iter()
                            .find(|(descriptor, _)| descriptor.name == "make")
                            .expect("make domain")
                            .1,
                    ),
                    PathBuf::from("make/contracts-registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Ops => (
                    "ops",
                    contracts::registry_snapshot(
                        "ops",
                        &domains
                            .iter()
                            .find(|(descriptor, _)| descriptor.name == "ops")
                            .expect("ops domain")
                            .1,
                    ),
                    PathBuf::from("ops/_generated/control-plane-surface-list.json"),
                ),
            };
            let payload = serde_json::json!({
                "schema_version": 1,
                "domain": domain_name,
                "contracts": rows.iter().map(|row| serde_json::json!({
                    "domain": row.domain,
                    "id": row.id,
                    "title": row.title,
                    "tests": row.test_ids,
                })).collect::<Vec<_>>()
            });
            let rendered = serde_json::to_string_pretty(&payload)
                .map_err(|e| format!("encode contracts snapshot failed: {e}"))?;
            let out_path = args.out.clone().unwrap_or_else(|| repo_root.join(default_rel));
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
            }
            fs::write(&out_path, format!("{rendered}\n"))
                .map_err(|e| format!("write {} failed: {e}", out_path.display()))?;
            return Ok((rendered, 0));
        }

        let (repo_root, common, domain_names, contract_filter_override) = match &command {
            ContractsCommand::All(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["docker", "make", "ops"],
                None,
            ),
            ContractsCommand::Docker(args) => (
                resolve_repo_root(args.common.repo_root.clone())?,
                args.common.clone(),
                vec!["docker"],
                None,
            ),
            ContractsCommand::Make(args) => (
                resolve_repo_root(args.common.repo_root.clone())?,
                args.common.clone(),
                vec!["make"],
                None,
            ),
            ContractsCommand::Ops(args) => (
                resolve_repo_root(args.common.repo_root.clone())?,
                args.common.clone(),
                vec!["ops"],
                args.domain.map(ops_domain_filter),
            ),
            ContractsCommand::Snapshot(_) => unreachable!("handled above"),
        };

        let format = common_format(&common);
        let lints = registry_lints(&repo_root)?;
        if !lints.is_empty() {
            return Ok((render_registry_lints(&lints, format)?, 2));
        }
        require_artifacts_root_in_ci(&common.artifacts_root)?;
        require_effect_allowances(common.mode, common.allow_subprocess, common.allow_network)?;

        let selected_domains = all_domains(&repo_root)?
            .into_iter()
            .filter(|(descriptor, _)| domain_names.iter().any(|name| descriptor.name == *name))
            .collect::<Vec<_>>();

        if common.list || common.list_tests {
            return Ok((render_list(&selected_domains, common.list_tests, format)?, 0));
        }

        if let Some(test_id) = &common.explain_test {
            for (descriptor, registry) in &selected_domains {
                for contract in registry {
                    if let Some(test) = contract
                        .tests
                        .iter()
                        .find(|test| test.id.0.eq_ignore_ascii_case(test_id))
                    {
                        return Ok((explain_test(descriptor.name, contract, test, format)?, 0));
                    }
                }
            }
            return Err(format!("unknown contract test id `{test_id}`"));
        }

        if let Some(contract_id) = &common.explain {
            for (descriptor, registry) in &selected_domains {
                if let Some(contract) = registry
                    .iter()
                    .find(|entry| entry.id.0.eq_ignore_ascii_case(contract_id))
                {
                    let mapped_gates = if descriptor.name == "ops" {
                        ops_mapped_gates(&repo_root, &contract.id.0)
                    } else {
                        Vec::new()
                    };
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "domain": descriptor.name,
                        "contract_id": contract.id.0,
                        "title": contract.title,
                        "tests": contract.tests.iter().map(|case| serde_json::json!({
                            "test_id": case.id.0,
                            "title": case.title
                        })).collect::<Vec<_>>(),
                        "mapped_gate": (descriptor.gate_fn)(&contract.id.0),
                        "mapped_gates": mapped_gates,
                        "mapped_command": (descriptor.gate_fn)(&contract.id.0),
                        "explain": (descriptor.explain_fn)(&contract.id.0)
                    });
                    let rendered = match format {
                        ContractsFormatArg::Json => serde_json::to_string_pretty(&payload)
                            .map_err(|e| format!("encode contracts explain failed: {e}"))?,
                        _ => {
                            let mut out = String::new();
                            out.push_str(&format!("{} {}\n", contract.id.0, contract.title));
                            out.push_str("Tests:\n");
                            for case in &contract.tests {
                                out.push_str(&format!("- {}: {}\n", case.id.0, case.title));
                            }
                            out.push_str("\nIntent:\n");
                            out.push_str(&(descriptor.explain_fn)(&contract.id.0));
                            out.push_str("\n\nMapped gate:\n");
                            out.push_str((descriptor.gate_fn)(&contract.id.0));
                            out.push('\n');
                            if !mapped_gates.is_empty() {
                                out.push_str("Mapped gates:\n");
                                for gate in mapped_gates {
                                    out.push_str(&format!("- {gate}\n"));
                                }
                            }
                            out
                        }
                    };
                    return Ok((rendered, 0));
                }
            }
            return Err(format!("unknown contract id `{contract_id}`"));
        }

        let mut reports = Vec::new();
        for (descriptor, _) in &selected_domains {
            reports.push(run_one(
                descriptor,
                &repo_root,
                &common,
                contract_filter_override.clone().or_else(|| common.filter_contract.clone()),
            )?);
        }

        let rendered = match format {
            ContractsFormatArg::Human => {
                if reports.len() == 1 {
                    contracts::to_pretty(&reports[0])
                } else {
                    contracts::to_pretty_all(&reports)
                }
            }
            ContractsFormatArg::Json => serde_json::to_string_pretty(&if reports.len() == 1 {
                contracts::to_json(&reports[0])
            } else {
                contracts::to_json_all(&reports)
            })
            .map_err(|e| format!("encode contracts report failed: {e}"))?,
            ContractsFormatArg::Junit => {
                if reports.len() == 1 {
                    contracts::to_junit(&reports[0])?
                } else {
                    contracts::to_junit_all(&reports)?
                }
            }
            ContractsFormatArg::Github => contracts::to_github(&reports),
        };

        let json_rendered = serde_json::to_string_pretty(&if reports.len() == 1 {
            contracts::to_json(&reports[0])
        } else {
            contracts::to_json_all(&reports)
        })
        .map_err(|e| format!("encode contracts json report failed: {e}"))?;
        let junit_rendered = if reports.len() == 1 {
            contracts::to_junit(&reports[0])?
        } else {
            contracts::to_junit_all(&reports)?
        };
        write_optional(&common.json_out, &json_rendered)?;
        write_optional(&common.junit_out, &junit_rendered)?;

        Ok((
            rendered,
            reports
                .iter()
                .map(contracts::RunReport::exit_code)
                .max()
                .unwrap_or(0),
        ))
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
