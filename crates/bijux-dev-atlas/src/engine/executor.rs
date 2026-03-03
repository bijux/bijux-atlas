// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::model::{
    CheckId, CheckResult, CheckStatus, Effect, ReportRef, RunId, RunReport, RunSummary,
    RunnableEntry, RunnableId,
};
use crate::runtime::{Capabilities, Fs, ProcessRunner};

use super::reporting::ArtifactStore;

#[derive(Debug, Clone)]
pub struct EffectPolicy {
    pub capabilities: Capabilities,
}

impl EffectPolicy {
    pub fn allows(&self, effect: Effect) -> bool {
        match effect {
            Effect::FsRead => true,
            Effect::FsWrite => self.capabilities.fs_write,
            Effect::Subprocess => self.capabilities.subprocess,
            Effect::Git => self.capabilities.git,
            Effect::Network => self.capabilities.network,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunnableRunContext {
    pub repo_root: std::path::PathBuf,
    pub run_id: RunId,
    pub artifact_store: ArtifactStore,
    pub effect_policy: EffectPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    Pass,
    Fail,
    Skip,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    pub reason: String,
    pub required_tool: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub runnable_id: RunnableId,
    pub status: RunStatus,
    pub duration_ms: u64,
    pub skipped: Option<Skipped>,
    pub report_refs: Vec<ReportRef>,
    pub artifacts: Vec<String>,
    pub failure_summary: Option<String>,
}

pub trait RunnableExecutor {
    fn execute(
        &self,
        entry: &RunnableEntry,
        context: &RunnableRunContext,
    ) -> Result<RunResult, String>;
}

pub enum Parallelism {
    Sequential,
    Bounded(usize),
}

pub fn execute_all<E: RunnableExecutor + Send + Sync + 'static>(
    executor: Arc<E>,
    entries: Vec<RunnableEntry>,
    context: RunnableRunContext,
    parallelism: Parallelism,
) -> Result<Vec<RunResult>, String> {
    match parallelism {
        Parallelism::Sequential => {
            let mut results = Vec::new();
            for entry in entries {
                results.push(executor.execute(&entry, &context)?);
            }
            Ok(results)
        }
        Parallelism::Bounded(limit) if limit <= 1 => {
            execute_all(executor, entries, context, Parallelism::Sequential)
        }
        Parallelism::Bounded(limit) => {
            let queue = Arc::new(Mutex::new(entries));
            let results = Arc::new(Mutex::new(Vec::<RunResult>::new()));
            let first_error = Arc::new(Mutex::new(None::<String>));
            let mut workers = Vec::new();
            for _ in 0..limit {
                let queue = Arc::clone(&queue);
                let results = Arc::clone(&results);
                let first_error = Arc::clone(&first_error);
                let executor = Arc::clone(&executor);
                let context = context.clone();
                workers.push(thread::spawn(move || loop {
                    let next = {
                        let mut queue = queue.lock().expect("queue lock");
                        if queue.is_empty() {
                            None
                        } else {
                            Some(queue.remove(0))
                        }
                    };
                    let Some(entry) = next else {
                        break;
                    };
                    match executor.execute(&entry, &context) {
                        Ok(result) => results.lock().expect("results lock").push(result),
                        Err(err) => {
                            let mut slot = first_error.lock().expect("error lock");
                            if slot.is_none() {
                                *slot = Some(err);
                            }
                            break;
                        }
                    }
                }));
            }
            for worker in workers {
                let _ = worker.join();
            }
            if let Some(err) = first_error.lock().expect("error lock").clone() {
                return Err(err);
            }
            let mut out = results.lock().expect("results lock").clone();
            out.sort_by(|a, b| a.runnable_id.as_str().cmp(b.runnable_id.as_str()));
            Ok(out)
        }
    }
}

pub fn core_run_report(
    run_id: RunId,
    repo_root: String,
    command: String,
    selections: BTreeMap<String, String>,
    capabilities: Capabilities,
    results: Vec<CheckResult>,
) -> RunReport {
    let timings = results
        .iter()
        .map(|row| (row.id.clone(), row.duration_ms))
        .collect::<BTreeMap<CheckId, u64>>();
    let summary = RunSummary {
        schema_version: crate::model::schema_version(),
        passed: results
            .iter()
            .filter(|row| row.status == CheckStatus::Pass)
            .count() as u64,
        failed: results
            .iter()
            .filter(|row| row.status == CheckStatus::Fail)
            .count() as u64,
        skipped: results
            .iter()
            .filter(|row| row.status == CheckStatus::Skip)
            .count() as u64,
        errors: results
            .iter()
            .filter(|row| row.status == CheckStatus::Error)
            .count() as u64,
        total: results.len() as u64,
    };
    RunReport {
        schema_version: crate::model::schema_version(),
        run_id,
        repo_root,
        command,
        selections,
        capabilities: BTreeMap::from([
            ("fs_write".to_string(), capabilities.fs_write),
            ("subprocess".to_string(), capabilities.subprocess),
            ("git".to_string(), capabilities.git),
            ("network".to_string(), capabilities.network),
        ]),
        results,
        durations_ms: timings.clone(),
        counts: summary.clone(),
        summary,
        timings_ms: timings,
    }
}

pub fn start_timer() -> Instant {
    Instant::now()
}

pub fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

pub fn check_effect_gate(entry: &RunnableEntry, context: &RunnableRunContext) -> Option<Skipped> {
    let missing_tool = entry
        .required_tools
        .iter()
        .find(|tool| !tool_in_path(tool))
        .cloned();
    if let Some(tool) = missing_tool {
        return Some(Skipped {
            reason: format!("required tool `{tool}` is not available"),
            required_tool: Some(tool),
        });
    }
    let denied = entry
        .effects_required
        .iter()
        .find(|effect| !context.effect_policy.allows(**effect));
    denied.map(|effect| Skipped {
        reason: format!("effect denied: {effect:?}"),
        required_tool: None,
    })
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

pub struct CommandRunnableExecutor<'a> {
    pub process: &'a dyn ProcessRunner,
    pub fs: &'a dyn Fs,
}

impl RunnableExecutor for CommandRunnableExecutor<'_> {
    fn execute(
        &self,
        entry: &RunnableEntry,
        context: &RunnableRunContext,
    ) -> Result<RunResult, String> {
        if let Some(skipped) = check_effect_gate(entry, context) {
            let payload = serde_json::json!({
                "report_id": "run-result",
                "version": 1,
                "inputs": {
                    "commands": entry.commands,
                    "effects_required": entry.effects_required,
                },
                "artifacts": [],
                "run_id": context.run_id.as_str(),
                "runnable_id": entry.id.as_str(),
                "status": "skip",
                "skipped": {
                    "reason": skipped.reason,
                    "required_tool": skipped.required_tool,
                }
            });
            let report = context.artifact_store.write_json_report(
                &context.run_id,
                &entry.id,
                "run-result",
                &payload,
            )?;
            return Ok(RunResult {
                runnable_id: entry.id.clone(),
                status: RunStatus::Skip,
                duration_ms: 0,
                skipped: Some(skipped),
                report_refs: vec![report],
                artifacts: Vec::new(),
                failure_summary: None,
            });
        }

        let start = start_timer();
        let mut failure_summary = None;
        let mut status = RunStatus::Pass;
        for command in &entry.commands {
            let (program, args) = parse_command_invocation(command)?;
            let code = self
                .process
                .run(&program, &args, &context.repo_root)
                .map_err(|err| err.to_string())?;
            if code != 0 {
                status = RunStatus::Fail;
                failure_summary = Some(format!("{command} exited with status {code}"));
                break;
            }
        }
        let duration_ms = elapsed_ms(start);
        let status_text = match status {
            RunStatus::Pass => "pass",
            RunStatus::Fail => "fail",
            RunStatus::Skip => "skip",
            RunStatus::Error => "error",
        };
        let payload = serde_json::json!({
            "report_id": "run-result",
            "version": 1,
            "inputs": {
                "commands": entry.commands,
                "effects_required": entry.effects_required,
            },
            "artifacts": [],
            "run_id": context.run_id.as_str(),
            "runnable_id": entry.id.as_str(),
            "status": status_text,
            "duration_ms": duration_ms,
            "failure_summary": failure_summary,
            "declared_reports": entry.report_ids,
        });
        let report = context.artifact_store.write_json_report(
            &context.run_id,
            &entry.id,
            "run-result",
            &payload,
        )?;
        let _ = self
            .fs
            .exists(&context.repo_root, std::path::Path::new("."));
        Ok(RunResult {
            runnable_id: entry.id.clone(),
            status,
            duration_ms,
            skipped: None,
            report_refs: vec![report],
            artifacts: Vec::new(),
            failure_summary,
        })
    }
}

fn parse_command_invocation(command: &str) -> Result<(String, Vec<String>), String> {
    let parts = command
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let Some((program, args)) = parts.split_first() else {
        return Err("command must not be empty".to_string());
    };
    Ok((program.clone(), args.to_vec()))
}
