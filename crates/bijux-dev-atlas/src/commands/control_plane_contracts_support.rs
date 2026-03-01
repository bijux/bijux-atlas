// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ContractsCommonArgs, ContractsFormatArg, ContractsLaneArg, ContractsModeArg,
    ContractsOpsDomainArg, ContractsProfileArg,
};
use bijux_dev_atlas::contracts;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(super) struct DomainDescriptor {
    pub(super) name: &'static str,
    pub(super) contracts_fn: fn(&Path) -> Result<Vec<contracts::Contract>, String>,
    pub(super) explain_fn: fn(&str) -> String,
    pub(super) gate_fn: fn(&str) -> &'static str,
}

pub(super) fn usage_error(message: impl Into<String>) -> Result<(String, i32), String> {
    Err(format!("usage: {}", message.into()))
}

pub(super) fn require_skip_policy(skip_contracts: &[String]) -> Result<(), String> {
    if std::env::var_os("CI").is_some()
        && !skip_contracts.is_empty()
        && std::env::var_os("CONTRACTS_ALLOW_SKIP").is_none()
    {
        return Err("CI contracts runs forbid --skip unless CONTRACTS_ALLOW_SKIP is set".to_string());
    }
    Ok(())
}

pub(super) fn ops_domain_filter(domain: ContractsOpsDomainArg) -> String {
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

pub(super) fn common_format(common: &ContractsCommonArgs) -> ContractsFormatArg {
    if common.json {
        ContractsFormatArg::Json
    } else {
        common.format
    }
}

pub(super) fn apply_lane_policy(common: &mut ContractsCommonArgs) {
    match common.lane {
        ContractsLaneArg::Local => {}
        ContractsLaneArg::Pr => {
            common.mode = ContractsModeArg::Static;
            common.profile = ContractsProfileArg::Ci;
            common.required = true;
        }
        ContractsLaneArg::Merge => {
            common.mode = ContractsModeArg::Effect;
            common.profile = ContractsProfileArg::Ci;
            common.required = true;
            common.allow_subprocess = true;
            common.allow_network = true;
            common.allow_k8s = true;
            common.allow_fs_write = true;
            common.allow_docker_daemon = true;
        }
        ContractsLaneArg::Release => {
            common.mode = ContractsModeArg::Effect;
            common.profile = ContractsProfileArg::Ci;
            common.allow_subprocess = true;
            common.allow_network = true;
            common.allow_k8s = true;
            common.allow_fs_write = true;
            common.allow_docker_daemon = true;
        }
    }
}

pub(super) fn apply_ci_policy(common: &mut ContractsCommonArgs) {
    if common.ci {
        common.profile = ContractsProfileArg::Ci;
        common.deny_skip_required = true;
    }
}

pub(super) fn validate_selection_patterns(common: &ContractsCommonArgs) -> Result<(), String> {
    for pattern in common.filter_contract.iter().chain(common.filter_test.iter()) {
        contracts::validate_wildcard_pattern(pattern)?;
    }
    for pattern in common
        .only_contracts
        .iter()
        .chain(common.only_tests.iter())
        .chain(common.skip_contracts.iter())
        .chain(common.tags.iter())
    {
        contracts::validate_wildcard_pattern(pattern)?;
    }
    Ok(())
}

pub(super) fn write_optional(path: &Option<PathBuf>, rendered: &str) -> Result<(), String> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
        }
        fs::write(path, rendered).map_err(|e| format!("write {} failed: {e}", path.display()))?;
    }
    Ok(())
}

pub(super) fn ops_mapped_gates(repo_root: &Path, contract_id: &str) -> Vec<String> {
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

pub(super) fn cli_lane(lane: ContractsLaneArg) -> contracts::ContractLane {
    match lane {
        ContractsLaneArg::Local => contracts::ContractLane::Local,
        ContractsLaneArg::Pr => contracts::ContractLane::Pr,
        ContractsLaneArg::Merge => contracts::ContractLane::Merge,
        ContractsLaneArg::Release => contracts::ContractLane::Release,
    }
}

pub(super) fn required_contract_rows_json(
    repo_root: &Path,
    domains: &[(DomainDescriptor, Vec<contracts::Contract>)],
) -> Result<serde_json::Value, String> {
    let mut rows = Vec::new();
    for (descriptor, registry) in domains {
        rows.extend(
            contracts::registry_snapshot_with_policy(repo_root, descriptor.name, registry)?
                .into_iter()
                .filter(|row| row.required)
                .map(|row| {
                    serde_json::json!({
                        "domain": row.domain,
                        "contract_id": row.id,
                        "required": row.required,
                        "lanes": row.lanes,
                    })
                }),
        );
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "contracts": rows,
    }))
}

pub(super) fn write_required_contract_artifact(
    repo_root: &Path,
    domains: &[(DomainDescriptor, Vec<contracts::Contract>)],
) -> Result<(), String> {
    let path = repo_root.join("ops/_generated.example/contracts-required.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&required_contract_rows_json(repo_root, domains)?)
            .map_err(|e| format!("encode required contracts artifact failed: {e}"))?,
    )
    .map_err(|e| format!("write {} failed: {e}", path.display()))
}

