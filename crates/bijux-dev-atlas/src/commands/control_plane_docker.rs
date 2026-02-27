// SPDX-License-Identifier: Apache-2.0

use crate::cli::{DockerCommand, DockerCommonArgs, DockerPolicyCommand};
use crate::*;
use bijux_dev_atlas::policies::DevAtlasPolicySet;
use std::collections::VecDeque;
use std::io::{self, Write};

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let mut queue = VecDeque::from([root.to_path_buf()]);
    while let Some(dir) = queue.pop_front() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut paths = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                queue.push_back(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out
}

fn policies_inventory_rows(
    doc: &bijux_dev_atlas::policies::DevAtlasPolicySetDocument,
) -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "repo",
            "title": "repository structure and budget policy",
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "ops",
            "title": "ops registry policy",
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "compatibility",
            "title": "policy mode compatibility matrix",
            "count": doc.compatibility.len(),
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "documented_defaults",
            "title": "documented default exceptions",
            "count": doc.documented_defaults.len(),
            "schema_version": doc.schema_version.as_str()
        }),
    ]
}

pub(crate) fn run_policies_list(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let rows = policies_inventory_rows(&doc);
    let payload = serde_json::json!({
        "schema_version": 1,
        "repo_root": root.display().to_string(),
        "rows": rows,
        "text": "control-plane policies listed"
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_explain(
    policy_id: String,
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let payload = match policy_id.as_str() {
        "repo" => serde_json::json!({
            "schema_version": 1,
            "id": "repo",
            "repo_root": root.display().to_string(),
            "title": "repository structure and budget policy",
            "fields": doc.repo,
        }),
        "ops" => serde_json::json!({
            "schema_version": 1,
            "id": "ops",
            "repo_root": root.display().to_string(),
            "title": "ops registry policy",
            "fields": doc.ops,
        }),
        "compatibility" => serde_json::json!({
            "schema_version": 1,
            "id": "compatibility",
            "repo_root": root.display().to_string(),
            "title": "policy mode compatibility matrix",
            "rows": doc.compatibility,
        }),
        "documented_defaults" => serde_json::json!({
            "schema_version": 1,
            "id": "documented_defaults",
            "repo_root": root.display().to_string(),
            "title": "documented default exceptions",
            "rows": doc.documented_defaults,
        }),
        _ => {
            return Err(format!(
                "unknown policy id `{}` (expected repo|ops|compatibility|documented_defaults)",
                policy_id
            ))
        }
    };
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_report(
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
        "policy_schema_version": doc.schema_version.as_str(),
        "mode": format!("{:?}", doc.mode).to_ascii_lowercase(),
        "policy_count": policies_inventory_rows(&doc).len(),
        "capabilities": {"fs_write": false, "subprocess": false, "network": false, "git": false},
        "report_kind": "control_plane_policies"
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

include!("control_plane_docker_runtime_helpers.inc.rs");

pub(crate) fn run_docker_command(quiet: bool, command: DockerCommand) -> i32 {
    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        let emit = |common: &DockerCommonArgs, payload: serde_json::Value, code: i32| {
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, code))
        };
        match command {
            DockerCommand::Contracts(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "status": "ok",
                    "rows": docker_contract_rows(&repo_root)?,
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Gates(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "status": "ok",
                    "rows": docker_gate_rows(&repo_root)?,
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Doctor(common) => {
                if !common.allow_subprocess {
                    return Err("docker doctor requires --allow-subprocess".to_string());
                }
                let mut rows = Vec::new();
                let (docker_code, _, docker_err) = run_subprocess(
                    &resolve_repo_root(common.repo_root.clone())?,
                    "docker",
                    &["version", "--format", "{{.Server.Version}}"],
                )?;
                rows.push(serde_json::json!({"check":"docker_version","status": if docker_code == 0 { "ok" } else { "failed" }, "stderr": docker_err}));
                let (syft_code, _, _) = run_subprocess(
                    &resolve_repo_root(common.repo_root.clone())?,
                    "syft",
                    &["version"],
                )
                .unwrap_or((1, String::new(), String::new()));
                rows.push(serde_json::json!({"check":"syft","status": if syft_code == 0 { "ok" } else { "failed" }}));
                let (trivy_code, _, _) = run_subprocess(
                    &resolve_repo_root(common.repo_root.clone())?,
                    "trivy",
                    &["--version"],
                )
                .unwrap_or((1, String::new(), String::new()));
                rows.push(serde_json::json!({"check":"trivy","status": if trivy_code == 0 { "ok" } else { "failed" }}));
                let failures = rows.iter().filter(|r| r["status"] == "failed").count();
                let payload = serde_json::json!({
                    "schema_version":1,
                    "status": if failures == 0 { "ok" } else { "failed" },
                    "rows": rows,
                    "summary": {"errors": failures, "warnings":0}
                });
                emit(&common, payload, if failures == 0 { 0 } else { 1 })
            }
            DockerCommand::Validate(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                check_contract_gate_mapping(&repo_root)?;
                let rows = validate_dockerfiles(&repo_root)?;
                let violations = rows.len();
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_validate"));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "status": if violations == 0 { "ok" } else { "failed" },
                    "text": if violations == 0 { "docker validate passed" } else { "docker validate found violations" },
                    "rows": rows,
                    "summary": {"errors": violations, "warnings": 0},
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                let artifact_dir = docker_artifact_dir(&common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let validate_report_path = artifact_dir.join("validate.json");
                fs::write(
                    &validate_report_path,
                    serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())? + "\n",
                )
                .map_err(|e| format!("cannot write {}: {e}", validate_report_path.display()))?;
                emit(&common, payload, if violations == 0 { 0 } else { 1 })
            }
            DockerCommand::Build(common) => {
                if !common.allow_subprocess {
                    return Err("docker build requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_build"));
                let tag = image_tag_for_run(&run_id);
                let artifact_dir = docker_artifact_dir(&common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let (code, stdout, stderr) = run_subprocess(
                    &repo_root,
                    "docker",
                    &[
                        "build",
                        "--file",
                        "docker/images/runtime/Dockerfile",
                        "--tag",
                        &tag,
                        ".",
                    ],
                )?;
                fs::write(artifact_dir.join("build.stdout.log"), &stdout)
                    .map_err(|e| format!("cannot write build stdout log: {e}"))?;
                fs::write(artifact_dir.join("build.stderr.log"), &stderr)
                    .map_err(|e| format!("cannot write build stderr log: {e}"))?;
                let (inspect_code, inspect_stdout, _) = run_subprocess(
                    &repo_root,
                    "docker",
                    &["image", "inspect", &tag, "--format", "{{.Id}}"],
                )
                .unwrap_or((1, String::new(), String::new()));
                let (size_code, size_stdout, size_stderr) = run_subprocess(
                    &repo_root,
                    "docker",
                    &["image", "inspect", &tag, "--format", "{{.Size}}"],
                )
                .unwrap_or((1, String::new(), String::new()));
                let actual_size = size_stdout.trim().parse::<u64>().unwrap_or_default();
                let max_size = runtime_image_budget_bytes(&repo_root)?;
                let size_ok = size_code == 0 && actual_size <= max_size;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "text": "docker build completed",
                    "rows": [
                        {"action":"build","repo_root": repo_root.display().to_string(), "image_tag": tag, "image_id": inspect_stdout.trim(), "inspect_status": inspect_code},
                        {"action":"image_size","gate_id":"docker.contract.image_size","contract_id":"DOCKER-010","status": if size_ok { "ok" } else { "failed" }, "actual_bytes": actual_size, "max_bytes": max_size, "stderr": size_stderr}
                    ],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                let final_code = if code == 0 && size_ok { 0 } else { 1 };
                emit(&common, payload, final_code)
            }
            DockerCommand::Check(common) => {
                let validate = run_docker_command(true, DockerCommand::Validate(common.clone()));
                emit(
                    &common,
                    serde_json::json!({"schema_version":1,"status": if validate == 0 { "ok" } else { "failed" }, "rows":[{"action":"check","validate_exit_code": validate}]}),
                    validate,
                )
            }
            DockerCommand::Smoke(common) => {
                if !common.allow_subprocess {
                    return Err("docker smoke requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_smoke"));
                let tag = image_tag_for_run(&run_id);
                let (help_code, help_out, help_err) = run_subprocess(
                    &repo_root,
                    "docker",
                    &[
                        "run",
                        "--rm",
                        "--entrypoint",
                        "/app/bijux-atlas",
                        &tag,
                        "--help",
                    ],
                )?;
                let (version_code, version_out, version_err) = run_subprocess(
                    &repo_root,
                    "docker",
                    &[
                        "run",
                        "--rm",
                        "--entrypoint",
                        "/app/bijux-atlas",
                        &tag,
                        "--version",
                    ],
                )?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker smoke executed",
                    "rows": [
                        {"action":"smoke_help","status": help_code, "stdout": help_out, "stderr": help_err, "contract_id":"DOCKER-007"},
                        {"action":"smoke_version","status": version_code, "stdout": version_out, "stderr": version_err, "contract_id":"DOCKER-007"}
                    ],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(
                    &common,
                    payload,
                    if help_code == 0 && version_code == 0 {
                        0
                    } else {
                        1
                    },
                )
            }
            DockerCommand::Scan(common) => {
                if !common.allow_subprocess {
                    return Err("docker scan requires --allow-subprocess".to_string());
                }
                if !common.allow_network {
                    return Err("docker scan requires --allow-network".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_scan"));
                let tag = image_tag_for_run(&run_id);
                let artifact_dir = docker_artifact_dir(&common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let report_path = artifact_dir.join("scan.trivy.json");
                let report_arg = report_path.display().to_string();
                let args = [
                    "image",
                    "--format",
                    "json",
                    "--output",
                    report_arg.as_str(),
                    tag.as_str(),
                ];
                let (code, stdout, stderr) = run_subprocess(&repo_root, "trivy", &args)?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker scan executed",
                    "rows": [{"action":"scan","repo_root": repo_root.display().to_string(),"scanner":"trivy","report":report_path.display().to_string(),"stdout":stdout,"stderr":stderr,"contract_id":"DOCKER-009"}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, code)
            }
            DockerCommand::Sbom(common) => {
                if !common.allow_subprocess {
                    return Err("docker sbom requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_sbom"));
                let tag = image_tag_for_run(&run_id);
                let artifact_dir = docker_artifact_dir(&common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let spdx_path = artifact_dir.join("sbom.spdx.json");
                let cyclonedx_path = artifact_dir.join("sbom.cyclonedx.json");
                let spdx_out = spdx_path.display().to_string();
                let cyclonedx_out = cyclonedx_path.display().to_string();
                let (spdx_code, spdx_stdout, spdx_stderr) = run_subprocess(
                    &repo_root,
                    "syft",
                    &[tag.as_str(), "-o", "spdx-json", "--file", spdx_out.as_str()],
                )?;
                let (cyclonedx_code, _, cyclonedx_stderr) = run_subprocess(
                    &repo_root,
                    "syft",
                    &[
                        tag.as_str(),
                        "-o",
                        "cyclonedx-json",
                        "--file",
                        cyclonedx_out.as_str(),
                    ],
                )?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker sbom executed",
                    "rows": [
                        {"action":"sbom","repo_root": repo_root.display().to_string(),"tool":"syft","format":"spdx-json","path":spdx_path.display().to_string(),"stdout":spdx_stdout,"stderr":spdx_stderr,"contract_id":"DOCKER-008"},
                        {"action":"sbom","repo_root": repo_root.display().to_string(),"tool":"syft","format":"cyclonedx-json","path":cyclonedx_path.display().to_string(),"stderr":cyclonedx_stderr,"contract_id":"DOCKER-008"}
                    ],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(
                    &common,
                    payload,
                    if spdx_code == 0 && cyclonedx_code == 0 {
                        0
                    } else {
                        1
                    },
                )
            }
            DockerCommand::Lock(common) => {
                if !common.allow_write {
                    return Err("docker lock requires --allow-write".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let lock_path = repo_root.join("ops/inventory/image-digests.lock.json");
                if let Some(parent) = lock_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
                }
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_lock"));
                let tag = image_tag_for_run(&run_id);
                let (inspect_code, inspect_stdout, inspect_stderr) = run_subprocess(
                    &repo_root,
                    "docker",
                    &["image", "inspect", &tag, "--format", "{{.Id}}"],
                )
                .unwrap_or((1, String::new(), String::new()));
                let lock_payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "docker_image_lock",
                    "images": [{"tag": tag, "digest": inspect_stdout.trim(), "inspect_status": inspect_code, "inspect_stderr": inspect_stderr}],
                    "timestamp_policy": "forbidden_by_default"
                });
                fs::write(
                    &lock_path,
                    serde_json::to_string_pretty(&lock_payload).map_err(|e| e.to_string())? + "\n",
                )
                .map_err(|e| format!("cannot write {}: {e}", lock_path.display()))?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker lockfile written under ops/inventory",
                    "rows": [{"action":"lock","path": lock_path.display().to_string()}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Policy { command } => match command {
                DockerPolicyCommand::Check(common) => {
                    let code = run_docker_command(true, DockerCommand::Validate(common.clone()));
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "action": "policy_check",
                        "status": if code == 0 { "ok" } else { "failed" },
                        "text": "docker policy check delegates to docker validate",
                        "rows": [{"action":"delegate","to":"docker validate","exit_code":code}],
                        "summary": {"errors": if code == 0 { 0 } else { 1 }, "warnings": 0},
                        "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                        "duration_ms": started.elapsed().as_millis() as u64
                    });
                    emit(&common, payload, code)
                }
            },
            DockerCommand::Push(args) => {
                if !args.i_know_what_im_doing {
                    return Err("docker push requires --i-know-what-im-doing".to_string());
                }
                if !args.common.allow_subprocess {
                    return Err("docker push requires --allow-subprocess".to_string());
                }
                if !args.common.allow_network {
                    return Err("docker push requires --allow-network".to_string());
                }
                let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
                let run_id = args
                    .common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_push"));
                let tag = image_tag_for_run(&run_id);
                let artifact_dir = docker_artifact_dir(&args.common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let (code, stdout, stderr) = run_subprocess(&repo_root, "docker", &["push", &tag])?;
                fs::write(artifact_dir.join("push.stdout.log"), &stdout)
                    .map_err(|e| format!("cannot write push stdout log: {e}"))?;
                fs::write(artifact_dir.join("push.stderr.log"), &stderr)
                    .map_err(|e| format!("cannot write push stderr log: {e}"))?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "text": "docker push executed",
                    "rows": [{"action":"push","repo_root": repo_root.display().to_string(),"image_tag":tag,"stdout":stdout,"stderr":stderr}],
                    "capabilities": {"subprocess": args.common.allow_subprocess, "fs_write": args.common.allow_write, "network": args.common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&args.common, payload, code)
            }
            DockerCommand::Release(args) => {
                if !args.i_know_what_im_doing {
                    return Err("docker release requires --i-know-what-im-doing".to_string());
                }
                if !args.common.allow_subprocess {
                    return Err("docker release requires --allow-subprocess".to_string());
                }
                if !args.common.allow_network {
                    return Err("docker release requires --allow-network".to_string());
                }
                let validate_code =
                    run_docker_command(true, DockerCommand::Validate(args.common.clone()));
                let build_code =
                    run_docker_command(true, DockerCommand::Build(args.common.clone()));
                let smoke_code =
                    run_docker_command(true, DockerCommand::Smoke(args.common.clone()));
                let sbom_code = run_docker_command(true, DockerCommand::Sbom(args.common.clone()));
                let scan_code = run_docker_command(true, DockerCommand::Scan(args.common.clone()));
                let push_code = run_docker_command(
                    true,
                    DockerCommand::Push(crate::cli::DockerReleaseArgs {
                        common: args.common.clone(),
                        i_know_what_im_doing: true,
                    }),
                );
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker release executed",
                    "rows": [
                        {"action":"validate","exit_code":validate_code},
                        {"action":"build","exit_code":build_code},
                        {"action":"smoke","exit_code":smoke_code},
                        {"action":"sbom","exit_code":sbom_code},
                        {"action":"scan","exit_code":scan_code},
                        {"action":"push","exit_code":push_code}
                    ],
                    "capabilities": {"subprocess": args.common.allow_subprocess, "fs_write": args.common.allow_write, "network": args.common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                let code = [
                    validate_code,
                    build_code,
                    smoke_code,
                    sbom_code,
                    scan_code,
                    push_code,
                ]
                .iter()
                .copied()
                .max()
                .unwrap_or(0);
                emit(&args.common, payload, code)
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
            let _ = writeln!(io::stderr(), "bijux-dev-atlas docker failed: {err}");
            1
        }
    }
}
