// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io::Write;
use std::io::{stdout, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::cli::{FormatArg, SuiteColorArg, SuiteModeArg, SuiteOutputFormatArg, SuitesCommand};
use crate::resolve_repo_root;
use bijux_dev_atlas::docs::site_output::{
    validate_named_report, validate_report_value_against_schema,
};
use bijux_dev_atlas::engine::{
    ArtifactStore, CommandRunnableExecutor, EffectPolicy, RunStatus, RunnableExecutor,
    RunnableRunContext,
};
use bijux_dev_atlas::model::{RunId, RunnableId, RunnableKind, RunnableSelection};
use bijux_dev_atlas::registry::RunnableRegistry;
use bijux_dev_atlas::runtime::{Capabilities, RealFs, RealProcessRunner};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct SuitesIndex {
    schema_version: u64,
    index_id: String,
    suites: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SuiteFile {
    schema_version: u64,
    suite_id: String,
    purpose: String,
    owner: String,
    stability: String,
    entries: Vec<SuiteFileEntry>,
}

#[derive(Debug, Deserialize)]
struct SuiteFileEntry {
    id: String,
    kind: String,
    mode: String,
    owner: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GroupRegistry {
    groups: Vec<GroupEntry>,
}

#[derive(Debug, Deserialize)]
struct GroupEntry {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ChecksRegistry {
    checks: Vec<CheckRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct CheckRegistryEntry {
    check_id: String,
    summary: String,
    group: String,
    commands: Vec<String>,
    report_ids: Vec<String>,
    reports: Vec<String>,
    tags: Option<Vec<String>>,
    retries: Option<u64>,
    overlaps_with: Option<Vec<String>>,
    requires_tools: Option<Vec<String>>,
    missing_tools_policy: Option<String>,
    severity: String,
    cpu_hint: Option<String>,
    mem_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContractsRegistry {
    contracts: Vec<ContractRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct ContractRegistryEntry {
    contract_id: String,
    summary: String,
    group: String,
    runner: String,
    reports: Vec<String>,
    tags: Option<Vec<String>>,
    retries: Option<u64>,
    overlaps_with: Option<Vec<String>>,
    requires_tools: Option<Vec<String>>,
    missing_tools_policy: Option<String>,
}

#[derive(Clone, Debug)]
struct SuiteTask {
    id: String,
    kind: String,
    summary: String,
    owner: String,
    mode: String,
    group: String,
    tags: Vec<String>,
    commands: Vec<String>,
    report_ids: Vec<String>,
    reports: Vec<String>,
    retries: u64,
    requires_tools: Vec<String>,
    missing_tools_policy: String,
    severity: String,
    cpu_hint: String,
    mem_hint: String,
}

#[derive(Clone, Debug)]
struct SuiteRunOptions {
    suite: String,
    repo_root: PathBuf,
    artifacts_root: PathBuf,
    run_id_override: Option<String>,
    jobs: String,
    fail_fast: Option<bool>,
    mode: SuiteModeArg,
    group: Option<String>,
    tag: Option<String>,
    format: SuiteOutputFormatArg,
    quiet: bool,
    verbose: bool,
    color: SuiteColorArg,
    strict: bool,
    out: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct TaskOutput {
    task: SuiteTask,
    status: String,
    duration_ms: u64,
    report_paths: Vec<String>,
    error_summary: Option<String>,
    stdout_path: String,
    stderr_path: String,
}

#[derive(Debug, Deserialize)]
struct PerfBudgetsConfig {
    schema_version: u64,
    budgets: Vec<PerfBudgetEntry>,
}

#[derive(Debug, Deserialize)]
struct PerfBudgetEntry {
    suite: String,
    duration_regression_ms: u64,
    duration_regression_ratio: f64,
}

#[derive(Debug, Deserialize)]
struct LatestRunsPointer {
    schema_version: u64,
    suites: BTreeMap<String, LatestSuiteRun>,
}

#[derive(Debug, Deserialize)]
struct LatestSuiteRun {
    run_id: String,
    suite_root: String,
}

#[derive(Debug, Deserialize)]
struct DefaultJobsPolicy {
    schema_version: u64,
    policy_id: String,
    suites: Vec<DefaultJobsEntry>,
}

#[derive(Debug, Deserialize)]
struct DefaultJobsEntry {
    suite_id: String,
    auto_jobs: u64,
    low_core_cap: u64,
    high_mem_parallelism: u64,
    effect_parallelism: u64,
}

#[derive(Clone, Debug)]
struct SchedulingLane {
    batch: usize,
    id: String,
    group: String,
    mode: String,
    severity: String,
    cpu_hint: String,
    mem_hint: String,
}

fn write_output_if_requested(out: Option<PathBuf>, rendered: &str) -> Result<(), String> {
    if let Some(path) = out {
        fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("cannot write {}: {err}", path.display()))?;
    }
    Ok(())
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn suites_index_path(root: &Path) -> PathBuf {
    root.join("configs/governance/suites/suites.index.json")
}

fn suite_file_path(root: &Path, suite_id: &str) -> PathBuf {
    root.join("configs/governance/suites")
        .join(format!("{suite_id}.suite.json"))
}

fn check_groups_path(root: &Path) -> PathBuf {
    root.join("configs/governance/check-groups.json")
}

fn contract_groups_path(root: &Path) -> PathBuf {
    root.join("configs/governance/contract-groups.json")
}

fn checks_registry_path(root: &Path) -> PathBuf {
    root.join("configs/governance/checks.registry.json")
}

fn contracts_registry_path(root: &Path) -> PathBuf {
    root.join("configs/governance/contracts.registry.json")
}

fn perf_budgets_path(root: &Path) -> PathBuf {
    root.join("configs/governance/perf-budgets.json")
}

fn default_jobs_policy_path(root: &Path) -> PathBuf {
    root.join("configs/governance/suites/default-jobs.json")
}

fn latest_runs_pointer_path(artifacts_root: &Path) -> PathBuf {
    artifacts_root.join("suites/LATEST")
}

fn suite_history_path(artifacts_root: &Path, suite_id: &str) -> PathBuf {
    artifacts_root
        .join("suites/history")
        .join(format!("{suite_id}.jsonl"))
}

fn suite_result_schema_name() -> &'static str {
    "suite-result.schema.json"
}

fn suite_summary_schema_name() -> &'static str {
    "suite-summary.schema.json"
}

fn suite_preflight_schema_name() -> &'static str {
    "suite-preflight.schema.json"
}

fn suite_diff_schema_name() -> &'static str {
    "suite-diff.schema.json"
}

fn suite_history_entry_schema_name() -> &'static str {
    "suite-history-entry.schema.json"
}

fn normalize_suite_id(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace('-', "_")
}

fn load_suites_index(root: &Path) -> Result<SuitesIndex, String> {
    let index: SuitesIndex = read_json_file(&suites_index_path(root))?;
    if index.schema_version != 1 || index.index_id != "governance-suites" {
        return Err(
            "suites index must declare schema_version=1 and index_id=governance-suites".to_string(),
        );
    }
    Ok(index)
}

fn load_suite(root: &Path, suite_id: &str) -> Result<SuiteFile, String> {
    let suite: SuiteFile = read_json_file(&suite_file_path(root, suite_id))?;
    if suite.schema_version != 1 {
        return Err(format!("suite `{suite_id}` must declare schema_version=1"));
    }
    Ok(suite)
}

fn group_order(root: &Path, suite_id: &str) -> Result<BTreeMap<String, usize>, String> {
    let groups: GroupRegistry = if suite_id == "contracts" {
        read_json_file(&contract_groups_path(root))?
    } else {
        read_json_file(&check_groups_path(root))?
    };
    Ok(groups
        .groups
        .into_iter()
        .enumerate()
        .map(|(idx, group)| (group.id, idx))
        .collect())
}

fn load_suite_tasks(root: &Path, suite_id: &str) -> Result<Vec<SuiteTask>, String> {
    let suite = load_suite(root, suite_id)?;
    if suite.suite_id != suite_id {
        return Err(format!(
            "suite file {} must declare suite_id `{suite_id}`",
            suite_file_path(root, suite_id).display()
        ));
    }
    let entry_map = suite
        .entries
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    match suite_id {
        "checks" => {
            let registry: ChecksRegistry = read_json_file(&checks_registry_path(root))?;
            registry
                .checks
                .into_iter()
                .filter_map(|entry| {
                    entry_map.get(&entry.check_id).map(|suite_entry| SuiteTask {
                        id: entry.check_id,
                        kind: suite_entry.kind.clone(),
                        summary: entry.summary,
                        owner: suite_entry.owner.clone(),
                        mode: suite_entry.mode.clone(),
                        group: entry.group,
                        tags: entry.tags.unwrap_or_else(|| suite_entry.tags.clone()),
                        commands: entry.commands,
                        report_ids: entry.report_ids,
                        reports: entry.reports,
                        retries: entry.retries.unwrap_or(0),
                        requires_tools: entry.requires_tools.unwrap_or_default(),
                        missing_tools_policy: entry
                            .missing_tools_policy
                            .unwrap_or_else(|| "fail".to_string()),
                        severity: entry.severity,
                        cpu_hint: entry.cpu_hint.unwrap_or_else(|| "light".to_string()),
                        mem_hint: entry.mem_hint.unwrap_or_else(|| "low".to_string()),
                    })
                })
                .collect::<Vec<_>>()
                .pipe(Ok)
        }
        "contracts" => {
            let registry: ContractsRegistry = read_json_file(&contracts_registry_path(root))?;
            registry
                .contracts
                .into_iter()
                .filter_map(|entry| {
                    entry_map
                        .get(&entry.contract_id)
                        .map(|suite_entry| SuiteTask {
                            id: entry.contract_id.clone(),
                            kind: suite_entry.kind.clone(),
                            summary: entry.summary,
                            owner: suite_entry.owner.clone(),
                            mode: suite_entry.mode.clone(),
                            group: entry.group,
                            tags: entry.tags.unwrap_or_else(|| suite_entry.tags.clone()),
                            commands: vec![normalized_contract_command(
                                &entry.runner,
                                &suite_entry.mode,
                                &entry.contract_id,
                            )],
                            report_ids: Vec::new(),
                            reports: entry.reports,
                            retries: entry.retries.unwrap_or(0),
                            requires_tools: entry.requires_tools.unwrap_or_default(),
                            missing_tools_policy: entry
                                .missing_tools_policy
                                .unwrap_or_else(|| "fail".to_string()),
                            severity: "blocker".to_string(),
                            cpu_hint: "moderate".to_string(),
                            mem_hint: if suite_entry.mode == "effect" {
                                "high".to_string()
                            } else {
                                "medium".to_string()
                            },
                        })
                })
                .collect::<Vec<_>>()
                .pipe(Ok)
        }
        other => Err(format!(
            "suite `{other}` is not runnable yet; expected checks or contracts"
        )),
    }
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}

fn normalized_contract_command(command: &str, mode: &str, contract_id: &str) -> String {
    if command == "bijux dev atlas ops check" {
        format!(
            "bijux dev atlas contract run --domain ops --mode {} --only {}",
            if mode == "effect" { "effect" } else { "static" },
            contract_id
        )
    } else {
        command.replace("--only-contract", "--only")
    }
}

fn load_default_jobs_policy(root: &Path) -> Result<DefaultJobsPolicy, String> {
    let policy: DefaultJobsPolicy = read_json_file(&default_jobs_policy_path(root))?;
    if policy.schema_version != 1 || policy.policy_id != "suite-default-jobs" {
        return Err(
            "default jobs policy must declare schema_version=1 and policy_id=suite-default-jobs"
                .to_string(),
        );
    }
    Ok(policy)
}

fn parse_jobs(root: &Path, suite_id: &str, raw: &str) -> Result<usize, String> {
    if raw.eq_ignore_ascii_case("auto") {
        let policy = load_default_jobs_policy(root)?;
        let suite_policy = policy
            .suites
            .into_iter()
            .find(|entry| entry.suite_id == suite_id)
            .ok_or_else(|| format!("default jobs policy missing suite `{suite_id}`"))?;
        let cores = std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1);
        let low_core_cap = suite_policy.low_core_cap as usize;
        let auto_jobs = suite_policy.auto_jobs as usize;
        let jobs = if cores <= 2 {
            cores.min(low_core_cap).max(1)
        } else {
            cores.min(auto_jobs).max(1)
        };
        return Ok(jobs);
    }
    let jobs = raw
        .parse::<usize>()
        .map_err(|err| format!("invalid jobs value `{raw}`: {err}"))?;
    if jobs == 0 {
        return Err("jobs must be at least 1".to_string());
    }
    Ok(jobs)
}

fn suite_effect_parallelism(root: &Path, suite_id: &str) -> Result<usize, String> {
    let policy = load_default_jobs_policy(root)?;
    let suite_policy = policy
        .suites
        .into_iter()
        .find(|entry| entry.suite_id == suite_id)
        .ok_or_else(|| format!("default jobs policy missing suite `{suite_id}`"))?;
    Ok(suite_policy.effect_parallelism as usize)
}

fn suite_high_mem_parallelism(root: &Path, suite_id: &str) -> Result<usize, String> {
    let policy = load_default_jobs_policy(root)?;
    let suite_policy = policy
        .suites
        .into_iter()
        .find(|entry| entry.suite_id == suite_id)
        .ok_or_else(|| format!("default jobs policy missing suite `{suite_id}`"))?;
    Ok(suite_policy.high_mem_parallelism as usize)
}

fn known_commands_local(root: &Path) -> Result<BTreeSet<String>, String> {
    let inventory: serde_json::Value = read_json_file(&root.join("make/target-list.json"))?;
    let targets = inventory["public_targets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(|target| format!("make {target}"))
        .collect::<BTreeSet<_>>();
    let mut commands = targets;
    commands.extend([
        "bijux dev atlas contract run --domain ops --mode static".to_string(),
        "bijux dev atlas contract run --domain ops --mode effect".to_string(),
        "bijux dev atlas contract run --domain ops --mode static --only".to_string(),
        "bijux dev atlas contract run --domain ops --mode effect --only".to_string(),
    ]);
    let checks_registry: serde_json::Value = read_json_file(&checks_registry_path(root))?;
    for check in checks_registry["checks"].as_array().into_iter().flatten() {
        for command in check["commands"].as_array().into_iter().flatten() {
            if let Some(command) = command.as_str() {
                commands.insert(command.to_string());
            }
        }
    }
    let contracts_registry: serde_json::Value = read_json_file(&contracts_registry_path(root))?;
    for contract in contracts_registry["contracts"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if let Some(runner) = contract["runner"].as_str() {
            commands.insert(runner.to_string());
            commands.insert(runner.replace("--only-contract", "--only"));
        }
    }
    Ok(commands)
}

fn parse_command_invocation(command: &str) -> Result<(String, Vec<String>), String> {
    let parts = command
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return Err("command cannot be empty".to_string());
    }
    if parts.len() >= 3 && parts[0] == "bijux" && parts[1] == "dev" && parts[2] == "atlas" {
        let exe =
            std::env::current_exe().map_err(|err| format!("resolve current exe failed: {err}"))?;
        return Ok((
            exe.display().to_string(),
            parts.into_iter().skip(3).collect::<Vec<_>>(),
        ));
    }
    Ok((
        parts[0].clone(),
        parts.into_iter().skip(1).collect::<Vec<_>>(),
    ))
}

fn git_sha(root: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output();
    match output {
        Ok(value) if value.status.success() => {
            String::from_utf8_lossy(&value.stdout).trim().to_string()
        }
        _ => "nogit".to_string(),
    }
}

fn deterministic_run_id(
    root: &Path,
    suite_id: &str,
    mode: SuiteModeArg,
    group: Option<&str>,
    tag: Option<&str>,
) -> Result<String, String> {
    let suite_text = fs::read_to_string(suite_file_path(root, suite_id))
        .map_err(|err| format!("read suite file failed: {err}"))?;
    let registry_text = if suite_id == "contracts" {
        fs::read_to_string(contracts_registry_path(root))
            .map_err(|err| format!("read contracts registry failed: {err}"))?
    } else {
        fs::read_to_string(checks_registry_path(root))
            .map_err(|err| format!("read checks registry failed: {err}"))?
    };
    let mut hasher = Sha256::new();
    hasher.update(suite_id.as_bytes());
    hasher.update(format!("{mode:?}").as_bytes());
    if let Some(group) = group {
        hasher.update(group.as_bytes());
    }
    if let Some(tag) = tag {
        hasher.update(tag.as_bytes());
    }
    hasher.update(git_sha(root).as_bytes());
    hasher.update(suite_text.as_bytes());
    hasher.update(registry_text.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    Ok(format!("{suite_id}-{}", &digest[..12]))
}

fn run_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}

fn supports_color(color: SuiteColorArg) -> bool {
    match color {
        SuiteColorArg::Always => true,
        SuiteColorArg::Never => false,
        SuiteColorArg::Auto => stdout().is_terminal(),
    }
}

fn load_perf_budget(root: &Path, suite_id: &str) -> Result<PerfBudgetEntry, String> {
    let schema_value: serde_json::Value =
        read_json_file(&root.join("configs/schemas/contracts/governance/perf-budgets.schema.json"))?;
    let required_fields = schema_value["required"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let config_value: serde_json::Value = read_json_file(&perf_budgets_path(root))?;
    let config_object = config_value
        .as_object()
        .ok_or_else(|| "perf budgets config must be a JSON object".to_string())?;
    for field in required_fields {
        if !config_object.contains_key(field) {
            return Err(format!(
                "perf budgets config missing required key `{field}`"
            ));
        }
    }
    let config: PerfBudgetsConfig = serde_json::from_value(config_value)
        .map_err(|err| format!("parse perf budgets failed: {err}"))?;
    if config.schema_version != 1 {
        return Err("perf budgets must declare schema_version=1".to_string());
    }
    config
        .budgets
        .into_iter()
        .find(|entry| entry.suite == suite_id)
        .ok_or_else(|| format!("perf budget for suite `{suite_id}` not found"))
}

fn load_latest_runs_pointer(artifacts_root: &Path) -> Result<LatestRunsPointer, String> {
    let path = latest_runs_pointer_path(artifacts_root);
    if !path.exists() {
        return Ok(LatestRunsPointer {
            schema_version: 1,
            suites: BTreeMap::new(),
        });
    }
    let pointer: LatestRunsPointer = read_json_file(&path)?;
    if pointer.schema_version != 1 {
        return Err("LATEST pointer must declare schema_version=1".to_string());
    }
    Ok(pointer)
}

fn write_latest_run_pointer(
    artifacts_root: &Path,
    suite_id: &str,
    run_id: &str,
    suite_root: &Path,
) -> Result<(), String> {
    let mut pointer = load_latest_runs_pointer(artifacts_root)?;
    pointer.suites.insert(
        suite_id.to_string(),
        LatestSuiteRun {
            run_id: run_id.to_string(),
            suite_root: suite_root.display().to_string(),
        },
    );
    let rendered = serde_json::to_string_pretty(&serde_json::json!({
        "schema_version": 1,
        "suites": pointer.suites.into_iter().map(|(suite, entry)| (suite, serde_json::json!({
            "run_id": entry.run_id,
            "suite_root": entry.suite_root,
        }))).collect::<serde_json::Map<_, _>>()
    }))
    .map_err(|err| format!("encode LATEST pointer failed: {err}"))?;
    fs::create_dir_all(artifacts_root.join("suites")).map_err(|err| {
        format!(
            "create {} failed: {err}",
            artifacts_root.join("suites").display()
        )
    })?;
    fs::write(
        latest_runs_pointer_path(artifacts_root),
        format!("{rendered}\n"),
    )
    .map_err(|err| format!("write LATEST pointer failed: {err}"))
}

fn append_suite_history_entries(
    repo_root: &Path,
    artifacts_root: &Path,
    suite_id: &str,
    run_id: &str,
    task_outputs: &[TaskOutput],
) -> Result<(), String> {
    let history_path = suite_history_path(artifacts_root, suite_id);
    if let Some(parent) = history_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let mut content = if history_path.exists() {
        fs::read_to_string(&history_path)
            .map_err(|err| format!("read {} failed: {err}", history_path.display()))?
    } else {
        String::new()
    };
    for task_output in task_outputs {
        let entry = serde_json::json!({
            "report_id": "suite-history-entry",
            "version": 1,
            "inputs": {
                "suite": suite_id,
                "run_id": run_id,
            },
            "suite": suite_id,
            "run_id": run_id,
            "task_id": task_output.task.id,
            "group": task_output.task.group,
            "mode": task_output.task.mode,
            "status": task_output.status,
            "duration_ms": task_output.duration_ms,
            "timestamp": run_timestamp(),
            "result_path": artifacts_root.join("suites").join(suite_id).join(run_id).join(&task_output.task.id).join("result.json").display().to_string(),
        });
        validate_named_report(repo_root, suite_history_entry_schema_name(), &entry)?;
        content.push_str(
            &serde_json::to_string(&entry)
                .map_err(|err| format!("encode suite history entry failed: {err}"))?,
        );
        content.push('\n');
    }
    let temp_path = history_path.with_extension("jsonl.tmp");
    fs::write(&temp_path, content)
        .map_err(|err| format!("write {} failed: {err}", temp_path.display()))?;
    fs::rename(&temp_path, &history_path)
        .map_err(|err| format!("rename {} failed: {err}", history_path.display()))
}

fn render_suite_list(root: &Path, format: FormatArg) -> Result<String, String> {
    let index = load_suites_index(root)?;
    let suite_ids = index
        .suites
        .into_iter()
        .map(|entry| entry.trim_end_matches(".suite.json").to_string())
        .collect::<Vec<_>>();
    match format {
        FormatArg::Text => Ok(suite_ids.join("\n")),
        FormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "suites": suite_ids
        }))
        .map_err(|err| format!("encode suites list failed: {err}")),
        FormatArg::Jsonl => Err("jsonl output is not supported for suites list".to_string()),
    }
}

