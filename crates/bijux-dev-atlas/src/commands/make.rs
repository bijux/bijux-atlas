// SPDX-License-Identifier: Apache-2.0

use crate::cli::{MakeCommand, MakeExplainArgs, MakeVerifyArgs};
use crate::{emit_payload, resolve_repo_root};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub(crate) fn run_make_command(quiet: bool, command: MakeCommand) -> i32 {
    let run: Result<(String, i32), String> = {
        let started = Instant::now();
        match command {
            MakeCommand::VerifyModule(args) => run_verify_module(args, started),
            MakeCommand::Surface(common) => run_surface(common, started),
            MakeCommand::List(common) => run_list(common, started),
            MakeCommand::Explain(args) => run_explain(args, started),
            MakeCommand::TargetList(common) => run_target_list(common, started),
            MakeCommand::LintPolicyReport(common) => run_lint_policy_report(common, started),
        }
    };
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas make failed: {err}");
            1
        }
    }
}

fn run_surface(
    common: crate::cli::MakeCommonArgs,
    _started: Instant,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let targets = load_curated_targets(&repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "surface",
        "source": "make/root.mk:CURATED_TARGETS",
        "public_targets": targets,
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_list(
    common: crate::cli::MakeCommonArgs,
    started: Instant,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let targets = load_curated_targets(&repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "list",
        "source": "make/root.mk:CURATED_TARGETS",
        "public_targets": targets,
        "target_count": targets.len(),
        "duration_ms": started.elapsed().as_millis() as u64,
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_explain(args: MakeExplainArgs, started: Instant) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let target = args.target.trim();
    if target.is_empty() {
        return Err("make explain requires a target name".to_string());
    }
    let targets = load_curated_targets(&repo_root)?;
    let known = targets.iter().any(|row| row == target);
    let payload = if known {
        serde_json::json!({
            "schema_version": 1,
            "action": "explain",
            "target": target,
            "known": true,
            "text": format!("{target} is a curated make wrapper target"),
            "guidance": "This target is a thin dispatcher. Keep orchestration in bijux dev atlas commands.",
            "source": "make/root.mk:CURATED_TARGETS",
            "duration_ms": started.elapsed().as_millis() as u64,
        })
    } else {
        serde_json::json!({
            "schema_version": 1,
            "action": "explain",
            "target": target,
            "known": false,
            "text": format!("{target} is not part of the curated make surface"),
            "hint": "Use `bijux dev atlas make list --format json` to inspect supported targets.",
            "source": "make/root.mk:CURATED_TARGETS",
            "duration_ms": started.elapsed().as_millis() as u64,
        })
    };
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, if known { 0 } else { 2 }))
}

fn run_target_list(
    common: crate::cli::MakeCommonArgs,
    started: Instant,
) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("make target-list requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let targets = load_curated_targets(&repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "source": "make/root.mk:CURATED_TARGETS",
        "public_targets": targets,
    });
    let output_path = repo_root.join("make/target-list.json");
    fs::write(
        &output_path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())? + "\n",
    )
    .map_err(|err| format!("write {} failed: {err}", output_path.display()))?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "action": "target-list",
        "text": "make target list regenerated",
        "output_path": output_path.display().to_string(),
        "public_target_count": payload["public_targets"].as_array().map(|rows| rows.len()).unwrap_or(0),
        "duration_ms": started.elapsed().as_millis() as u64,
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}

