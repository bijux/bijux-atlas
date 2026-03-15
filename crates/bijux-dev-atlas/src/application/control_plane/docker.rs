// SPDX-License-Identifier: Apache-2.0

use crate::cli::{DockerCommand, DockerCommonArgs, DockerPolicyCommand, DockerReleaseArgs};
use crate::*;
use serde_json::json;
use std::io::{self, Write};

fn docker_rows(repo_root: &Path) -> Result<Vec<serde_json::Value>, String> {
    let mut rows = Vec::new();
    let checks = [
        ("policy", repo_root.join("ops/docker/policy.json")),
        (
            "manifest",
            repo_root.join("ops/docker/images.manifest.json"),
        ),
        (
            "runtime_dockerfile",
            repo_root.join("ops/docker/images/runtime/Dockerfile"),
        ),
    ];
    for (id, path) in checks {
        rows.push(json!({
            "id": id,
            "path": path.strip_prefix(repo_root).unwrap_or(&path).display().to_string(),
            "status": if path.exists() { "ok" } else { "missing" },
        }));
    }
    Ok(rows)
}

fn emit_result(
    common: &DockerCommonArgs,
    payload: serde_json::Value,
    code: i32,
) -> Result<(String, i32), String> {
    Ok((
        emit_payload(common.format, common.out.clone(), &payload)?,
        code,
    ))
}

fn require_subprocess(common: &DockerCommonArgs, action: &str) -> Result<(), String> {
    if common.allow_subprocess {
        Ok(())
    } else {
        Err(format!("docker {action} requires --allow-subprocess"))
    }
}

fn require_network(common: &DockerCommonArgs, action: &str) -> Result<(), String> {
    if common.allow_network {
        Ok(())
    } else {
        Err(format!("docker {action} requires --allow-network"))
    }
}

fn require_write(common: &DockerCommonArgs, action: &str) -> Result<(), String> {
    if common.allow_write {
        Ok(())
    } else {
        Err(format!("docker {action} requires --allow-write"))
    }
}

fn run_check(common: DockerCommonArgs) -> Result<(String, i32), String> {
    require_subprocess(&common, "check")?;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let rows = docker_rows(&repo_root)?;
    let failed = rows.iter().any(|row| row["status"] == "missing");
    emit_result(
        &common,
        json!({
            "schema_version": 1,
            "action": "check",
            "status": if failed { "failed" } else { "ok" },
            "rows": rows,
        }),
        if failed { 1 } else { 0 },
    )
}

fn run_policy_check(common: DockerCommonArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let rows = docker_rows(&repo_root)?;
    let failed = rows.iter().any(|row| row["status"] == "missing");
    emit_result(
        &common,
        json!({
            "schema_version": 1,
            "action": "policy_check",
            "status": if failed { "failed" } else { "ok" },
            "rows": rows,
        }),
        if failed { 1 } else { 0 },
    )
}

fn run_release(args: DockerReleaseArgs) -> Result<(String, i32), String> {
    require_subprocess(&args.common, "release")?;
    require_network(&args.common, "release")?;
    if !args.i_know_what_im_doing {
        return Err("docker release requires --i-know-what-im-doing".to_string());
    }
    emit_result(
        &args.common,
        json!({
            "schema_version": 1,
            "action": "release",
            "status": "ready",
        }),
        0,
    )
}

pub(crate) fn run_docker_command(quiet: bool, command: DockerCommand) -> i32 {
    let result = (|| -> Result<(String, i32), String> {
        match command {
            DockerCommand::Build(common) => {
                require_subprocess(&common, "build")?;
                emit_result(
                    &common,
                    json!({"schema_version": 1, "action": "build", "status": "ready"}),
                    0,
                )
            }
            DockerCommand::Check(common) => run_check(common),
            DockerCommand::Smoke(common) => {
                require_subprocess(&common, "smoke")?;
                emit_result(
                    &common,
                    json!({"schema_version": 1, "action": "smoke", "status": "ready"}),
                    0,
                )
            }
            DockerCommand::Scan(common) => {
                require_subprocess(&common, "scan")?;
                require_network(&common, "scan")?;
                emit_result(
                    &common,
                    json!({"schema_version": 1, "action": "scan", "status": "ready"}),
                    0,
                )
            }
            DockerCommand::Sbom(common) => {
                require_subprocess(&common, "sbom")?;
                emit_result(
                    &common,
                    json!({"schema_version": 1, "action": "sbom", "status": "ready"}),
                    0,
                )
            }
            DockerCommand::Lock(common) => {
                require_write(&common, "lock")?;
                emit_result(
                    &common,
                    json!({"schema_version": 1, "action": "lock", "status": "ready"}),
                    0,
                )
            }
            DockerCommand::Policy { command } => match command {
                DockerPolicyCommand::Check(common) => run_policy_check(common),
            },
            DockerCommand::Push(args) => {
                require_subprocess(&args.common, "push")?;
                require_network(&args.common, "push")?;
                if !args.i_know_what_im_doing {
                    return Err("docker push requires --i-know-what-im-doing".to_string());
                }
                emit_result(
                    &args.common,
                    json!({"schema_version": 1, "action": "push", "status": "ready"}),
                    0,
                )
            }
            DockerCommand::Release(args) => run_release(args),
        }
    })();

    match result {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = if code == 0 {
                    writeln!(io::stdout(), "{rendered}")
                } else {
                    writeln!(io::stderr(), "{rendered}")
                };
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas docker failed: {err}");
            1
        }
    }
}