fn render_suite_describe(root: &Path, suite_id: &str, format: FormatArg) -> Result<String, String> {
    let suite = load_suite(root, suite_id)?;
    let tasks = load_suite_tasks(root, suite_id)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "suite_id": suite.suite_id,
        "purpose": suite.purpose,
        "owner": suite.owner,
        "stability": suite.stability,
        "entries": tasks.into_iter().map(|task| serde_json::json!({
            "id": task.id,
            "kind": task.kind,
            "mode": task.mode,
            "group": task.group,
            "owner": task.owner,
            "tags": task.tags,
            "summary": task.summary,
            "commands": task.commands,
            "reports": task.reports,
        })).collect::<Vec<_>>()
    });
    match format {
        FormatArg::Text => {
            let mut lines = vec![
                format!(
                    "suite_id: {}",
                    payload["suite_id"].as_str().unwrap_or_default()
                ),
                format!(
                    "purpose: {}",
                    payload["purpose"].as_str().unwrap_or_default()
                ),
                format!("owner: {}", payload["owner"].as_str().unwrap_or_default()),
                format!(
                    "stability: {}",
                    payload["stability"].as_str().unwrap_or_default()
                ),
                "entries:".to_string(),
            ];
            for entry in payload["entries"].as_array().into_iter().flatten() {
                lines.push(format!(
                    "- {} [{}|{}|{}] {}",
                    entry["id"].as_str().unwrap_or_default(),
                    entry["group"].as_str().unwrap_or_default(),
                    entry["mode"].as_str().unwrap_or_default(),
                    entry["kind"].as_str().unwrap_or_default(),
                    entry["summary"].as_str().unwrap_or_default()
                ));
            }
            Ok(lines.join("\n"))
        }
        FormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode suites describe failed: {err}")),
        FormatArg::Jsonl => Err("jsonl output is not supported for suites describe".to_string()),
    }
}