pub(super) fn forbid_skip_required(
    repo_root: &Path,
    domains: &[(DomainDescriptor, Vec<contracts::Contract>, String)],
    common: &ContractsCommonArgs,
    contract_filter_override: &Option<String>,
) -> Result<(), String> {
    if !common.deny_skip_required || common.skip_contracts.is_empty() {
        return Ok(());
    }
    let lane = cli_lane(common.lane);
    let filter = contract_filter_override
        .clone()
        .or_else(|| common.filter_contract.clone());
    for (descriptor, registry, _) in domains {
        for row in contracts::registry_snapshot_with_policy(repo_root, descriptor.name, registry)? {
            if !row.required || !row.lanes.iter().any(|value| value == lane.as_str()) {
                continue;
            }
            if !contracts::matches_filter(&filter, &row.id)
                || !contracts::matches_any_filter(&common.only_contracts, &row.id)
            {
                continue;
            }
            if contracts::matches_skip_filter(&common.skip_contracts, &row.id) {
                return Err(format!(
                    "required contract `{}` cannot be skipped in lane `{}`",
                    row.id, lane
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn domain_descriptor(name: &str) -> Option<DomainDescriptor> {
    match name {
        "root" => Some(DomainDescriptor {
            name: "root",
            contracts_fn: contracts::root::contracts,
            explain_fn: contracts::root::contract_explain,
            gate_fn: contracts::root::contract_gate_command,
        }),
        "runtime" => Some(DomainDescriptor {
            name: "runtime",
            contracts_fn: contracts::runtime::contracts,
            explain_fn: contracts::runtime::contract_explain,
            gate_fn: contracts::runtime::contract_gate_command,
        }),
        "control-plane" => Some(DomainDescriptor {
            name: "control-plane",
            contracts_fn: contracts::control_plane::contracts,
            explain_fn: contracts::control_plane::contract_explain,
            gate_fn: contracts::control_plane::contract_gate_command,
        }),
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
        "configs" => Some(DomainDescriptor {
            name: "configs",
            contracts_fn: contracts::configs::contracts,
            explain_fn: contracts::configs::contract_explain,
            gate_fn: contracts::configs::contract_gate_command,
        }),
        "docs" => Some(DomainDescriptor {
            name: "docs",
            contracts_fn: contracts::docs::contracts,
            explain_fn: contracts::docs::contract_explain,
            gate_fn: contracts::docs::contract_gate_command,
        }),
        _ => None,
    }
}

pub(super) fn all_domains(
    repo_root: &Path,
) -> Result<Vec<(DomainDescriptor, Vec<contracts::Contract>)>, String> {
    let mut out = Vec::new();
    for name in [
        "root",
        "runtime",
        "control-plane",
        "docker",
        "make",
        "ops",
        "configs",
        "docs",
    ] {
        let descriptor = domain_descriptor(name)
            .ok_or_else(|| format!("internal contracts domain registry is missing `{name}`"))?;
        out.push((descriptor, (descriptor.contracts_fn)(repo_root)?));
    }
    Ok(out)
}

pub(super) fn domain_registry<'a>(
    domains: &'a [(DomainDescriptor, Vec<contracts::Contract>)],
    name: &str,
) -> Result<&'a Vec<contracts::Contract>, String> {
    domains
        .iter()
        .find(|(descriptor, _)| descriptor.name == name)
        .map(|(_, registry)| registry)
        .ok_or_else(|| format!("internal contracts domain registry is missing `{name}`"))
}

pub(super) fn registry_lints(repo_root: &Path) -> Result<Vec<contracts::RegistryLint>, String> {
    let mut rows = Vec::new();
    for (descriptor, registry) in all_domains(repo_root)? {
        rows.extend(contracts::registry_snapshot_with_policy(
            repo_root,
            descriptor.name,
            &registry,
        )?);
    }
    Ok(contracts::lint_registry_rows(&rows))
}

pub(super) fn render_registry_lints(
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
        ContractsFormatArg::Human
        | ContractsFormatArg::Table
        | ContractsFormatArg::Junit
        | ContractsFormatArg::Github => Ok(lints
            .iter()
            .map(|lint| format!("{}: {}", lint.code, lint.message))
            .collect::<Vec<_>>()
            .join("\n")),
    }
}

pub(super) fn render_list(
    repo_root: &Path,
    domains: &[(DomainDescriptor, &[contracts::Contract])],
    include_tests: bool,
    format: ContractsFormatArg,
) -> Result<String, String> {
    let mut rows = Vec::new();
    for (descriptor, registry) in domains {
        rows.extend(contracts::registry_snapshot_with_policy(
            repo_root,
            descriptor.name,
            registry,
        )?);
    }
    rows.sort_by(|a, b| a.domain.cmp(&b.domain).then(a.id.cmp(&b.id)));
    match format {
        ContractsFormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "contracts": rows.iter().map(|row| serde_json::json!({
                "domain": row.domain,
                "id": row.id,
                "required": row.required,
                "lanes": row.lanes,
                "severity": row.severity,
                "title": row.title,
                "tests": row.test_ids.iter().map(|test_id| serde_json::json!({
                    "test_id": test_id
                })).collect::<Vec<_>>()
            })).collect::<Vec<_>>()
        }))
        .map_err(|e| format!("encode contracts list failed: {e}")),
        ContractsFormatArg::Human
        | ContractsFormatArg::Table
        | ContractsFormatArg::Junit
        | ContractsFormatArg::Github => {
            let mut out = String::new();
            out.push_str("GROUP    CONTRACT ID        REQUIRED LANES                SEVERITY TITLE\n");
            for row in rows {
                out.push_str(&format!(
                    "{:<8} {:<18} {:<8} {:<20} {:<8} {}\n",
                    row.domain,
                    row.id,
                    if row.required { "yes" } else { "no" },
                    row.lanes.join(","),
                    row.severity,
                    row.title
                ));
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

pub(super) fn changed_paths_since_merge_base(repo_root: &Path) -> Option<Vec<String>> {
    let repo_display = repo_root.display().to_string();
    let target = std::env::var("CONTRACTS_CHANGED_BASE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "HEAD".to_string());
    let base = std::process::Command::new("git")
        .args(["-C", &repo_display, "merge-base", "HEAD", &target])
        .output()
        .ok()?;
    if !base.status.success() {
        return None;
    }
    let base_sha = String::from_utf8_lossy(&base.stdout).trim().to_string();
    if base_sha.is_empty() {
        return None;
    }
    let diff = std::process::Command::new("git")
        .args(["-C", &repo_display, "diff", "--name-only", &base_sha, "HEAD"])
        .output()
        .ok()?;
    if !diff.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&diff.stdout)
            .lines()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
    )
}

pub(super) fn domain_change_reason(name: &str, changed_paths: &[String]) -> Option<String> {
    if changed_paths.is_empty() {
        return None;
    }
    if name == "root" {
        if changed_paths.iter().any(|path| !path.contains('/')) {
            return Some("changed root-level files".to_string());
        }
        return None;
    }
    if changed_paths
        .iter()
        .any(|path| path.starts_with(&format!("{name}/")))
    {
        return Some(format!("changed files under `{name}/`"));
    }
    None
}

pub(super) fn explain_test(
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
        ContractsFormatArg::Human
        | ContractsFormatArg::Table
        | ContractsFormatArg::Junit
        | ContractsFormatArg::Github => Ok(format!(
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

pub(super) fn run_one(
    descriptor: &DomainDescriptor,
    repo_root: &Path,
    common: &ContractsCommonArgs,
    contract_filter: Option<String>,
) -> Result<contracts::RunReport, String> {
    let run_id = common
        .run_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            std::env::var("RUN_ID")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "local".to_string());
    let mode = match common.mode {
        ContractsModeArg::Static => contracts::Mode::Static,
        ContractsModeArg::Effect => contracts::Mode::Effect,
    };
    let profile = match common.profile {
        ContractsProfileArg::Local => "local",
        ContractsProfileArg::Ci => "ci",
    };
    let artifacts_root = common.artifacts_root.clone().unwrap_or_else(|| {
        canonical_contracts_gate_root(repo_root, &run_id)
            .join(descriptor.name)
            .join(profile)
            .join(mode.to_string())
    });
    let previous_profile = std::env::var_os("BIJUX_CONTRACTS_PROFILE");
    std::env::set_var("BIJUX_CONTRACTS_PROFILE", profile);
    let ci_mode = common.ci || std::env::var_os("CI").is_some();
    let options = contracts::RunOptions {
        lane: cli_lane(common.lane),
        mode,
        run_id: Some(run_id.clone()),
        required_only: common.required,
        ci: ci_mode,
        color_enabled: !ci_mode,
        allow_subprocess: common.allow_subprocess,
        allow_network: common.allow_network,
        allow_k8s: common.allow_k8s,
        allow_fs_write: common.allow_fs_write,
        allow_docker_daemon: common.allow_docker_daemon,
        deny_skip_required: common.deny_skip_required,
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
        artifacts_root: Some(artifacts_root),
    };
    let result = contracts::run(descriptor.name, descriptor.contracts_fn, repo_root, &options);
    if let Some(value) = previous_profile {
        std::env::set_var("BIJUX_CONTRACTS_PROFILE", value);
    } else {
        std::env::remove_var("BIJUX_CONTRACTS_PROFILE");
    }
    result
}

fn canonical_contracts_gate_root(repo_root: &Path, run_id: &str) -> std::path::PathBuf {
    repo_root
        .join("artifacts")
        .join("run")
        .join(run_id)
        .join("gates")
        .join("contracts")
}
