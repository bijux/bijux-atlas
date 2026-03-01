// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

const DOMAIN_DIRS: &[&str] = &[
    "datasets",
    "e2e",
    "env",
    "inventory",
    "k8s",
    "load",
    "observe",
    "report",
    "schema",
    "stack",
];

mod common;

fn violation(contract_id: &str, test_id: &str, message: &str, file: Option<String>) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line: Some(1),
        message: message.to_string(),
        evidence: None,
    }
}

fn ops_root(repo_root: &Path) -> PathBuf {
    repo_root.join("ops")
}

fn walk_files(root: &Path, out: &mut Vec<PathBuf>) {
    common::walk_files(root, out)
}

fn rel_to_root(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn read_json(path: &Path) -> Option<Value> {
    common::read_json(path)
}

fn markdown_line_count(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .map(|c| c.lines().count())
        .unwrap_or(0)
}

fn file_sha256(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Some(format!("{:x}", hasher.finalize()))
}

fn sha256_text(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn ops_effect_artifact_dir(ctx: &RunContext) -> Option<PathBuf> {
    let root = ctx.artifacts_root.as_ref()?;
    let dir = root.join("contracts/ops");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn write_ops_effect_json(
    ctx: &RunContext,
    rel: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    let Some(root) = ops_effect_artifact_dir(ctx) else {
        return Ok(());
    };
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(value)
            .map_err(|err| format!("encode {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn append_ops_effect_log(ctx: &RunContext, entry: &serde_json::Value) {
    let Some(root) = ops_effect_artifact_dir(ctx) else {
        return;
    };
    let path = root.join("effects.log");
    let mut line = serde_json::to_string(entry)
        .unwrap_or_else(|_| "{\"status\":\"encode-error\"}".to_string());
    line.push('\n');
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()));
}

struct OpsEffectCommand<'a> {
    contract_id: &'a str,
    test_id: &'a str,
    program: &'a str,
    args: &'a [&'a str],
    stdout_rel: String,
    stderr_rel: String,
    network_allowed: bool,
}

fn run_ops_effect_command(
    ctx: &RunContext,
    spec: OpsEffectCommand<'_>,
) -> Result<std::process::Output, TestResult> {
    if !ctx.allow_subprocess {
        return Err(TestResult::Error("requires --allow-subprocess".to_string()));
    }
    if spec.network_allowed && !ctx.allow_network {
        return Err(TestResult::Error("requires --allow-network".to_string()));
    }
    let started = Instant::now();
    let mut command = Command::new(spec.program);
    command
        .args(spec.args)
        .current_dir(&ctx.repo_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("TZ", "UTC")
        .env("LC_ALL", "C")
        .env("LANG", "C");
    let output = match command.output() {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound && ctx.skip_missing_tools => {
            return Err(TestResult::Skip(format!(
                "`{}` is not installed",
                spec.program
            )));
        }
        Err(err) => {
            return Err(TestResult::Error(format!(
                "spawn `{}` failed: {err}",
                spec.program
            )));
        }
    };
    if let Some(root) = ops_effect_artifact_dir(ctx) {
        let stdout_path = root.join(&spec.stdout_rel);
        let stderr_path = root.join(&spec.stderr_rel);
        if let Some(parent) = stdout_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(stdout_path, &output.stdout);
        if let Some(parent) = stderr_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(stderr_path, &output.stderr);
    }
    append_ops_effect_log(
        ctx,
        &serde_json::json!({
            "contract_id": spec.contract_id,
            "test_id": spec.test_id,
            "program": spec.program,
            "args": spec.args,
            "network_allowed": spec.network_allowed,
            "duration_ms": started.elapsed().as_millis() as u64,
            "status": output.status.code(),
        }),
    );
    Ok(output)
}

fn normalize_major_version(raw: &str) -> Option<u64> {
    let trimmed = raw.trim().trim_start_matches('v');
    trimmed.split('.').next()?.parse::<u64>().ok()
}

fn verify_declared_tool_versions(ctx: &RunContext, tool_names: &[&str]) -> TestResult {
    let policy_rel = "ops/policy/effect-tool-version-policy.json";
    let Some(policy) = read_json(&ctx.repo_root.join(policy_rel)) else {
        return TestResult::Fail(vec![violation(
            "OPS-ROOT-001",
            "ops.effect.tools.version_policy_present",
            "effect tool version policy must be valid json",
            Some(policy_rel.to_string()),
        )]);
    };
    let mut rows = Vec::new();
    let mut violations = Vec::new();
    for tool in tool_names {
        let output = match run_ops_effect_command(
            ctx,
            OpsEffectCommand {
                contract_id: "OPS-ROOT-001",
                test_id: "ops.effect.tools.version_policy_present",
                program: tool,
                args: &["version"],
                stdout_rel: format!("tools/{tool}.stdout.log"),
                stderr_rel: format!("tools/{tool}.stderr.log"),
                network_allowed: false,
            },
        ) {
            Ok(output) => output,
            Err(TestResult::Skip(reason)) => {
                violations.push(violation(
                    "OPS-ROOT-001",
                    "ops.effect.tools.version_policy_present",
                    &format!("required effect tool `{tool}` is unavailable: {reason}"),
                    Some(policy_rel.to_string()),
                ));
                continue;
            }
            Err(other) => return other,
        };
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let combined = if stdout.is_empty() {
            stderr.clone()
        } else {
            format!("{stdout}\n{stderr}")
        };
        let version = combined
            .split_whitespace()
            .find(|part| {
                part.starts_with('v') || part.chars().next().is_some_and(|c| c.is_ascii_digit())
            })
            .unwrap_or_default()
            .to_string();
        rows.push(serde_json::json!({
            "tool": tool,
            "version": version,
            "raw": combined.trim(),
        }));
        let Some(expected_major) = policy
            .get("tools")
            .and_then(|value| value.get(*tool))
            .and_then(|value| value.get("allowed_major"))
            .and_then(|value| value.as_u64())
        else {
            violations.push(violation(
                "OPS-ROOT-001",
                "ops.effect.tools.version_policy_present",
                "tool version policy must declare allowed_major for every effect tool",
                Some(policy_rel.to_string()),
            ));
            continue;
        };
        match normalize_major_version(&version) {
            Some(actual_major) if actual_major == expected_major => {}
            Some(actual_major) => violations.push(violation(
                "OPS-ROOT-001",
                "ops.effect.tools.version_policy_present",
                &format!("tool `{tool}` major version `{actual_major}` must match allowlisted major `{expected_major}`"),
                Some(policy_rel.to_string()),
            )),
            None => violations.push(violation(
                "OPS-ROOT-001",
                "ops.effect.tools.version_policy_present",
                &format!("tool `{tool}` version output could not be normalized"),
                Some(policy_rel.to_string()),
            )),
        }
    }
    let _ = write_ops_effect_json(
        ctx,
        "tools.json",
        &serde_json::json!({
            "schema_version": 1,
            "rows": rows
        }),
    );
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

include!("root/mod.rs");
include!("governance/mod.rs");
include!("inventory/mod.rs");
include!("datasets/mod.rs");
include!("e2e/mod.rs");
include!("environment/mod.rs");
include!("platform/mod.rs");
include!("load/mod.rs");
include!("observe/mod.rs");
include!("reporting/mod.rs");