fn render_suite_lint(root: &Path, format: FormatArg) -> Result<(String, i32), String> {
    let checks = load_suite(root, "checks")?;
    let contracts = load_suite(root, "contracts")?;
    let checks_registry: ChecksRegistry = read_json_file(&checks_registry_path(root))?;
    let contracts_registry: ContractsRegistry = read_json_file(&contracts_registry_path(root))?;
    let allowed_overlaps = checks_registry
        .checks
        .into_iter()
        .flat_map(|entry| {
            entry
                .overlaps_with
                .unwrap_or_default()
                .into_iter()
                .map(move |overlap| (entry.check_id.clone(), overlap))
        })
        .chain(contracts_registry.contracts.into_iter().flat_map(|entry| {
            entry
                .overlaps_with
                .unwrap_or_default()
                .into_iter()
                .map(move |overlap| (entry.contract_id.clone(), overlap))
        }))
        .collect::<BTreeSet<_>>();
    let check_ids = checks
        .entries
        .into_iter()
        .map(|entry| entry.id)
        .collect::<BTreeSet<_>>();
    let contract_ids = contracts
        .entries
        .into_iter()
        .map(|entry| entry.id)
        .collect::<BTreeSet<_>>();
    let overlaps = check_ids
        .intersection(&contract_ids)
        .filter(|id| !allowed_overlaps.contains(&((*id).clone(), (*id).clone())))
        .cloned()
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "lint_id": "suite-membership-lint",
        "summary": {
            "duplicate_ids": overlaps.len(),
        },
        "duplicate_ids": overlaps,
    });
    let rendered = match format {
        FormatArg::Text => {
            if payload["summary"]["duplicate_ids"].as_u64().unwrap_or(0) == 0 {
                "suite lint passed".to_string()
            } else {
                let mut lines =
                    vec!["suite lint found duplicate ids across checks and contracts".to_string()];
                lines.extend(
                    payload["duplicate_ids"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(serde_json::Value::as_str)
                        .map(|value| format!("  {value}")),
                );
                lines.join("\n")
            }
        }
        FormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode suite lint failed: {err}"))?,
        FormatArg::Jsonl => Err("jsonl output is not supported for suites lint".to_string())?,
    };
    let exit = if payload["summary"]["duplicate_ids"].as_u64().unwrap_or(0) == 0 {
        0
    } else {
        1
    };
    Ok((rendered, exit))
}

fn colorize(label: &str, color: SuiteColorArg, code: &str) -> String {
    if supports_color(color) {
        format!("\u{1b}[{code}m{label}\u{1b}[0m")
    } else {
        label.to_string()
    }
}

fn status_label(status: &str, color: SuiteColorArg) -> String {
    match status {
        "pass" => colorize("[PASS]", color, "32"),
        "fail" => colorize("[FAIL]", color, "31"),
        "warn" => colorize("[WARN]", color, "33"),
        "skip" => colorize("[SKIP]", color, "33"),
        _ => format!("[{}]", status.to_ascii_uppercase()),
    }
}

fn format_duration_ms(duration_ms: u64) -> String {
    if duration_ms >= 1_000 {
        format!("{:.2}s", duration_ms as f64 / 1_000.0)
    } else {
        format!("{duration_ms}ms")
    }
}

fn suite_root_path(artifacts_root: &Path, suite_id: &str, run_id: &str) -> PathBuf {
    artifacts_root.join("suites").join(suite_id).join(run_id)
}

fn read_suite_summary(suite_root: &Path) -> Result<serde_json::Value, String> {
    read_json_file(&suite_root.join("suite-summary.json"))
}

fn read_result_value(result_path: &Path) -> Result<serde_json::Value, String> {
    read_json_file(result_path)
}

fn filter_summary_tasks(
    summary: &serde_json::Value,
    failed_only: bool,
    group: Option<&str>,
    id: Option<&str>,
) -> Vec<serde_json::Value> {
    summary["tasks"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|task| {
            (!failed_only || task["status"].as_str() == Some("fail"))
                && group.is_none_or(|value| task["group"].as_str() == Some(value))
                && id.is_none_or(|value| task["id"].as_str() == Some(value))
        })
        .cloned()
        .collect::<Vec<_>>()
}

#[allow(clippy::too_many_arguments)]
fn human_suite_report(
    summary: &serde_json::Value,
    task_details: &[serde_json::Value],
    suite_root: &Path,
    quiet: bool,
    verbose: bool,
    color: SuiteColorArg,
    slowdown_rows: &[String],
    registry_footer: Option<&str>,
) -> String {
    let suite = summary["suite"].as_str().unwrap_or_default();
    let run_id = summary["run_id"].as_str().unwrap_or_default();
    let elapsed_ms = summary["elapsed_ms"].as_u64().unwrap_or(0);
    let mut lines = vec![format!("Suite {suite}  run_id={run_id}")];
    let mut grouped = BTreeMap::<String, Vec<&serde_json::Value>>::new();
    for task in task_details {
        let group = task["group"].as_str().unwrap_or("ungrouped").to_string();
        grouped.entry(group).or_default().push(task);
    }
    for (group, tasks) in grouped {
        lines.push(format!("Group {group}"));
        for task in tasks {
            let status = task["status"].as_str().unwrap_or("unknown");
            let severity = task["severity"].as_str().unwrap_or("blocker");
            let rendered_status = if status == "fail" && severity == "info" {
                "warn"
            } else {
                status
            };
            if quiet && rendered_status == "pass" {
                continue;
            }
            let task_id = task["id"].as_str().unwrap_or_default();
            let duration_ms = task["duration_ms"].as_u64().unwrap_or(0);
            let summary_text = task["summary"].as_str().unwrap_or_default();
            lines.push(format!(
                "{} {} - {} ({})",
                status_label(rendered_status, color),
                task_id,
                summary_text,
                format_duration_ms(duration_ms)
            ));
            if rendered_status == "fail" || rendered_status == "warn" {
                let short_error = task["error_summary"].as_str().unwrap_or("command failed");
                lines.push(format!(
                    "  {}: {short_error}",
                    if rendered_status == "warn" {
                        "warning"
                    } else {
                        "error"
                    }
                ));
                let result_path = task["result_path"].as_str().unwrap_or_default();
                if !result_path.is_empty() {
                    lines.push(format!("  artifacts: {result_path}"));
                }
                if verbose && rendered_status == "fail" {
                    let stderr_path = task["stderr_path"].as_str().unwrap_or_default();
                    if !stderr_path.is_empty() {
                        let tail = fs::read_to_string(stderr_path)
                            .unwrap_or_default()
                            .lines()
                            .take(10)
                            .map(str::to_string)
                            .collect::<Vec<_>>();
                        if !tail.is_empty() {
                            lines.push("  stderr_tail:".to_string());
                            for line in tail {
                                lines.push(format!("    {line}"));
                            }
                        }
                    }
                }
            }
        }
    }
    if !slowdown_rows.is_empty() {
        lines.push("Slowdowns".to_string());
        lines.extend(slowdown_rows.iter().map(|row| format!("  {row}")));
    }
    lines.push(format!(
        "Totals pass={} fail={} warn={} skip={} total={} elapsed={} run_id={} artifacts={}",
        summary["summary"]["pass"].as_u64().unwrap_or(0),
        summary["summary"]["fail"].as_u64().unwrap_or(0),
        summary["summary"]["warn"].as_u64().unwrap_or(0),
        summary["summary"]["skip"].as_u64().unwrap_or(0),
        summary["summary"]["total"].as_u64().unwrap_or(0),
        format_duration_ms(elapsed_ms),
        run_id,
        suite_root.display()
    ));
    if let Some(footer) = registry_footer {
        lines.push(footer.to_string());
    }
    lines.join("\n")
}

fn registry_completeness_footer(root: &Path) -> Result<String, String> {
    let checks: serde_json::Value = read_json_file(&checks_registry_path(root))?;
    let contracts: serde_json::Value = read_json_file(&contracts_registry_path(root))?;
    let known_commands = known_commands_local(root)?;
    let mut total = 0usize;
    let mut missing = 0usize;
    for row in checks["checks"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(contracts["contracts"].as_array().into_iter().flatten())
    {
        total += 1;
        let owner_missing = row["owner"]
            .as_str()
            .is_none_or(|value| value.trim().is_empty());
        let reports_missing = row["reports"]
            .as_array()
            .is_none_or(|value| value.is_empty());
        let membership_missing = row["suite_membership"]
            .as_array()
            .is_none_or(|value| value.is_empty());
        let command_missing = row
            .get("commands")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|value| value.is_empty())
            || row
                .get("runner")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.trim().is_empty());
        let broken_command = row
            .get("commands")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .any(|command| !known_commands.contains(command))
            || row
                .get("runner")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|runner| {
                    !runner.trim().is_empty() && !known_commands.contains(runner)
                });
        if owner_missing
            || reports_missing
            || membership_missing
            || command_missing
            || broken_command
        {
            missing += 1;
        }
    }
    Ok(format!(
        "Registry completeness total={} fully_specified={} work_remaining={}",
        total,
        total.saturating_sub(missing),
        missing
    ))
}

fn select_tasks(
    root: &Path,
    suite_id: &str,
    mode: SuiteModeArg,
    group: Option<&str>,
    tag: Option<&str>,
) -> Result<Vec<SuiteTask>, String> {
    let mut tasks = load_suite_tasks(root, suite_id)?;
    let group_order = group_order(root, suite_id)?;
    tasks.retain(|task| match mode {
        SuiteModeArg::Pure => task.mode == "pure",
        SuiteModeArg::Effect => task.mode == "effect",
        SuiteModeArg::All => true,
    });
    if let Some(group_filter) = group {
        tasks.retain(|task| task.group == group_filter);
    }
    if let Some(tag_filter) = tag {
        tasks.retain(|task| task.tags.iter().any(|value| value == tag_filter));
    }
    tasks.sort_by(|a, b| {
        let left = group_order.get(&a.group).copied().unwrap_or(usize::MAX);
        let right = group_order.get(&b.group).copied().unwrap_or(usize::MAX);
        left.cmp(&right).then_with(|| a.id.cmp(&b.id))
    });
    Ok(tasks)
}

fn can_share_batch(
    suite_id: &str,
    batch: &[SuiteTask],
    candidate: &SuiteTask,
    high_mem_parallelism: usize,
    effect_parallelism: usize,
) -> bool {
    if batch.is_empty() {
        return true;
    }
    let high_mem_in_batch = batch.iter().filter(|task| task.mem_hint == "high").count();
    if candidate.mem_hint == "high" && high_mem_in_batch >= high_mem_parallelism.max(1) {
        return false;
    }
    if suite_id == "contracts" && candidate.mode == "effect" {
        let effect_in_batch = batch.iter().filter(|task| task.mode == "effect").count();
        if effect_in_batch >= effect_parallelism.max(1) {
            return false;
        }
    }
    true
}

fn build_execution_plan(
    root: &Path,
    suite_id: &str,
    tasks: Vec<SuiteTask>,
    jobs: usize,
) -> Result<(Vec<Vec<SuiteTask>>, Vec<SchedulingLane>), String> {
    let high_mem_parallelism = suite_high_mem_parallelism(root, suite_id)?;
    let effect_parallelism = suite_effect_parallelism(root, suite_id)?;
    let ordered_tasks = if suite_id == "checks" {
        let mut grouped = BTreeMap::<String, VecDeque<SuiteTask>>::new();
        for task in tasks {
            grouped
                .entry(task.group.clone())
                .or_default()
                .push_back(task);
        }
        let mut round_robin = Vec::new();
        loop {
            let mut advanced = false;
            for queue in grouped.values_mut() {
                if let Some(task) = queue.pop_front() {
                    round_robin.push(task);
                    advanced = true;
                }
            }
            if !advanced {
                break;
            }
        }
        round_robin
    } else {
        tasks
    };
    let mut batches = Vec::<Vec<SuiteTask>>::new();
    let mut plan = Vec::<SchedulingLane>::new();
    for task in ordered_tasks {
        let mut placed = false;
        for (batch_index, batch) in batches.iter_mut().enumerate() {
            if batch.len() >= jobs {
                continue;
            }
            if !can_share_batch(
                suite_id,
                batch,
                &task,
                high_mem_parallelism,
                effect_parallelism,
            ) {
                continue;
            }
            batch.push(task.clone());
            plan.push(SchedulingLane {
                batch: batch_index,
                id: task.id.clone(),
                group: task.group.clone(),
                mode: task.mode.clone(),
                severity: task.severity.clone(),
                cpu_hint: task.cpu_hint.clone(),
                mem_hint: task.mem_hint.clone(),
            });
            placed = true;
            break;
        }
        if !placed {
            batches.push(vec![task.clone()]);
            plan.push(SchedulingLane {
                batch: batches.len() - 1,
                id: task.id.clone(),
                group: task.group.clone(),
                mode: task.mode.clone(),
                severity: task.severity.clone(),
                cpu_hint: task.cpu_hint.clone(),
                mem_hint: task.mem_hint.clone(),
            });
        }
    }
    Ok((batches, plan))
}

fn task_report_paths(task: &SuiteTask, task_root: &Path) -> Vec<String> {
    task.reports
        .iter()
        .map(|report| task_root.join(report).display().to_string())
        .collect()
}

