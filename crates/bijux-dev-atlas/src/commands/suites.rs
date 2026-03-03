// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::cli::{FormatArg, SuiteModeArg, SuitesCommand};
use crate::resolve_repo_root;
use bijux_dev_atlas::docs::site_output::validate_named_report;
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
    reports: Vec<String>,
    tags: Option<Vec<String>>,
    retries: Option<u64>,
    requires_tools: Option<Vec<String>>,
    missing_tools_policy: Option<String>,
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
    reports: Vec<String>,
    retries: u64,
    requires_tools: Vec<String>,
    missing_tools_policy: String,
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
    format: FormatArg,
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

fn suite_result_schema_name() -> &'static str {
    "suite-result.schema.json"
}

fn suite_summary_schema_name() -> &'static str {
    "suite-summary.schema.json"
}

fn suite_preflight_schema_name() -> &'static str {
    "suite-preflight.schema.json"
}

fn normalize_suite_id(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace('-', "_")
}

fn load_suites_index(root: &Path) -> Result<SuitesIndex, String> {
    let index: SuitesIndex = read_json_file(&suites_index_path(root))?;
    if index.schema_version != 1 || index.index_id != "governance-suites" {
        return Err("suites index must declare schema_version=1 and index_id=governance-suites"
            .to_string());
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
                        reports: entry.reports,
                        retries: entry.retries.unwrap_or(0),
                        requires_tools: entry.requires_tools.unwrap_or_default(),
                        missing_tools_policy: entry
                            .missing_tools_policy
                            .unwrap_or_else(|| "fail".to_string()),
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
                    entry_map.get(&entry.contract_id).map(|suite_entry| SuiteTask {
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
                        reports: entry.reports,
                        retries: entry.retries.unwrap_or(0),
                        requires_tools: entry.requires_tools.unwrap_or_default(),
                        missing_tools_policy: entry
                            .missing_tools_policy
                            .unwrap_or_else(|| "fail".to_string()),
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
            "bijux dev atlas contracts ops --mode {} --only {}",
            if mode == "effect" { "effect" } else { "static" },
            contract_id
        )
    } else {
        command.replace("--only-contract", "--only")
    }
}

fn parse_jobs(raw: &str) -> Result<usize, String> {
    if raw.eq_ignore_ascii_case("auto") {
        let count = std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1);
        return Ok(count.min(8).max(1));
    }
    let jobs = raw
        .parse::<usize>()
        .map_err(|err| format!("invalid jobs value `{raw}`: {err}"))?;
    if jobs == 0 {
        return Err("jobs must be at least 1".to_string());
    }
    Ok(jobs)
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
        let exe = std::env::current_exe().map_err(|err| format!("resolve current exe failed: {err}"))?;
        return Ok((
            exe.display().to_string(),
            parts.into_iter().skip(3).collect::<Vec<_>>(),
        ));
    }
    Ok((parts[0].clone(), parts.into_iter().skip(1).collect::<Vec<_>>()))
}