fn run_lint_policy_report(
    common: crate::cli::MakeCommonArgs,
    started: Instant,
) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("make lint-policy-report requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let output_path = repo_root.join("artifacts/lint/effective-clippy-policy.txt");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml"))
        .map_err(|err| format!("read Cargo.toml failed: {err}"))?;
    let clippy_toml = fs::read_to_string(repo_root.join("configs/rust/clippy.toml"))
        .map_err(|err| format!("read configs/rust/clippy.toml failed: {err}"))?;
    let workspace_lints = extract_workspace_lints(&cargo_toml);
    let cargo_clippy_version = ProcessCommand::new("cargo")
        .current_dir(&repo_root)
        .args(["clippy", "--version"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or_default()
        .trim()
        .to_string();
    let rendered_report = format!(
        "schema_version=1\nworkspace_lints_file=Cargo.toml\nclippy_conf_dir=configs/rust\nclippy_conf_file=configs/rust/clippy.toml\ncargo_clippy_version={cargo_clippy_version}\nworkspace_lints:\n{workspace_lints}\nclippy_toml:\n{clippy_toml}"
    );
    fs::write(&output_path, rendered_report)
        .map_err(|err| format!("write {} failed: {err}", output_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "lint-policy-report",
        "text": "effective clippy policy report generated",
        "output_path": output_path.display().to_string(),
        "cargo_clippy_version": cargo_clippy_version,
        "duration_ms": started.elapsed().as_millis() as u64,
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_verify_module(args: MakeVerifyArgs, started: Instant) -> Result<(String, i32), String> {
    if !args.common.allow_subprocess {
        return Err("make verify-module requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let module = args.module.trim().to_string();
    if module.is_empty() {
        return Err("make verify-module requires a module name".to_string());
    }
    let makefile = resolve_make_module(&repo_root, &module)?;
    let targets = discover_targets(&makefile)?;
    if targets.is_empty() {
        return Err(format!(
            "make verify-module found no runnable targets in {}",
            makefile.display()
        ));
    }

    let accept_codes = accepted_exit_codes(&module);
    let mut rows = Vec::new();
    let mut failed = 0usize;
    for (index, target) in targets.iter().enumerate() {
        let result = if target.ends_with("-serve") {
            run_serve_target(&repo_root, &module, target, &accept_codes)?
        } else {
            run_target(&repo_root, target, &accept_codes)?
        };
        if !result.accepted {
            failed += 1;
        }
        rows.push(serde_json::json!({
            "order": index + 1,
            "target": target,
            "status": if result.accepted { "pass" } else { "fail" },
            "exit_code": result.exit_code,
            "detail": result.detail,
            "log_path": result.log_path.map(|path| path.display().to_string()),
        }));
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "verify-module",
        "module": module,
        "makefile": makefile.display().to_string(),
        "text": if failed == 0 {
            "make module verification passed"
        } else {
            "make module verification failed"
        },
        "rows": rows,
        "counts": {
            "targets": targets.len(),
            "failed": failed,
        },
        "accepted_exit_codes": accept_codes,
        "repo_root": repo_root.display().to_string(),
        "duration_ms": started.elapsed().as_millis() as u64,
        "capabilities": {
            "subprocess": args.common.allow_subprocess,
        }
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, if failed == 0 { 0 } else { 1 }))
}

fn resolve_make_module(repo_root: &Path, module: &str) -> Result<PathBuf, String> {
    let primary = repo_root.join("make").join(format!("{module}.mk"));
    if primary.is_file() {
        return Ok(primary);
    }
    let underscored = repo_root.join("make").join(format!("_{module}.mk"));
    if underscored.is_file() {
        return Ok(underscored);
    }
    Err(format!(
        "make verify-module could not find make/{module}.mk or make/_{module}.mk"
    ))
}

fn load_curated_targets(repo_root: &Path) -> Result<Vec<String>, String> {
    let root_mk = fs::read_to_string(repo_root.join("make/root.mk"))
        .map_err(|err| format!("read make/root.mk failed: {err}"))?;
    let mut collecting = false;
    let mut targets = Vec::new();
    for line in root_mk.lines() {
        let trimmed = line.trim();
        if !collecting && trimmed.starts_with("CURATED_TARGETS :=") {
            collecting = true;
        }
        if !collecting {
            continue;
        }
        for token in trimmed.replace('\\', " ").split_whitespace() {
            if token
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            {
                targets.push(token.to_string());
            }
        }
        if !trimmed.ends_with('\\') {
            break;
        }
    }
    targets.sort();
    targets.dedup();
    if targets.is_empty() {
        return Err("CURATED_TARGETS is missing or empty".to_string());
    }
    Ok(targets)
}

fn extract_workspace_lints(cargo_toml: &str) -> String {
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.lints.rust]" {
            in_section = true;
        }
        if in_section {
            if trimmed.starts_with('[') && trimmed != "[workspace.lints.rust]" && !lines.is_empty()
            {
                break;
            }
            lines.push(line.to_string());
        }
    }
    if lines.is_empty() {
        "[workspace.lints.rust]".to_string()
    } else {
        lines.join("\n")
    }
}

fn discover_targets(makefile: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(makefile)
        .map_err(|err| format!("read {} failed: {err}", makefile.display()))?;
    let mut targets = Vec::new();
    for line in text.lines() {
        if line.starts_with('\t') {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('.') {
            continue;
        }
        if trimmed.contains(":=") || trimmed.contains("?=") {
            continue;
        }
        let Some((head, tail)) = trimmed.split_once(':') else {
            continue;
        };
        if tail.starts_with('=') {
            continue;
        }
        let target = head.trim();
        if target.is_empty()
            || target.starts_with('_')
            || target.starts_with('.')
            || target.ends_with("/internal")
            || target.contains("/internal/")
        {
            continue;
        }
        if !target
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '/' | '-'))
        {
            continue;
        }
        targets.push(target.to_string());
    }
    targets.sort();
    targets.dedup();
    Ok(targets)
}

fn accepted_exit_codes(module: &str) -> Vec<i32> {
    match module {
        "docs" | "_docs" | "configs" | "_configs" => vec![0, 2],
        _ => vec![0],
    }
}

struct TargetResult {
    accepted: bool,
    exit_code: i32,
    detail: String,
    log_path: Option<PathBuf>,
}

fn run_target(
    repo_root: &Path,
    target: &str,
    accept_codes: &[i32],
) -> Result<TargetResult, String> {
    let status = ProcessCommand::new("make")
        .current_dir(repo_root)
        .args(["--no-print-directory", "-s", target])
        .status()
        .map_err(|err| format!("failed to run make {target}: {err}"))?;
    let exit_code = status.code().unwrap_or(1);
    let accepted = accept_codes.contains(&exit_code);
    Ok(TargetResult {
        accepted,
        exit_code,
        detail: if accepted {
            if exit_code == 0 {
                "pass".to_string()
            } else {
                format!("accepted exit={exit_code}")
            }
        } else {
            format!("exit={exit_code}")
        },
        log_path: None,
    })
}

fn run_serve_target(
    repo_root: &Path,
    module: &str,
    target: &str,
    accept_codes: &[i32],
) -> Result<TargetResult, String> {
    let safe_target = target.replace('/', "-");
    let log_dir = repo_root.join("artifacts").join("verification");
    fs::create_dir_all(&log_dir)
        .map_err(|err| format!("create {} failed: {err}", log_dir.display()))?;
    let log_path = log_dir.join(format!("{module}-{safe_target}.log"));
    let stdout = File::create(&log_path)
        .map_err(|err| format!("create {} failed: {err}", log_path.display()))?;
    let stderr = stdout
        .try_clone()
        .map_err(|err| format!("clone {} failed: {err}", log_path.display()))?;
    let mut child = ProcessCommand::new("make")
        .current_dir(repo_root)
        .args(["--no-print-directory", "-s", target])
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|err| format!("failed to run make {target}: {err}"))?;
    thread::sleep(Duration::from_secs(2));
    if child
        .try_wait()
        .map_err(|err| format!("poll make {target} failed: {err}"))?
        .is_none()
    {
        let _ = child.kill();
        let _ = child.wait();
        return Ok(TargetResult {
            accepted: true,
            exit_code: 0,
            detail: "startup verified; process terminated".to_string(),
            log_path: Some(log_path),
        });
    }
    let status = child
        .wait()
        .map_err(|err| format!("wait for make {target} failed: {err}"))?;
    let exit_code = status.code().unwrap_or(1);
    let accepted = accept_codes.contains(&exit_code);
    Ok(TargetResult {
        accepted,
        exit_code,
        detail: if accepted {
            format!("accepted exit={exit_code}")
        } else {
            format!("exit={exit_code}")
        },
        log_path: Some(log_path),
    })
}