fn check_report_schema_path(repo_root: &Path, report_id: &str) -> PathBuf {
    repo_root
        .join("configs/schemas/contracts/reports/checks")
        .join(format!("{report_id}.schema.json"))
}

fn source_artifact_root(task_root: &Path) -> PathBuf {
    task_root.join("_source_artifacts")
}

fn source_report_path(task_root: &Path, rel: &str) -> PathBuf {
    source_artifact_root(task_root).join(rel)
}

fn extract_json_from_text(text: &str) -> Option<serde_json::Value> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(value) = serde_json::from_str(trimmed) {
        return Some(value);
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&trimmed[start..=end]).ok()
}

fn read_optional_json(path: &Path) -> Option<serde_json::Value> {
    let text = fs::read_to_string(path).ok()?;
    extract_json_from_text(&text)
}

fn read_optional_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn pinned_tool_version(repo_root: &Path, tool: &str) -> Option<String> {
    if matches!(tool, "rustfmt" | "clippy" | "cargo") {
        let toolchain = fs::read_to_string(repo_root.join("rust-toolchain.toml")).ok()?;
        return toolchain
            .lines()
            .find_map(|line| line.trim().strip_prefix("channel = "))
            .map(|value| value.trim_matches('"').to_string());
    }
    if tool == "bijux-dev-atlas" {
        return Some(git_sha(repo_root));
    }
    let payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root.join("configs/ops/pins/tools.json")).ok()?,
    )
    .ok()?;
    payload
        .get("tools")
        .and_then(|value| value.get(tool))
        .and_then(|value| value.get("version"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn actual_tool_version(repo_root: &Path, tool: &str) -> Option<String> {
    let command = match tool {
        "rustfmt" => ("rustfmt", vec!["--version"]),
        "clippy" => ("cargo", vec!["clippy", "--version"]),
        "cargo" => ("cargo", vec!["--version"]),
        "cargo-audit" => ("cargo", vec!["audit", "--version"]),
        "cargo-deny" => ("cargo", vec!["deny", "--version"]),
        "helm" => ("helm", vec!["version", "--short"]),
        "kubeconform" => ("kubeconform", vec!["-v"]),
        _ => return None,
    };
    Command::new(command.0)
        .args(&command.1)
        .current_dir(repo_root)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn tool_version_rows(repo_root: &Path, tools: &[&str]) -> Vec<serde_json::Value> {
    tools
        .iter()
        .map(|tool| {
            let expected = pinned_tool_version(repo_root, tool);
            let actual = actual_tool_version(repo_root, tool).or_else(|| expected.clone());
            serde_json::json!({
                "tool": tool,
                "expected_version": expected,
                "actual_version": actual,
                "pinned": expected.is_some(),
            })
        })
        .collect()
}

fn report_payload_for_check(
    repo_root: &Path,
    task_root: &Path,
    task: &SuiteTask,
    report_id: &str,
    stdout: &str,
    stderr: &str,
    status: &str,
) -> serde_json::Value {
    let source_root = source_artifact_root(task_root);
    let base = serde_json::json!({
        "version": 1,
        "inputs": {
            "check_id": task.id,
            "commands": task.commands,
            "source_artifact_root": source_root.display().to_string(),
        },
        "check_id": task.id,
        "summary": task.summary,
        "status": status,
        "artifacts": {
            "stdout": task_root.join("stdout.log").display().to_string(),
            "stderr": task_root.join("stderr.log").display().to_string(),
        },
    });
    match report_id {
        "check-rustfmt" => {
            let source =
                read_optional_text(&source_report_path(task_root, "fmt/suite-run/report.txt"))
                    .and_then(|text| extract_json_from_text(&text));
            serde_json::json!({
                "report_id": report_id,
                "version": 1,
                "inputs": base["inputs"],
                "check_id": task.id,
                "summary": task.summary,
                "status": status,
                "tool_versions": tool_version_rows(repo_root, &["cargo", "rustfmt"]),
                "files": source
                    .as_ref()
                    .and_then(|value| value.get("stderr"))
                    .and_then(serde_json::Value::as_str)
                    .map(|text| text.lines().filter(|line| line.contains(".rs")).map(ToString::to_string).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "source_report": source,
                "artifacts": base["artifacts"],
            })
        }
        "check-clippy" => {
            let source =
                read_optional_json(&source_report_path(task_root, "lint/suite-run/report.json"));
            let stderr_text = source
                .as_ref()
                .and_then(|value| value.get("stderr"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(stderr);
            let top_messages = stderr_text
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(5)
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            serde_json::json!({
                "report_id": report_id,
                "version": 1,
                "inputs": base["inputs"],
                "check_id": task.id,
                "summary": task.summary,
                "status": status,
                "tool_versions": tool_version_rows(repo_root, &["cargo", "clippy"]),
                "diagnostics": {
                    "error_lines": stderr_text.lines().filter(|line| line.contains("error")).count(),
                    "warning_lines": stderr_text.lines().filter(|line| line.contains("warning")).count(),
                    "top_messages": top_messages,
                },
                "source_report": source,
                "artifacts": base["artifacts"],
            })
        }
        "check-config-format" => {
            let source = if task.id == "CHECK-CONFIGS-LINT-001" {
                read_optional_json(&source_report_path(
                    task_root,
                    "configs-lint/suite-run/report.json",
                ))
            } else {
                None
            };
            let combined = format!("{stdout}\n{stderr}");
            serde_json::json!({
                "report_id": report_id,
                "version": 1,
                "inputs": base["inputs"],
                "check_id": task.id,
                "summary": task.summary,
                "status": status,
                "tool_versions": tool_version_rows(repo_root, &["bijux-dev-atlas"]),
                "violations": combined.lines().filter(|line| line.contains("ERROR") || line.contains("WARN")).map(ToString::to_string).take(20).collect::<Vec<_>>(),
                "source_report": source,
                "artifacts": base["artifacts"],
            })
        }
        "check-docs-links" => {
            let source = read_optional_json(&source_report_path(
                task_root,
                "docs-validate/suite-run/report.json",
            ));
            let links = source
                .as_ref()
                .and_then(|value| value.get("checks"))
                .and_then(|value| value.get("links"))
                .cloned();
            serde_json::json!({
                "report_id": report_id,
                "version": 1,
                "inputs": base["inputs"],
                "check_id": task.id,
                "summary": task.summary,
                "status": status,
                "tool_versions": tool_version_rows(repo_root, &["bijux-dev-atlas"]),
                "links": links,
                "source_report": source,
                "artifacts": base["artifacts"],
            })
        }
        "check-helm-lint" | "check-kubeconform" => {
            let source = read_optional_json(&source_report_path(
                task_root,
                "k8s-validate/suite-run/report.json",
            ));
            let summary = source
                .as_ref()
                .and_then(|value| value.get("summary"))
                .cloned();
            serde_json::json!({
                "report_id": report_id,
                "version": 1,
                "inputs": base["inputs"],
                "check_id": task.id,
                "summary": task.summary,
                "status": status,
                "tool_versions": tool_version_rows(repo_root, &["helm", "kubeconform"]),
                "k8s_summary": summary,
                "source_report": source,
                "artifacts": base["artifacts"],
            })
        }
        "check-deps" => {
            let tool = if task.id.contains("DENY") {
                "cargo-deny"
            } else {
                "cargo-audit"
            };
            serde_json::json!({
                "report_id": report_id,
                "version": 1,
                "inputs": base["inputs"],
                "check_id": task.id,
                "summary": task.summary,
                "status": status,
                "tool_versions": tool_version_rows(repo_root, &[tool]),
                "stderr_excerpt": stderr.lines().take(20).collect::<Vec<_>>(),
                "stdout_excerpt": stdout.lines().take(20).collect::<Vec<_>>(),
                "artifacts": base["artifacts"],
            })
        }
        "check-suite-summary" => {
            let source = [
                "ops-fast/suite-run/report.json",
                "ops-pr/suite-run/report.json",
                "ops-nightly/suite-run/report.json",
            ]
            .into_iter()
            .find_map(|rel| read_optional_json(&source_report_path(task_root, rel)))
            .or_else(|| extract_json_from_text(stdout))
            .or_else(|| extract_json_from_text(stderr));
            serde_json::json!({
                "report_id": report_id,
                "version": 1,
                "inputs": base["inputs"],
                "check_id": task.id,
                "summary": task.summary,
                "status": status,
                "tool_versions": tool_version_rows(repo_root, &["bijux-dev-atlas"]),
                "source_report": source,
                "artifacts": base["artifacts"],
            })
        }
        _ => serde_json::json!({
            "report_id": report_id,
            "version": 1,
            "inputs": base["inputs"],
            "check_id": task.id,
            "summary": task.summary,
            "status": status,
            "artifacts": base["artifacts"],
        }),
    }
}

fn write_governed_check_reports(
    repo_root: &Path,
    task_root: &Path,
    task: &SuiteTask,
    stdout: &str,
    stderr: &str,
    status: &str,
) -> Result<Vec<String>, String> {
    let mut report_paths = Vec::new();
    for (report_id, report_rel) in task.report_ids.iter().zip(task.reports.iter()) {
        let payload = report_payload_for_check(
            repo_root, task_root, task, report_id, stdout, stderr, status,
        );
        let unpinned_tools = payload
            .get("tool_versions")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter(|row| row.get("pinned").and_then(serde_json::Value::as_bool) == Some(false))
            .filter_map(|row| row.get("tool").and_then(serde_json::Value::as_str))
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if !unpinned_tools.is_empty() {
            return Err(format!(
                "{} uses unpinned tools in report `{}`: {}",
                task.id,
                report_id,
                unpinned_tools.join(", ")
            ));
        }
        let schema_path = check_report_schema_path(repo_root, report_id);
        validate_report_value_against_schema(&payload, &schema_path)?;
        let report_path = task_root.join(report_rel);
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
        }
        let rendered = serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode {report_id} failed: {err}"))?;
        fs::write(&report_path, format!("{rendered}\n"))
            .map_err(|err| format!("write {} failed: {err}", report_path.display()))?;
        report_paths.push(report_path.display().to_string());
    }
    Ok(report_paths)
}

fn tool_in_path(tool: &str) -> bool {
    if tool == "bijux-dev-atlas" {
        return true;
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|entry| entry.join(tool).is_file())
}

fn suite_preflight_report(suite_id: &str, run_id: &str, tasks: &[SuiteTask]) -> serde_json::Value {
    let rows = tasks
        .iter()
        .map(|task| {
            let missing_tools = task
                .requires_tools
                .iter()
                .filter(|tool| !tool_in_path(tool))
                .cloned()
                .collect::<Vec<_>>();
            serde_json::json!({
                "id": task.id,
                "group": task.group,
                "mode": task.mode,
                "required_tools": task.requires_tools,
                "missing_tools": missing_tools,
                "missing_tools_policy": task.missing_tools_policy,
                "status": if missing_tools.is_empty() { "ready" } else if task.missing_tools_policy == "skip" { "skip" } else { "fail" }
            })
        })
        .collect::<Vec<_>>();
    let missing_tool_tasks = rows
        .iter()
        .filter(|row| {
            row["missing_tools"]
                .as_array()
                .is_some_and(|value| !value.is_empty())
        })
        .count();
    serde_json::json!({
        "report_id": "suite-preflight",
        "version": 1,
        "inputs": {
            "suite": suite_id,
            "run_id": run_id
        },
        "suite": suite_id,
        "run_id": run_id,
        "status": if missing_tool_tasks == 0 { "ready" } else { "degraded" },
        "summary": {
            "task_count": tasks.len(),
            "missing_tool_tasks": missing_tool_tasks
        },
        "rows": rows
    })
}

fn skipped_or_failed_output(task: &SuiteTask, missing_tools: Vec<String>) -> TaskOutput {
    TaskOutput {
        task: task.clone(),
        status: if task.missing_tools_policy == "skip" {
            "skip".to_string()
        } else {
            "fail".to_string()
        },
        duration_ms: 0,
        report_paths: Vec::new(),
        error_summary: Some(format!(
            "missing required tools: {}",
            missing_tools.join(", ")
        )),
        stdout_path: String::new(),
        stderr_path: String::new(),
    }
}

fn task_status_from_exit(code: i32) -> &'static str {
    if code == 0 {
        "pass"
    } else {
        "fail"
    }
}

fn run_task(repo_root: &Path, task_root: &Path, task: &SuiteTask) -> Result<TaskOutput, String> {
    fs::create_dir_all(task_root)
        .map_err(|err| format!("create {} failed: {err}", task_root.display()))?;
    let temp_root = task_root.join("tmp");
    fs::create_dir_all(&temp_root)
        .map_err(|err| format!("create {} failed: {err}", temp_root.display()))?;
    let stdout_path = task_root.join("stdout.log");
    let stderr_path = task_root.join("stderr.log");
    let source_root = source_artifact_root(task_root);
    fs::create_dir_all(&source_root)
        .map_err(|err| format!("create {} failed: {err}", source_root.display()))?;
    let start = Instant::now();
    let mut combined_stdout = String::new();
    let mut combined_stderr = String::new();
    let mut status = "pass".to_string();
    let mut error_summary = None;

    for attempt in 0..=task.retries {
        combined_stdout.clear();
        combined_stderr.clear();
        status = "pass".to_string();
        error_summary = None;
        let mut failed = false;
        for command in &task.commands {
            let (program, args) = parse_command_invocation(command)?;
            let output = Command::new(&program)
                .args(&args)
                .current_dir(repo_root)
                .env("TMPDIR", &temp_root)
                .env("TMP", &temp_root)
                .env("TEMP", &temp_root)
                .env("ARTIFACT_ROOT", &source_root)
                .env("RUN_ID", "suite-run")
                .env("ATLAS_SUITE_TASK_ARTIFACT_ROOT", task_root)
                .output()
                .map_err(|err| format!("run `{command}` failed: {err}"))?;
            if !combined_stdout.is_empty() {
                combined_stdout.push('\n');
            }
            if !combined_stderr.is_empty() {
                combined_stderr.push('\n');
            }
            combined_stdout.push_str(&String::from_utf8_lossy(&output.stdout));
            combined_stderr.push_str(&String::from_utf8_lossy(&output.stderr));
            let code = output.status.code().unwrap_or(1);
            if code != 0 {
                status = task_status_from_exit(code).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr);
                let summary = stderr
                    .lines()
                    .find(|line| !line.trim().is_empty())
                    .unwrap_or("command failed");
                error_summary = Some(format!("{command}: {summary}"));
                failed = true;
                break;
            }
        }
        if !failed || attempt == task.retries {
            break;
        }
    }

    fs::write(&stdout_path, &combined_stdout)
        .map_err(|err| format!("write {} failed: {err}", stdout_path.display()))?;
    fs::write(&stderr_path, &combined_stderr)
        .map_err(|err| format!("write {} failed: {err}", stderr_path.display()))?;

    let mut report_paths = task_report_paths(task, task_root);
    if task.kind == "check" && !task.report_ids.is_empty() {
        match write_governed_check_reports(
            repo_root,
            task_root,
            task,
            &combined_stdout,
            &combined_stderr,
            &status,
        ) {
            Ok(paths) => {
                report_paths = paths;
            }
            Err(err) => {
                status = "fail".to_string();
                error_summary = Some(format!("governed report generation failed: {err}"));
            }
        }
    }

    Ok(TaskOutput {
        task: task.clone(),
        status,
        duration_ms: start.elapsed().as_millis() as u64,
        report_paths,
        error_summary,
        stdout_path: stdout_path.display().to_string(),
        stderr_path: stderr_path.display().to_string(),
    })
}

fn result_report(task_output: &TaskOutput) -> serde_json::Value {
    serde_json::json!({
        "report_id": "suite-result",
        "version": 1,
        "inputs": {
            "task_id": task_output.task.id,
            "commands": task_output.task.commands,
        },
        "id": task_output.task.id,
        "status": task_output.status,
        "duration_ms": task_output.duration_ms,
        "mode": task_output.task.mode,
        "group": task_output.task.group,
        "severity": task_output.task.severity,
        "cpu_hint": task_output.task.cpu_hint,
        "mem_hint": task_output.task.mem_hint,
        "summary": task_output.task.summary,
        "report_paths": task_output.report_paths,
        "error_summary": task_output.error_summary,
        "artifacts": {
            "stdout": task_output.stdout_path,
            "stderr": task_output.stderr_path,
        }
    })
}

fn summary_report(
    suite_id: &str,
    run_id: &str,
    tasks: &[TaskOutput],
    artifacts_root: &Path,
    elapsed_ms: u64,
    strict: bool,
    scheduling_plan: &[SchedulingLane],
) -> serde_json::Value {
    let failures = tasks
        .iter()
        .filter(|task| task.status == "fail" && (strict || task.task.severity != "info"))
        .map(|task| {
            serde_json::json!({
                "id": task.task.id,
                "group": task.task.group,
                "error_summary": task.error_summary,
            })
        })
        .collect::<Vec<_>>();
    let warnings = tasks
        .iter()
        .filter(|task| task.status == "fail" && !strict && task.task.severity == "info")
        .map(|task| {
            serde_json::json!({
                "id": task.task.id,
                "group": task.task.group,
                "error_summary": task.error_summary,
            })
        })
        .collect::<Vec<_>>();
    let pass = tasks.iter().filter(|task| task.status == "pass").count();
    let fail = tasks
        .iter()
        .filter(|task| task.status == "fail" && (strict || task.task.severity != "info"))
        .count();
    let warn = tasks
        .iter()
        .filter(|task| task.status == "fail" && !strict && task.task.severity == "info")
        .count();
    let skip = tasks.iter().filter(|task| task.status == "skip").count();
    serde_json::json!({
        "report_id": "suite-summary",
        "version": 1,
        "inputs": {
            "suite": suite_id,
            "run_id": run_id,
        },
        "suite": suite_id,
        "run_id": run_id,
        "run_timestamp": run_timestamp(),
        "artifacts_root": artifacts_root.display().to_string(),
        "elapsed_ms": elapsed_ms,
        "status": if fail == 0 { "pass" } else { "fail" },
        "summary": {
            "pass": pass,
            "fail": fail,
            "warn": warn,
            "skip": skip,
            "total": tasks.len(),
            "strict": strict,
        },
        "failures": failures,
        "warnings": warnings,
        "scheduling_plan": scheduling_plan.iter().map(|lane| serde_json::json!({
            "batch": lane.batch,
            "id": lane.id,
            "group": lane.group,
            "mode": lane.mode,
            "severity": lane.severity,
            "cpu_hint": lane.cpu_hint,
            "mem_hint": lane.mem_hint,
        })).collect::<Vec<_>>(),
        "tasks": tasks.iter().map(|task| serde_json::json!({
            "id": task.task.id,
            "status": task.status,
            "group": task.task.group,
            "mode": task.task.mode,
            "severity": task.task.severity,
            "cpu_hint": task.task.cpu_hint,
            "mem_hint": task.task.mem_hint,
            "duration_ms": task.duration_ms,
            "summary": task.task.summary,
            "error_summary": task.error_summary,
            "result_path": artifacts_root.join(&task.task.id).join("result.json").display().to_string(),
            "stdout_path": task.stdout_path,
            "stderr_path": task.stderr_path,
        })).collect::<Vec<_>>()
    })
}

fn write_task_result(
    repo_root: &Path,
    task_root: &Path,
    task_output: &TaskOutput,
) -> Result<(), String> {
    let report = result_report(task_output);
    validate_named_report(repo_root, suite_result_schema_name(), &report)?;
    let rendered = serde_json::to_string_pretty(&report)
        .map_err(|err| format!("encode suite result failed: {err}"))?;
    fs::write(task_root.join("result.json"), format!("{rendered}\n"))
        .map_err(|err| format!("write result.json failed: {err}"))
}

fn run_tasks_parallel(
    repo_root: &Path,
    suite_root: &Path,
    tasks: Vec<SuiteTask>,
    jobs: usize,
    fail_fast: bool,
) -> Result<Vec<TaskOutput>, String> {
    let queue = Arc::new(Mutex::new(VecDeque::from(tasks)));
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel::<Result<TaskOutput, String>>();
    let worker_count = jobs.max(1);

    thread::scope(|scope| {
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let stop = Arc::clone(&stop);
            let sender = sender.clone();
            scope.spawn(move || loop {
                if stop.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                let next = {
                    let mut guard = queue
                        .lock()
                        .unwrap_or_else(|err| panic!("suite queue mutex poisoned: {err}"));
                    guard.pop_front()
                };
                let Some(task) = next else {
                    return;
                };
                let task_root = suite_root.join(&task.id);
                let result = run_task(repo_root, &task_root, &task).and_then(|task_output| {
                    write_task_result(repo_root, &task_root, &task_output)?;
                    Ok(task_output)
                });
                let should_stop =
                    matches!(&result, Ok(output) if output.status == "fail") && fail_fast;
                let _ = sender.send(result);
                if should_stop {
                    stop.store(true, std::sync::atomic::Ordering::Relaxed);
                    return;
                }
            });
        }
    });
    drop(sender);

    let mut outputs = Vec::new();
    let mut errors = Vec::new();
    for item in receiver {
        match item {
            Ok(output) => outputs.push(output),
            Err(err) => errors.push(err),
        }
    }
    outputs.sort_by(|a, b| a.task.id.cmp(&b.task.id));
    if errors.is_empty() {
        Ok(outputs)
    } else {
        Err(errors.join("\n"))
    }
}

fn execute_plan_batches(
    repo_root: &Path,
    suite_root: &Path,
    batches: Vec<Vec<SuiteTask>>,
    jobs: usize,
    fail_fast: bool,
) -> Result<Vec<TaskOutput>, String> {
    let mut outputs = Vec::new();
    for batch in batches {
        let mut batch_outputs = run_tasks_parallel(repo_root, suite_root, batch, jobs, fail_fast)?;
        let should_stop = fail_fast && batch_outputs.iter().any(|task| task.status == "fail");
        outputs.append(&mut batch_outputs);
        if should_stop {
            break;
        }
    }
    outputs.sort_by(|a, b| a.task.id.cmp(&b.task.id));
    Ok(outputs)
}

#[allow(clippy::too_many_arguments)]
fn render_suite_output(
    summary: &serde_json::Value,
    task_details: &[serde_json::Value],
    suite_root: &Path,
    format: SuiteOutputFormatArg,
    quiet: bool,
    verbose: bool,
    color: SuiteColorArg,
    slowdown_rows: &[String],
    registry_footer: Option<&str>,
) -> Result<String, String> {
    let human = human_suite_report(
        summary,
        task_details,
        suite_root,
        quiet,
        verbose,
        color,
        slowdown_rows,
        registry_footer,
    );
    let json = serde_json::to_string_pretty(summary)
        .map_err(|err| format!("encode suite summary failed: {err}"))?;
    match format {
        SuiteOutputFormatArg::Human => Ok(human),
        SuiteOutputFormatArg::Json => Ok(json),
        SuiteOutputFormatArg::Both => Ok(format!("{human}\n\n{json}")),
    }
}

fn suite_task_details(
    suite_root: &Path,
    tasks: &[serde_json::Value],
) -> Result<Vec<serde_json::Value>, String> {
    tasks
        .iter()
        .map(|task| {
            let result_path = task["result_path"]
                .as_str()
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    suite_root
                        .join(task["id"].as_str().unwrap_or_default())
                        .join("result.json")
                });
            let result = read_result_value(&result_path)?;
            Ok(serde_json::json!({
                "id": result["id"],
                "status": result["status"],
                "duration_ms": result["duration_ms"],
                "mode": result["mode"],
                "group": result["group"],
                "severity": result["severity"],
                "cpu_hint": result["cpu_hint"],
                "mem_hint": result["mem_hint"],
                "summary": result["summary"],
                "error_summary": result["error_summary"],
                "result_path": result_path.display().to_string(),
                "stdout_path": result["artifacts"]["stdout"],
                "stderr_path": result["artifacts"]["stderr"],
            }))
        })
        .collect()
}