fn git_sha(root: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output();
    match output {
        Ok(value) if value.status.success() => String::from_utf8_lossy(&value.stdout).trim().to_string(),
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
                format!("suite_id: {}", payload["suite_id"].as_str().unwrap_or_default()),
                format!("purpose: {}", payload["purpose"].as_str().unwrap_or_default()),
                format!("owner: {}", payload["owner"].as_str().unwrap_or_default()),
                format!("stability: {}", payload["stability"].as_str().unwrap_or_default()),
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
        left.cmp(&right)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(tasks)
}

fn task_report_paths(task: &SuiteTask, task_root: &Path) -> Vec<String> {
    task.reports
        .iter()
        .map(|report| task_root.join(report).display().to_string())
        .collect()
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
        .filter(|row| row["missing_tools"].as_array().is_some_and(|value| !value.is_empty()))
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
        error_summary: Some(format!("missing required tools: {}", missing_tools.join(", "))),
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

    Ok(TaskOutput {
        task: task.clone(),
        status,
        duration_ms: start.elapsed().as_millis() as u64,
        report_paths: task_report_paths(task, task_root),
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
) -> serde_json::Value {
    let failures = tasks
        .iter()
        .filter(|task| task.status == "fail")
        .map(|task| serde_json::json!({
            "id": task.task.id,
            "group": task.task.group,
            "error_summary": task.error_summary,
        }))
        .collect::<Vec<_>>();
    let pass = tasks.iter().filter(|task| task.status == "pass").count();
    let fail = tasks.iter().filter(|task| task.status == "fail").count();
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
        "status": if fail == 0 { "pass" } else { "fail" },
        "summary": {
            "pass": pass,
            "fail": fail,
            "skip": skip,
            "total": tasks.len(),
        },
        "failures": failures,
        "tasks": tasks.iter().map(|task| serde_json::json!({
            "id": task.task.id,
            "status": task.status,
            "group": task.task.group,
            "mode": task.task.mode,
            "duration_ms": task.duration_ms,
            "result_path": artifacts_root.join(&task.task.id).join("result.json").display().to_string(),
        })).collect::<Vec<_>>()
    })
}

fn write_task_result(repo_root: &Path, task_root: &Path, task_output: &TaskOutput) -> Result<(), String> {
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
                    let mut guard = queue.lock().expect("suite queue mutex poisoned");
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
                let should_stop = matches!(&result, Ok(output) if output.status == "fail") && fail_fast;
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

fn render_run_output(summary: &serde_json::Value, format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => Ok(format!(
            "suite={} run_id={} status={} pass={} fail={} skip={} total={}",
            summary["suite"].as_str().unwrap_or_default(),
            summary["run_id"].as_str().unwrap_or_default(),
            summary["status"].as_str().unwrap_or_default(),
            summary["summary"]["pass"].as_u64().unwrap_or(0),
            summary["summary"]["fail"].as_u64().unwrap_or(0),
            summary["summary"]["skip"].as_u64().unwrap_or(0),
            summary["summary"]["total"].as_u64().unwrap_or(0),
        )),
        FormatArg::Json => serde_json::to_string_pretty(summary)
            .map_err(|err| format!("encode suite summary failed: {err}")),
        FormatArg::Jsonl => Err("jsonl output is not supported for suites run".to_string()),
    }
}

fn execute_suite_run(options: SuiteRunOptions) -> Result<(String, i32), String> {
    let suite_id = normalize_suite_id(&options.suite);
    if options.fail_fast == Some(true) && options.fail_fast == Some(false) {
        return Err("invalid fail-fast state".to_string());
    }
    let jobs = parse_jobs(&options.jobs)?;
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
    validate_named_report(&options.repo_root, suite_preflight_schema_name(), &preflight)?;
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

    let mut executed = run_tasks_parallel(
        &options.repo_root,
        &suite_root,
        runnable,
        jobs,
        options.fail_fast.unwrap_or(false),
    )?;
    task_outputs.append(&mut executed);
    task_outputs.sort_by(|a, b| a.task.id.cmp(&b.task.id));
    let summary = summary_report(&suite_id, &run_id, &task_outputs, &suite_root);
    validate_named_report(&options.repo_root, suite_summary_schema_name(), &summary)?;
    let summary_path = suite_root.join("suite-summary.json");
    let summary_text = serde_json::to_string_pretty(&summary)
        .map_err(|err| format!("encode suite summary failed: {err}"))?;
    fs::write(&summary_path, format!("{summary_text}\n"))
        .map_err(|err| format!("write {} failed: {err}", summary_path.display()))?;
    let rendered = render_run_output(&summary, options.format)?;
    write_output_if_requested(options.out, &rendered)?;
    let exit = if summary["status"].as_str() == Some("pass") { 0 } else { 1 };
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
    let tasks = select_tasks(&root, "checks", SuiteModeArg::All, None, None)?;
    let known = tasks
        .iter()
        .map(|task| task.id.clone())
        .collect::<BTreeSet<_>>();
    if !known.contains(&check_id) {
        return Err(format!("unknown check id `{check_id}`"));
    }
    let options = SuiteRunOptions {
        suite: "checks".to_string(),
        repo_root: root.clone(),
        artifacts_root: artifacts_root.unwrap_or_else(|| root.join("artifacts")),
        run_id_override: run_id.map(|value| format!("check-{check_id}-{value}")),
        jobs: "1".to_string(),
        fail_fast: Some(fail_fast),
        mode: SuiteModeArg::All,
        group: None,
        tag: None,
        format,
        out: out.clone(),
    };
    let suite_id = normalize_suite_id(&options.suite);
    let run_id = match options.run_id_override.clone() {
        Some(value) => value,
        None => deterministic_run_id(&options.repo_root, &suite_id, options.mode, None, None)?,
    };
    let suite_root = options
        .artifacts_root
        .join("suites")
        .join(&suite_id)
        .join(&run_id);
    fs::create_dir_all(&suite_root)
        .map_err(|err| format!("create {} failed: {err}", suite_root.display()))?;
    let task = tasks
        .into_iter()
        .find(|task| task.id == check_id)
        .ok_or_else(|| format!("unknown check id `{check_id}`"))?;
    let task_root = suite_root.join(&task.id);
    let output = run_task(&options.repo_root, &task_root, &task)?;
    write_task_result(&options.repo_root, &task_root, &output)?;
    let summary = summary_report(&suite_id, &run_id, std::slice::from_ref(&output), &suite_root);
    validate_named_report(&options.repo_root, suite_summary_schema_name(), &summary)?;
    let summary_text = serde_json::to_string_pretty(&summary)
        .map_err(|err| format!("encode suite summary failed: {err}"))?;
    fs::write(suite_root.join("suite-summary.json"), format!("{summary_text}\n"))
        .map_err(|err| format!("write suite summary failed: {err}"))?;
    let rendered = render_run_output(&summary, format)?;
    write_output_if_requested(out, &rendered)?;
    let exit = if output.status == "pass" { 0 } else { 1 };
    Ok((rendered, exit))
}

fn normalized_contract_runner(task: &SuiteTask) -> Vec<String> {
    task.commands
        .iter()
        .map(|command| normalized_contract_command(command, &task.mode, &task.id))
        .collect()
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
    let task = select_tasks(&root, "contracts", SuiteModeArg::All, None, None)?
        .into_iter()
        .find(|task| task.id == contract_id)
        .ok_or_else(|| format!("unknown contract id `{contract_id}`"))?;
    let effective_run_id = run_id.unwrap_or_else(|| format!("contract-{}", contract_id.to_ascii_lowercase()));
    let suite_root = artifacts_root
        .unwrap_or_else(|| root.join("artifacts"))
        .join("suites")
        .join("contracts")
        .join(&effective_run_id);
    fs::create_dir_all(&suite_root)
        .map_err(|err| format!("create {} failed: {err}", suite_root.display()))?;

    let mut runnable = task.clone();
    runnable.commands = normalized_contract_runner(&task);
    let preflight = suite_preflight_report("contracts", &effective_run_id, std::slice::from_ref(&runnable));
    validate_named_report(&root, suite_preflight_schema_name(), &preflight)?;
    let preflight_text = serde_json::to_string_pretty(&preflight)
        .map_err(|err| format!("encode suite preflight failed: {err}"))?;
    fs::write(suite_root.join("suite-preflight.json"), format!("{preflight_text}\n"))
        .map_err(|err| format!("write suite preflight failed: {err}"))?;

    let task_root = suite_root.join(&runnable.id);
    let missing_tools = runnable
        .requires_tools
        .iter()
        .filter(|tool| !tool_in_path(tool))
        .cloned()
        .collect::<Vec<_>>();
    let output = if missing_tools.is_empty() {
        let output = run_task(&root, &task_root, &runnable)?;
        write_task_result(&root, &task_root, &output)?;
        output
    } else {
        let output = skipped_or_failed_output(&runnable, missing_tools);
        fs::create_dir_all(&task_root)
            .map_err(|err| format!("create {} failed: {err}", task_root.display()))?;
        write_task_result(&root, &task_root, &output)?;
        output
    };

    let summary = summary_report(
        "contracts",
        &effective_run_id,
        std::slice::from_ref(&output),
        &suite_root,
    );
    validate_named_report(&root, suite_summary_schema_name(), &summary)?;
    let summary_text = serde_json::to_string_pretty(&summary)
        .map_err(|err| format!("encode suite summary failed: {err}"))?;
    fs::write(suite_root.join("suite-summary.json"), format!("{summary_text}\n"))
        .map_err(|err| format!("write suite summary failed: {err}"))?;
    let rendered = render_run_output(&summary, format)?;
    write_output_if_requested(out, &rendered)?;
    let exit = if output.status == "pass" { 0 } else { 1 };
    Ok((rendered, exit))
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
                    fail_fast: if no_fail_fast { Some(false) } else { Some(fail_fast) },
                    mode,
                    group,
                    tag,
                    format,
                    out,
                })
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
            &dir.path().join("configs/governance/suites/suites.index.json"),
            &serde_json::json!({
                "schema_version": 1,
                "index_id": "governance-suites",
                "suites": ["checks", "contracts", "tests"]
            }),
        );
        write_json(
            &dir.path().join("configs/governance/suites/checks.suite.json"),
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
            &dir.path().join("configs/governance/suites/contracts.suite.json"),
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
            &dir.path().join("configs/governance/checks.registry.json"),
            &serde_json::json!({
                "checks": [{
                    "check_id":"CHECK-GIT-VERSION-001",
                    "summary":"git version",
                    "owner":"team:atlas-governance",
                    "mode":"pure",
                    "group":"rust",
                    "commands":["git --version"],
                    "reports":["check-git-version.json"],
                    "tags":["rust"]
                }]
            }),
        );
        write_json(
            &dir.path().join("configs/governance/contracts.registry.json"),
            &serde_json::json!({
                "contracts": [{
                    "contract_id":"CONTRACT-GIT-VERSION-001",
                    "summary":"git version",
                    "owner":"team:atlas-governance",
                    "mode":"pure",
                    "group":"ops",
                    "runner":"git --version",
                    "reports":["contract-git-version.json"],
                    "tags":["ops"]
                }]
            }),
        );
        write_json(
            &dir.path().join("configs/contracts/reports/suite-result.schema.json"),
            &serde_json::json!({
                "required":["report_id","version","inputs","id","status","duration_ms","mode","group","report_paths"],
                "properties":{
                    "report_id":{"const":"suite-result"},
                    "version":{"const":1},
                    "inputs":{"type":"object"},
                    "id":{"type":"string"},
                    "status":{"type":"string"},
                    "duration_ms":{"type":"integer"},
                    "mode":{"type":"string"},
                    "group":{"type":"string"},
                    "report_paths":{"type":"array"}
                }
            }),
        );
        write_json(
            &dir.path().join("configs/contracts/reports/suite-summary.schema.json"),
            &serde_json::json!({
                "required":["report_id","version","inputs","suite","run_id","status","summary","failures","tasks"],
                "properties":{
                    "report_id":{"const":"suite-summary"},
                    "version":{"const":1},
                    "inputs":{"type":"object"},
                    "suite":{"type":"string"},
                    "run_id":{"type":"string"},
                    "status":{"type":"string"},
                    "summary":{"type":"object"},
                    "failures":{"type":"array"},
                    "tasks":{"type":"array"}
                }
            }),
        );
        write_json(
            &dir.path().join("configs/contracts/reports/suite-preflight.schema.json"),
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
            format: FormatArg::Json,
            out: None,
        })
        .expect("suite run");
        assert_eq!(result.1, 0);
        let summary_path = root
            .path()
            .join("artifacts/suites/checks/checks-test/suite-summary.json");
        assert!(summary_path.exists());
    }
}