#[cfg(test)]
mod tests {
    use super::{run_explain, run_list};
    use crate::cli::{FormatArg, MakeCommonArgs, MakeExplainArgs};
    use serde_json::Value;
    use std::fs;
    use std::time::Instant;

    fn write_minimal_root_mk(root: &std::path::Path) {
        fs::create_dir_all(root.join("make")).expect("create make dir");
        fs::write(
            root.join("make/root.mk"),
            "CURATED_TARGETS := \\\n\tfmt help test\n",
        )
        .expect("write root.mk");
    }

    fn common(repo_root: &std::path::Path) -> MakeCommonArgs {
        MakeCommonArgs {
            repo_root: Some(repo_root.to_path_buf()),
            format: FormatArg::Json,
            out: None,
            allow_subprocess: false,
            allow_write: false,
        }
    }

    #[test]
    fn make_list_is_sorted_and_stable() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_root_mk(temp.path());
        let (rendered, code) = run_list(common(temp.path()), Instant::now()).expect("run list");
        assert_eq!(code, 0);
        let payload: Value = serde_json::from_str(&rendered).expect("parse payload");
        assert_eq!(payload["action"], "list");
        assert_eq!(payload["public_targets"], serde_json::json!(["fmt", "help", "test"]));
    }

    #[test]
    fn make_explain_unknown_target_is_stable() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_root_mk(temp.path());
        let args = MakeExplainArgs {
            common: common(temp.path()),
            target: "unknown-target".to_string(),
        };
        let (rendered, code) = run_explain(args, Instant::now()).expect("run explain");
        assert_eq!(code, 2);
        let payload: Value = serde_json::from_str(&rendered).expect("parse payload");
        assert_eq!(payload["action"], "explain");
        assert_eq!(payload["known"], false);
        assert_eq!(
            payload["hint"],
            "Use `bijux dev atlas make list --format json` to inspect supported targets."
        );
    }

    #[test]
    fn make_list_excludes_hidden_and_invalid_tokens() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join("make")).expect("create make dir");
        fs::write(
            temp.path().join("make/root.mk"),
            "CURATED_TARGETS := \\\n\tfmt _hidden help invalid$name test-all\n",
        )
        .expect("write root.mk");
        let (rendered, code) = run_list(common(temp.path()), Instant::now()).expect("run list");
        assert_eq!(code, 0);
        let payload: Value = serde_json::from_str(&rendered).expect("parse payload");
        assert_eq!(
            payload["public_targets"],
            serde_json::json!(["fmt", "help", "test-all"])
        );
    }
}