fn render_suite_last(
    artifacts_root: &Path,
    suite_id: &str,
    format: FormatArg,
) -> Result<String, String> {
    let pointer = load_latest_runs_pointer(artifacts_root)?;
    let entry = pointer
        .suites
        .get(suite_id)
        .ok_or_else(|| format!("no latest run recorded for suite `{suite_id}`"))?;
    match format {
        FormatArg::Text => Ok(entry.suite_root.clone()),
        FormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "suite": suite_id,
            "run_id": entry.run_id,
            "suite_root": entry.suite_root,
        }))
        .map_err(|err| format!("encode suites last failed: {err}")),
        FormatArg::Jsonl => Err("jsonl output is not supported for suites last".to_string()),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_report_command(
    artifacts_root: &Path,
    suite_id: &str,
    run_id: &str,
    failed_only: bool,
    group: Option<&str>,
    id: Option<&str>,
    format: SuiteOutputFormatArg,
    quiet: bool,
    verbose: bool,
    color: SuiteColorArg,
) -> Result<(String, i32), String> {
    let suite_root = suite_root_path(artifacts_root, suite_id, run_id);
    let summary = read_suite_summary(&suite_root)?;
    let filtered_tasks = filter_summary_tasks(&summary, failed_only, group, id);
    let task_details = suite_task_details(&suite_root, &filtered_tasks)?;
    let rendered = render_suite_output(
        &summary,
        &task_details,
        &suite_root,
        format,
        quiet,
        verbose,
        color,
        &[],
        None,
    )?;
    let exit = if summary["status"].as_str() == Some("pass") {
        0
    } else {
        1
    };
    Ok((rendered, exit))
}

fn run_history_command(
    artifacts_root: &Path,
    suite_id: &str,
    task_id: &str,
    limit: usize,
    format: FormatArg,
) -> Result<(String, i32), String> {
    let history_path = suite_history_path(artifacts_root, suite_id);
    let entries = if history_path.exists() {
        fs::read_to_string(&history_path)
            .map_err(|err| format!("read {} failed: {err}", history_path.display()))?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<serde_json::Value>(line)
                    .map_err(|err| format!("parse {} failed: {err}", history_path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    let filtered = entries
        .into_iter()
        .filter(|entry| entry["task_id"].as_str() == Some(task_id))
        .rev()
        .take(limit.max(1))
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "suite": suite_id,
        "task_id": task_id,
        "history_path": history_path.display().to_string(),
        "entries": filtered.iter().rev().cloned().collect::<Vec<_>>(),
    });
    let rendered = match format {
        FormatArg::Text => {
            let mut lines = vec![format!("History {suite_id} {task_id}")];
            for entry in payload["entries"].as_array().into_iter().flatten() {
                lines.push(format!(
                    "{} {} {} {}",
                    entry["run_id"].as_str().unwrap_or_default(),
                    entry["status"].as_str().unwrap_or_default(),
                    format_duration_ms(entry["duration_ms"].as_u64().unwrap_or(0)),
                    entry["timestamp"].as_str().unwrap_or_default()
                ));
            }
            lines.join("\n")
        }
        FormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode suites history failed: {err}"))?,
        FormatArg::Jsonl => Err("jsonl output is not supported for suites history".to_string())?,
    };
    Ok((rendered, 0))
}

fn suite_diff_report(
    root: &Path,
    suite_id: &str,
    baseline: &serde_json::Value,
    current: &serde_json::Value,
) -> Result<(serde_json::Value, Vec<String>), String> {
    let budget = load_perf_budget(root, suite_id)?;
    let baseline_tasks = baseline["tasks"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|task| (task["id"].as_str().unwrap_or_default().to_string(), task))
        .collect::<BTreeMap<_, _>>();
    let current_tasks = current["tasks"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|task| (task["id"].as_str().unwrap_or_default().to_string(), task))
        .collect::<BTreeMap<_, _>>();

    let new_failures = current_tasks
        .iter()
        .filter_map(|(id, task)| {
            let current_failed = task["status"].as_str() == Some("fail");
            let baseline_failed = baseline_tasks
                .get(id)
                .is_some_and(|baseline_task| baseline_task["status"].as_str() == Some("fail"));
            if current_failed && !baseline_failed {
                Some(serde_json::json!({
                    "id": id,
                    "group": task["group"],
                    "current_status": task["status"],
                    "baseline_status": baseline_tasks.get(id).map(|v| v["status"].clone()).unwrap_or(serde_json::Value::Null),
                }))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let fixed_failures = baseline_tasks
        .iter()
        .filter_map(|(id, task)| {
            let baseline_failed = task["status"].as_str() == Some("fail");
            let current_failed = current_tasks
                .get(id)
                .is_some_and(|current_task| current_task["status"].as_str() == Some("fail"));
            if baseline_failed && !current_failed {
                Some(serde_json::json!({
                    "id": id,
                    "group": task["group"],
                    "baseline_status": task["status"],
                    "current_status": current_tasks.get(id).map(|v| v["status"].clone()).unwrap_or(serde_json::Value::Null),
                }))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let new_tasks = current_tasks
        .iter()
        .filter(|(id, _)| !baseline_tasks.contains_key(*id))
        .map(|(id, task)| {
            serde_json::json!({
                "id": id,
                "group": task["group"],
                "status": task["status"],
            })
        })
        .collect::<Vec<_>>();

    let duration_regressions = current_tasks
        .iter()
        .filter_map(|(id, task)| {
            let current_duration = task["duration_ms"].as_u64().unwrap_or(0);
            let baseline_task = baseline_tasks.get(id)?;
            let baseline_duration = baseline_task["duration_ms"].as_u64().unwrap_or(0);
            if current_duration <= baseline_duration {
                return None;
            }
            let delta_ms = current_duration - baseline_duration;
            let delta_ratio = if baseline_duration == 0 {
                1.0
            } else {
                delta_ms as f64 / baseline_duration as f64
            };
            if delta_ms >= budget.duration_regression_ms
                || delta_ratio >= budget.duration_regression_ratio
            {
                Some(serde_json::json!({
                    "id": id,
                    "group": task["group"],
                    "baseline_duration_ms": baseline_duration,
                    "current_duration_ms": current_duration,
                    "delta_ms": delta_ms,
                    "delta_ratio": delta_ratio,
                }))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let slowdown_rows = duration_regressions
        .iter()
        .map(|row| {
            format!(
                "{} {} -> {} (+{}, +{:.0}%)",
                row["id"].as_str().unwrap_or_default(),
                format_duration_ms(row["baseline_duration_ms"].as_u64().unwrap_or(0)),
                format_duration_ms(row["current_duration_ms"].as_u64().unwrap_or(0)),
                format_duration_ms(row["delta_ms"].as_u64().unwrap_or(0)),
                row["delta_ratio"].as_f64().unwrap_or(0.0) * 100.0
            )
        })
        .collect::<Vec<_>>();

    let report = serde_json::json!({
        "report_id": "suite-diff",
        "version": 1,
        "inputs": {
            "suite": suite_id,
            "baseline_run_id": baseline["run_id"],
            "current_run_id": current["run_id"],
        },
        "suite": suite_id,
        "baseline_run_id": baseline["run_id"],
        "current_run_id": current["run_id"],
        "thresholds": {
            "duration_regression_ms": budget.duration_regression_ms,
            "duration_regression_ratio": budget.duration_regression_ratio,
        },
        "summary": {
            "new_failures": new_failures.len(),
            "fixed_failures": fixed_failures.len(),
            "duration_regressions": duration_regressions.len(),
            "new_tasks": new_tasks.len(),
        },
        "new_failures": new_failures,
        "fixed_failures": fixed_failures,
        "duration_regressions": duration_regressions,
        "new_tasks": new_tasks,
    });
    Ok((report, slowdown_rows))
}

fn human_suite_diff_report(
    report: &serde_json::Value,
    suite_root: &Path,
    quiet: bool,
    slowdown_rows: &[String],
) -> String {
    let mut lines = vec![format!(
        "Suite diff {}  {} -> {}",
        report["suite"].as_str().unwrap_or_default(),
        report["baseline_run_id"].as_str().unwrap_or_default(),
        report["current_run_id"].as_str().unwrap_or_default()
    )];
    let sections = [
        ("New failures", "new_failures"),
        ("Fixed failures", "fixed_failures"),
        ("New tasks", "new_tasks"),
    ];
    for (label, key) in sections {
        let rows = report[key].as_array().cloned().unwrap_or_default();
        if quiet && rows.is_empty() {
            continue;
        }
        lines.push(label.to_string());
        if rows.is_empty() {
            lines.push("  none".to_string());
        } else {
            for row in rows {
                lines.push(format!(
                    "  {} [{}]",
                    row["id"].as_str().unwrap_or_default(),
                    row["group"].as_str().unwrap_or_default()
                ));
            }
        }
    }
    if !quiet || !slowdown_rows.is_empty() {
        lines.push("Slowdowns".to_string());
        if slowdown_rows.is_empty() {
            lines.push("  none".to_string());
        } else {
            lines.extend(slowdown_rows.iter().map(|row| format!("  {row}")));
        }
    }
    lines.push(format!(
        "Totals new_failures={} fixed_failures={} duration_regressions={} new_tasks={} artifacts={}",
        report["summary"]["new_failures"].as_u64().unwrap_or(0),
        report["summary"]["fixed_failures"].as_u64().unwrap_or(0),
        report["summary"]["duration_regressions"].as_u64().unwrap_or(0),
        report["summary"]["new_tasks"].as_u64().unwrap_or(0),
        suite_root.display(),
    ));
    lines.join("\n")
}

fn run_diff_command(
    root: &Path,
    artifacts_root: &Path,
    suite_id: &str,
    baseline_run_id: &str,
    current_run_id: &str,
    format: SuiteOutputFormatArg,
    quiet: bool,
) -> Result<(String, i32), String> {
    let current_root = suite_root_path(artifacts_root, suite_id, current_run_id);
    let baseline_root = suite_root_path(artifacts_root, suite_id, baseline_run_id);
    let baseline = read_suite_summary(&baseline_root)?;
    let current = read_suite_summary(&current_root)?;
    let (report, slowdown_rows) = suite_diff_report(root, suite_id, &baseline, &current)?;
    validate_named_report(root, suite_diff_schema_name(), &report)?;
    let rendered = match format {
        SuiteOutputFormatArg::Human => {
            human_suite_diff_report(&report, &current_root, quiet, &slowdown_rows)
        }
        SuiteOutputFormatArg::Json => serde_json::to_string_pretty(&report)
            .map_err(|err| format!("encode suite diff failed: {err}"))?,
        SuiteOutputFormatArg::Both => format!(
            "{}\n\n{}",
            human_suite_diff_report(&report, &current_root, quiet, &slowdown_rows),
            serde_json::to_string_pretty(&report)
                .map_err(|err| format!("encode suite diff failed: {err}"))?
        ),
    };
    let exit = if report["summary"]["new_failures"].as_u64().unwrap_or(0) > 0 {
        1
    } else {
        0
    };
    Ok((rendered, exit))
}

fn execute_suite_run(options: SuiteRunOptions) -> Result<(String, i32), String> {
    let started = Instant::now();
    let suite_id = normalize_suite_id(&options.suite);
    if options.fail_fast == Some(true) && options.fail_fast == Some(false) {
        return Err("invalid fail-fast state".to_string());
    }
    let jobs = parse_jobs(&options.repo_root, &suite_id, &options.jobs)?;
    let tasks = select_tasks(
        &options.repo_root,
        &suite_id,
        options.mode,
        options.group.as_deref(),
        options.tag.as_deref(),
    )?;
    if tasks.is_empty() {
        return Err(format!("suite `{suite_id}` selected no entries"));
    }
    let run_id = match options.run_id_override {
        Some(value) => value,
        None => deterministic_run_id(
            &options.repo_root,
            &suite_id,
            options.mode,
            options.group.as_deref(),
            options.tag.as_deref(),
        )?,
    };
    let suite_root = options
        .artifacts_root
        .join("suites")
        .join(&suite_id)
        .join(&run_id);
    fs::create_dir_all(&suite_root)
        .map_err(|err| format!("create {} failed: {err}", suite_root.display()))?;
    let preflight = suite_preflight_report(&suite_id, &run_id, &tasks);
    validate_named_report(
        &options.repo_root,
        suite_preflight_schema_name(),
        &preflight,
    )?;
    let preflight_path = suite_root.join("suite-preflight.json");
    let preflight_text = serde_json::to_string_pretty(&preflight)
        .map_err(|err| format!("encode suite preflight failed: {err}"))?;
    fs::write(&preflight_path, format!("{preflight_text}\n"))
        .map_err(|err| format!("write {} failed: {err}", preflight_path.display()))?;

    let (runnable, gated) = tasks
        .into_iter()
        .partition::<Vec<_>, _>(|task| task.requires_tools.iter().all(|tool| tool_in_path(tool)));
    let mut task_outputs = gated
        .into_iter()
        .map(|task| {
            let missing_tools = task
                .requires_tools
                .iter()
                .filter(|tool| !tool_in_path(tool))
                .cloned()
                .collect::<Vec<_>>();
            let output = skipped_or_failed_output(&task, missing_tools);
            let task_root = suite_root.join(&task.id);
            fs::create_dir_all(&task_root)
                .map_err(|err| format!("create {} failed: {err}", task_root.display()))?;
            write_task_result(&options.repo_root, &task_root, &output)?;
            Ok(output)
        })
        .collect::<Result<Vec<_>, String>>()?;

    let (plan_batches, scheduling_plan) =
        build_execution_plan(&options.repo_root, &suite_id, runnable, jobs)?;
    let mut executed = execute_plan_batches(
        &options.repo_root,
        &suite_root,
        plan_batches,
        jobs,
        options.fail_fast.unwrap_or(false),
    )?;
    task_outputs.append(&mut executed);
    task_outputs.sort_by(|a, b| a.task.id.cmp(&b.task.id));
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let summary = summary_report(
        &suite_id,
        &run_id,
        &task_outputs,
        &suite_root,
        elapsed_ms,
        options.strict,
        &scheduling_plan,
    );
    validate_named_report(&options.repo_root, suite_summary_schema_name(), &summary)?;
    let summary_path = suite_root.join("suite-summary.json");
    let summary_text = serde_json::to_string_pretty(&summary)
        .map_err(|err| format!("encode suite summary failed: {err}"))?;
    fs::write(&summary_path, format!("{summary_text}\n"))
        .map_err(|err| format!("write {} failed: {err}", summary_path.display()))?;
    write_latest_run_pointer(&options.artifacts_root, &suite_id, &run_id, &suite_root)?;
    append_suite_history_entries(
        &options.repo_root,
        &options.artifacts_root,
        &suite_id,
        &run_id,
        &task_outputs,
    )?;
    let detail_rows = task_outputs.iter().map(result_report).collect::<Vec<_>>();
    let rendered = render_suite_output(
        &summary,
        &detail_rows,
        &suite_root,
        options.format,
        options.quiet,
        options.verbose,
        options.color,
        &[],
        Some(&registry_completeness_footer(&options.repo_root)?),
    )?;
    write_output_if_requested(options.out, &rendered)?;
    let exit = if summary["status"].as_str() == Some("pass") {
        0
    } else {
        1
    };
    Ok((rendered, exit))
}

pub(crate) fn run_registry_check_by_id(
    repo_root: Option<PathBuf>,
    artifacts_root: Option<PathBuf>,
    run_id: Option<String>,
    check_id: String,
    fail_fast: bool,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    run_registered_runnable(
        &root,
        artifacts_root,
        run_id.map(|value| format!("check-{check_id}-{value}")),
        check_id,
        RunnableKind::Check,
        fail_fast,
        format,
        out,
    )
}

pub(crate) fn run_registry_contract_by_id(
    repo_root: Option<PathBuf>,
    artifacts_root: Option<PathBuf>,
    run_id: Option<String>,
    contract_id: String,
    _fail_fast: bool,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    run_registered_runnable(
        &root,
        artifacts_root,
        run_id.or_else(|| Some(format!("contract-{}", contract_id.to_ascii_lowercase()))),
        contract_id,
        RunnableKind::Contract,
        false,
        format,
        out,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_registered_runnable(
    repo_root: &Path,
    artifacts_root: Option<PathBuf>,
    run_id: Option<String>,
    runnable_id: String,
    kind: RunnableKind,
    _fail_fast: bool,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    if matches!(format, FormatArg::Jsonl) {
        return Err("jsonl output is not supported for runnable execution".to_string());
    }

    let registry = RunnableRegistry::load(repo_root)?;
    let selection = RunnableSelection {
        suite: None,
        group: None,
        tag: None,
        id: Some(RunnableId::parse(&runnable_id)?),
    };
    let mut selected = registry.select(&selection);
    let Some(mut entry) = selected.pop() else {
        return Err(format!("unknown runnable id `{runnable_id}`"));
    };
    if entry.kind != kind {
        return Err(format!("runnable `{runnable_id}` is not a {:?}", kind).to_lowercase());
    }
    if entry.kind == RunnableKind::Contract {
        entry.commands = normalized_contract_runner_from_entry(&entry);
    }

    let effective_run_id = RunId::from_seed(
        &run_id.unwrap_or_else(|| format!("{}-{}", entry.suite.as_str(), entry.id.as_str())),
    );
    let artifact_root = artifacts_root.unwrap_or_else(|| repo_root.join("artifacts").join("run"));
    let context = RunnableRunContext {
        repo_root: repo_root.to_path_buf(),
        run_id: effective_run_id.clone(),
        artifact_store: ArtifactStore::new(artifact_root),
        effect_policy: EffectPolicy {
            capabilities: Capabilities {
                fs_write: true,
                subprocess: true,
                git: true,
                network: true,
            },
        },
    };
    let process = RealProcessRunner;
    let fs = RealFs;
    let executor = CommandRunnableExecutor {
        process: &process,
        fs: &fs,
    };
    let result = executor.execute(&entry, &context)?;
    let status = match result.status {
        RunStatus::Pass => "pass",
        RunStatus::Fail => "fail",
        RunStatus::Skip => "skip",
        RunStatus::Error => "error",
    };
    let payload = serde_json::json!({
        "suite": entry.suite.as_str(),
        "run_id": effective_run_id.as_str(),
        "runnable_id": entry.id.as_str(),
        "kind": match entry.kind {
            RunnableKind::Check => "check",
            RunnableKind::Contract => "contract",
        },
        "status": status,
        "duration_ms": result.duration_ms,
        "summary": entry.summary,
        "group": entry.group,
        "reports": result.report_refs,
        "failure_summary": result.failure_summary,
        "skipped": result.skipped.as_ref().map(|item| serde_json::json!({
            "reason": item.reason,
            "required_tool": item.required_tool,
        })),
    });
    let rendered = match format {
        FormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode runnable result failed: {err}"))?,
        FormatArg::Text => {
            let mut lines = vec![
                format!(
                    "{} {}",
                    payload["kind"].as_str().unwrap_or("runnable"),
                    entry.id
                ),
                format!("status: {}", status),
                format!("run_id: {}", effective_run_id.as_str()),
            ];
            if let Some(summary) = result.failure_summary.as_ref() {
                lines.push(format!("failure: {summary}"));
            }
            if let Some(skipped) = result.skipped.as_ref() {
                lines.push(format!("skip: {}", skipped.reason));
            }
            lines.push(format!(
                "artifacts: {}",
                context.artifact_store.root().display()
            ));
            lines.join("\n")
        }
        FormatArg::Jsonl => unreachable!(),
    };
    write_output_if_requested(out, &rendered)?;
    let exit = if result.status == RunStatus::Pass {
        0
    } else {
        1
    };
    Ok((rendered, exit))
}

fn normalized_contract_runner_from_entry(
    entry: &bijux_dev_atlas::model::RunnableEntry,
) -> Vec<String> {
    let mode = match entry.mode {
        bijux_dev_atlas::model::RunnableMode::Pure => "pure",
        bijux_dev_atlas::model::RunnableMode::Effect => "effect",
    };
    entry
        .commands
        .iter()
        .map(|command| normalized_contract_command(command, mode, entry.id.as_str()))
        .collect()
}

pub(crate) fn run_suites_command(quiet: bool, command: SuitesCommand) -> i32 {
    let result = (|| -> Result<(String, i32), String> {
        match command {
            SuitesCommand::List {
                repo_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let rendered = render_suite_list(&root, format)?;
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, 0))
            }
            SuitesCommand::Describe {
                suite,
                repo_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let rendered = render_suite_describe(&root, &normalize_suite_id(&suite), format)?;
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, 0))
            }
            SuitesCommand::Last {
                suite,
                repo_root,
                artifacts_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let rendered = render_suite_last(
                    &artifacts_root.unwrap_or_else(|| root.join("artifacts")),
                    &normalize_suite_id(&suite),
                    format,
                )?;
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, 0))
            }
            SuitesCommand::Report {
                suite,
                run,
                repo_root,
                artifacts_root,
                failed_only,
                group,
                id,
                format,
                quiet,
                verbose,
                color,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let (rendered, code) = run_report_command(
                    &artifacts_root.unwrap_or_else(|| root.join("artifacts")),
                    &normalize_suite_id(&suite),
                    &run,
                    failed_only,
                    group.as_deref(),
                    id.as_deref(),
                    format,
                    quiet,
                    verbose,
                    color,
                )?;
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, code))
            }
            SuitesCommand::History {
                suite,
                id,
                repo_root,
                artifacts_root,
                limit,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let (rendered, code) = run_history_command(
                    &artifacts_root.unwrap_or_else(|| root.join("artifacts")),
                    &normalize_suite_id(&suite),
                    &id,
                    limit,
                    format,
                )?;
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, code))
            }
            SuitesCommand::Run {
                suite,
                repo_root,
                artifacts_root,
                run_id,
                jobs,
                fail_fast,
                no_fail_fast,
                mode,
                group,
                tag,
                format,
                quiet,
                verbose,
                color,
                strict,
                out,
            } => {
                if fail_fast && no_fail_fast {
                    return Err("cannot set both --fail-fast and --no-fail-fast".to_string());
                }
                let root = resolve_repo_root(repo_root)?;
                execute_suite_run(SuiteRunOptions {
                    suite,
                    repo_root: root.clone(),
                    artifacts_root: artifacts_root.unwrap_or_else(|| root.join("artifacts")),
                    run_id_override: run_id,
                    jobs,
                    fail_fast: if no_fail_fast {
                        Some(false)
                    } else {
                        Some(fail_fast)
                    },
                    mode,
                    group,
                    tag,
                    format,
                    quiet,
                    verbose,
                    color,
                    strict,
                    out,
                })
            }
            SuitesCommand::Diff {
                suite,
                a,
                b,
                repo_root,
                artifacts_root,
                format,
                quiet,
                verbose: _,
                color: _,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let (rendered, code) = run_diff_command(
                    &root,
                    &artifacts_root.unwrap_or_else(|| root.join("artifacts")),
                    &normalize_suite_id(&suite),
                    &a,
                    &b,
                    format,
                    quiet,
                )?;
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, code))
            }
            SuitesCommand::Lint {
                repo_root,
                format,
                out,
            } => {
                let root = resolve_repo_root(repo_root)?;
                let (rendered, code) = render_suite_lint(&root, format)?;
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, code))
            }
        }
    })();

    match result {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let stream = if code == 0 {
                    &mut std::io::stdout() as &mut dyn Write
                } else {
                    &mut std::io::stderr() as &mut dyn Write
                };
                let _ = writeln!(stream, "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(std::io::stderr(), "bijux-dev-atlas suites failed: {err}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_json(path: &Path, value: &serde_json::Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent");
        }
        fs::write(path, serde_json::to_string_pretty(value).expect("json")).expect("write");
    }

    fn fixture_root() -> tempfile::TempDir {
        let dir = tempdir().expect("tempdir");
        write_json(
            &dir.path()
                .join("configs/governance/suites/suites.index.json"),
            &serde_json::json!({
                "schema_version": 1,
                "index_id": "governance-suites",
                "suites": ["checks", "contracts", "tests"]
            }),
        );
        write_json(
            &dir.path()
                .join("configs/governance/suites/checks.suite.json"),
            &serde_json::json!({
                "schema_version": 1,
                "suite_id": "checks",
                "purpose": "checks",
                "owner": "team:atlas-governance",
                "stability": "stable",
                "entries": [
                    {"id":"CHECK-GIT-VERSION-001","kind":"check","mode":"pure","owner":"team:atlas-governance","tags":["rust"]}
                ]
            }),
        );
        write_json(
            &dir.path()
                .join("configs/governance/suites/contracts.suite.json"),
            &serde_json::json!({
                "schema_version": 1,
                "suite_id": "contracts",
                "purpose": "contracts",
                "owner": "team:atlas-governance",
                "stability": "stable",
                "entries": [
                    {"id":"CONTRACT-GIT-VERSION-001","kind":"contract","mode":"pure","owner":"team:atlas-governance","tags":["ops"]}
                ]
            }),
        );
        write_json(
            &dir.path().join("configs/governance/check-groups.json"),
            &serde_json::json!({
                "groups": [{"id":"rust"}]
            }),
        );
        write_json(
            &dir.path().join("configs/governance/contract-groups.json"),
            &serde_json::json!({
                "groups": [{"id":"ops"}]
            }),
        );
        write_json(
            &dir.path().join("make/target-list.json"),
            &serde_json::json!({
                "public_targets": ["fmt"]
            }),
        );
        write_json(
            &dir.path().join("configs/governance/checks.registry.json"),
            &serde_json::json!({
                "checks": [{
                    "check_id":"CHECK-GIT-VERSION-001",
                    "summary":"git version",
                    "owner":"team:atlas-governance",
                    "mode":"pure",
                    "group":"rust",
                    "commands":["git --version"],
                    "report_ids":["check-suite-summary"],
                    "reports":["check-git-version.json"],
                    "tags":["rust"],
                    "suite_membership":["checks"],
                    "severity":"blocker",
                    "stage":"local",
                    "runtime_cost":"low",
                    "determinism":"strict",
                    "overlaps_with":[],
                    "cpu_hint":"light",
                    "mem_hint":"low"
                }]
            }),
        );
        write_json(
            &dir.path()
                .join("configs/governance/contracts.registry.json"),
            &serde_json::json!({
                "contracts": [{
                    "contract_id":"CONTRACT-GIT-VERSION-001",
                    "summary":"git version",
                    "owner":"team:atlas-governance",
                    "mode":"pure",
                    "group":"ops",
                    "runner":"git --version",
                    "reports":["contract-git-version.json"],
                    "suite_membership":["contracts"],
                    "tags":["ops"],
                    "overlaps_with":[]
                }]
            }),
        );
        write_json(
            &dir.path()
                .join("configs/governance/suites/default-jobs.json"),
            &serde_json::json!({
                "schema_version": 1,
                "policy_id": "suite-default-jobs",
                "suites": [
                    {
                        "suite_id": "checks",
                        "auto_jobs": 2,
                        "low_core_cap": 1,
                        "high_mem_parallelism": 1,
                        "effect_parallelism": 1
                    },
                    {
                        "suite_id": "contracts",
                        "auto_jobs": 2,
                        "low_core_cap": 1,
                        "high_mem_parallelism": 1,
                        "effect_parallelism": 1
                    }
                ]
            }),
        );
        write_json(
            &dir.path()
                .join("configs/schemas/contracts/report-checks/schema-index.json"),
            &serde_json::json!({
                "schema_version": 1,
                "index_id": "checks-schema-index",
                "schemas": [{
                    "report_id":"check-suite-summary",
                    "schema_path":"configs/schemas/contracts/report-checks/check-suite-summary.schema.json"
                }]
            }),
        );
        write_json(
            &dir.path()
                .join("configs/schemas/contracts/report-checks/check-suite-summary.schema.json"),
            &serde_json::json!({
                "required":["report_id","version","inputs"],
                "properties":{
                    "report_id":{"const":"check-suite-summary"},
                    "version":{"const":1},
                    "inputs":{"type":"object"}
                }
            }),
        );
        write_json(
            &dir.path()
                .join("configs/schemas/contracts/reports/suite-result.schema.json"),
            &serde_json::json!({
                "required":["report_id","version","inputs","id","status","duration_ms","mode","group","summary","report_paths","artifacts"],
                "properties":{
                    "report_id":{"const":"suite-result"},
                    "version":{"const":1},
                    "inputs":{"type":"object"},
                    "id":{"type":"string"},
                    "status":{"type":"string"},
                    "duration_ms":{"type":"integer"},
                    "mode":{"type":"string"},
                    "group":{"type":"string"},
                    "severity":{"type":"string"},
                    "cpu_hint":{"type":"string"},
                    "mem_hint":{"type":"string"},
                    "summary":{"type":"string"},
                    "report_paths":{"type":"array"},
                    "artifacts":{"type":"object"}
                }
            }),
        );
        write_json(
            &dir.path()
                .join("configs/schemas/contracts/reports/suite-summary.schema.json"),
            &serde_json::json!({
                "required":["report_id","version","inputs","suite","run_id","run_timestamp","artifacts_root","elapsed_ms","status","summary","failures","warnings","scheduling_plan","tasks"],
                "properties":{
                    "report_id":{"const":"suite-summary"},
                    "version":{"const":1},
                    "inputs":{"type":"object"},
                    "suite":{"type":"string"},
                    "run_id":{"type":"string"},
                    "run_timestamp":{"type":"string"},
                    "artifacts_root":{"type":"string"},
                    "elapsed_ms":{"type":"integer"},
                    "status":{"type":"string"},
                    "summary":{"type":"object"},
                    "failures":{"type":"array"},
                    "warnings":{"type":"array"},
                    "scheduling_plan":{"type":"array"},
                    "tasks":{"type":"array"}
                }
            }),
        );
        write_json(
            &dir.path()
                .join("configs/schemas/contracts/reports/suite-preflight.schema.json"),
            &serde_json::json!({
                "required":["report_id","version","inputs","suite","run_id","status","summary","rows"],
                "properties":{
                    "report_id":{"const":"suite-preflight"},
                    "version":{"const":1},
                    "inputs":{"type":"object"},
                    "suite":{"type":"string"},
                    "run_id":{"type":"string"},
                    "status":{"type":"string"},
                    "summary":{"type":"object"},
                    "rows":{"type":"array"}
                }
            }),
        );
        write_json(
            &dir.path()
                .join("configs/schemas/contracts/reports/suite-diff.schema.json"),
            &serde_json::json!({
                "required":["report_id","version","inputs","suite","baseline_run_id","current_run_id","thresholds","summary","new_failures","fixed_failures","duration_regressions","new_tasks"],
                "properties":{
                    "report_id":{"const":"suite-diff"},
                    "version":{"const":1},
                    "inputs":{"type":"object"},
                    "suite":{"type":"string"},
                    "baseline_run_id":{"type":"string"},
                    "current_run_id":{"type":"string"},
                    "thresholds":{"type":"object"},
                    "summary":{"type":"object"},
                    "new_failures":{"type":"array"},
                    "fixed_failures":{"type":"array"},
                    "duration_regressions":{"type":"array"},
                    "new_tasks":{"type":"array"}
                }
            }),
        );
        write_json(
            &dir.path()
                .join("configs/schemas/contracts/reports/suite-history-entry.schema.json"),
            &serde_json::json!({
                "required":["report_id","version","inputs","suite","run_id","task_id","group","mode","status","duration_ms","timestamp","result_path"],
                "properties":{
                    "report_id":{"const":"suite-history-entry"},
                    "version":{"const":1},
                    "inputs":{"type":"object"},
                    "suite":{"type":"string"},
                    "run_id":{"type":"string"},
                    "task_id":{"type":"string"},
                    "group":{"type":"string"},
                    "mode":{"type":"string"},
                    "status":{"type":"string"},
                    "duration_ms":{"type":"integer"},
                    "timestamp":{"type":"string"},
                    "result_path":{"type":"string"}
                }
            }),
        );
        write_json(
            &dir.path().join("configs/governance/perf-budgets.json"),
            &serde_json::json!({
                "schema_version": 1,
                "budgets": [
                    {
                        "suite": "checks",
                        "duration_regression_ms": 1,
                        "duration_regression_ratio": 0.0
                    },
                    {
                        "suite": "contracts",
                        "duration_regression_ms": 1,
                        "duration_regression_ratio": 0.0
                    }
                ]
            }),
        );
        write_json(
            &dir.path()
                .join("configs/schemas/contracts/governance/perf-budgets.schema.json"),
            &serde_json::json!({
                "required":["schema_version","budgets"],
                "properties":{
                    "schema_version":{"const":1},
                    "budgets":{"type":"array"}
                }
            }),
        );
        dir
    }

    #[test]
    fn deterministic_run_id_depends_on_filters() {
        let root = fixture_root();
        let a = deterministic_run_id(root.path(), "checks", SuiteModeArg::All, None, None)
            .expect("run id");
        let b = deterministic_run_id(root.path(), "checks", SuiteModeArg::Pure, None, None)
            .expect("run id");
        assert_ne!(a, b);
    }

    #[test]
    fn suite_run_executes_registry_entries() {
        let root = fixture_root();
        let result = execute_suite_run(SuiteRunOptions {
            suite: "checks".to_string(),
            repo_root: root.path().to_path_buf(),
            artifacts_root: root.path().join("artifacts"),
            run_id_override: Some("checks-test".to_string()),
            jobs: "1".to_string(),
            fail_fast: Some(false),
            mode: SuiteModeArg::All,
            group: None,
            tag: None,
            format: SuiteOutputFormatArg::Json,
            quiet: false,
            verbose: false,
            color: SuiteColorArg::Never,
            strict: false,
            out: None,
        })
        .expect("suite run");
        assert_eq!(result.1, 0);
        let summary_path = root
            .path()
            .join("artifacts/suites/checks/checks-test/suite-summary.json");
        assert!(summary_path.exists());
    }

    #[test]
    fn report_and_last_read_existing_suite_artifacts() {
        let root = fixture_root();
        execute_suite_run(SuiteRunOptions {
            suite: "checks".to_string(),
            repo_root: root.path().to_path_buf(),
            artifacts_root: root.path().join("artifacts"),
            run_id_override: Some("checks-report".to_string()),
            jobs: "1".to_string(),
            fail_fast: Some(false),
            mode: SuiteModeArg::All,
            group: None,
            tag: None,
            format: SuiteOutputFormatArg::Human,
            quiet: false,
            verbose: false,
            color: SuiteColorArg::Never,
            strict: false,
            out: None,
        })
        .expect("suite run");

        let latest = render_suite_last(&root.path().join("artifacts"), "checks", FormatArg::Text)
            .expect("latest");
        assert!(latest.ends_with("artifacts/suites/checks/checks-report"));

        let (rendered, _) = run_report_command(
            &root.path().join("artifacts"),
            "checks",
            "checks-report",
            false,
            None,
            None,
            SuiteOutputFormatArg::Human,
            false,
            false,
            SuiteColorArg::Never,
        )
        .expect("report");
        assert!(rendered.contains("CHECK-GIT-VERSION-001"));
    }

    #[test]
    fn diff_reports_duration_regressions() {
        let root = fixture_root();
        let baseline_root = root.path().join("artifacts/suites/checks/run-a");
        let current_root = root.path().join("artifacts/suites/checks/run-b");
        fs::create_dir_all(&baseline_root).expect("baseline root");
        fs::create_dir_all(&current_root).expect("current root");
        write_json(
            &baseline_root.join("suite-summary.json"),
            &serde_json::json!({
                "report_id":"suite-summary",
                "version":1,
                "inputs":{"suite":"checks","run_id":"run-a"},
                "suite":"checks",
                "run_id":"run-a",
                "run_timestamp":"1",
                "artifacts_root": baseline_root.display().to_string(),
                "elapsed_ms": 10,
                "status":"pass",
                "summary":{"pass":1,"fail":0,"skip":0,"total":1},
                "failures":[],
                "tasks":[{
                    "id":"CHECK-GIT-VERSION-001",
                    "status":"pass",
                    "group":"rust",
                    "mode":"pure",
                    "duration_ms":10,
                    "summary":"git version",
                    "error_summary": null,
                    "result_path": baseline_root.join("CHECK-GIT-VERSION-001/result.json").display().to_string(),
                    "stdout_path": "",
                    "stderr_path": ""
                }]
            }),
        );
        write_json(
            &current_root.join("suite-summary.json"),
            &serde_json::json!({
                "report_id":"suite-summary",
                "version":1,
                "inputs":{"suite":"checks","run_id":"run-b"},
                "suite":"checks",
                "run_id":"run-b",
                "run_timestamp":"2",
                "artifacts_root": current_root.display().to_string(),
                "elapsed_ms": 50,
                "status":"pass",
                "summary":{"pass":1,"fail":0,"skip":0,"total":1},
                "failures":[],
                "tasks":[{
                    "id":"CHECK-GIT-VERSION-001",
                    "status":"pass",
                    "group":"rust",
                    "mode":"pure",
                    "duration_ms":50,
                    "summary":"git version",
                    "error_summary": null,
                    "result_path": current_root.join("CHECK-GIT-VERSION-001/result.json").display().to_string(),
                    "stdout_path": "",
                    "stderr_path": ""
                }]
            }),
        );
        let (rendered, code) = run_diff_command(
            root.path(),
            &root.path().join("artifacts"),
            "checks",
            "run-a",
            "run-b",
            SuiteOutputFormatArg::Human,
            false,
        )
        .expect("diff");
        assert_eq!(code, 0);
        assert!(rendered.contains("Slowdowns"));
        assert!(rendered.contains("CHECK-GIT-VERSION-001"));
    }
}
