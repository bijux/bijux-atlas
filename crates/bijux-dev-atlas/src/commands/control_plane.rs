// SPDX-License-Identifier: Apache-2.0

use crate::cli::{DockerCommand, DockerCommonArgs, DockerPolicyCommand, PoliciesCommand};
use crate::*;
use bijux_dev_atlas::model::CONTRACT_SCHEMA_VERSION;
use bijux_dev_atlas::policies::{canonical_policy_json, DevAtlasPolicySet};
use std::collections::VecDeque;
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

pub(crate) fn run_docker_command(quiet: bool, command: DockerCommand) -> i32 {
    fn extract_copy_sources(line: &str) -> Option<Vec<String>> {
        let trimmed = line.trim();
        if !trimmed.starts_with("COPY ") || trimmed.contains("--from=") {
            return None;
        }
        let rest = trimmed.trim_start_matches("COPY ").trim();
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        if tokens.len() < 2 {
            return None;
        }
        Some(
            tokens[..tokens.len() - 1]
                .iter()
                .map(|s| s.trim_matches('"').to_string())
                .collect(),
        )
    }

    fn run_instruction_has_unapproved_network_usage(line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.starts_with("RUN ") {
            return false;
        }
        let allowed = [
            "apt-get update",
            "apt-get install",
            "rm -rf /var/lib/apt/lists/*",
            "cargo build --locked",
            "--mount=type=cache",
        ];
        if allowed.iter().any(|token| trimmed.contains(token)) {
            return false;
        }
        let disallowed = [
            "curl ",
            "wget ",
            "git clone",
            "pip ",
            "npm ",
            "go get",
            "apk add",
            "dnf ",
            "yum ",
            "apt ",
            "cargo install",
        ];
        disallowed.iter().any(|token| trimmed.contains(token))
    }

    fn validate_runtime_dockerfile(repo_root: &Path) -> Result<Vec<serde_json::Value>, String> {
        let dockerfile = repo_root.join("docker/images/runtime/Dockerfile");
        let text = fs::read_to_string(&dockerfile)
            .map_err(|e| format!("failed to read {}: {e}", dockerfile.display()))?;
        let policy_path = repo_root.join("docker/contracts/digest-pinning.json");
        let policy_text = fs::read_to_string(&policy_path)
            .map_err(|e| format!("failed to read {}: {e}", policy_path.display()))?;
        let policy: serde_json::Value = serde_json::from_str(&policy_text)
            .map_err(|e| format!("failed to parse {}: {e}", policy_path.display()))?;
        let exceptions = policy["allow_tagged_images_exceptions"]
            .as_array()
            .ok_or_else(|| "digest pinning policy missing allowlist array".to_string())?
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();

        let mut rows = Vec::new();
        let mut violations = 0usize;
        for (idx, line) in text.lines().enumerate() {
            if let Some(srcs) = extract_copy_sources(line) {
                for src in srcs {
                    if src == "." || src.starts_with('/') {
                        continue;
                    }
                    if !repo_root.join(&src).exists() {
                        violations += 1;
                        rows.push(serde_json::json!({
                            "kind":"copy_source_missing",
                            "line": idx + 1,
                            "path": src
                        }));
                    }
                }
            }
            let trimmed = line.trim();
            if run_instruction_has_unapproved_network_usage(trimmed) {
                violations += 1;
                rows.push(serde_json::json!({
                    "kind":"build_network_policy_violation",
                    "line": idx + 1,
                    "instruction": trimmed
                }));
            }
            if !trimmed.starts_with("FROM ") {
                continue;
            }
            let from_spec = trimmed
                .split_whitespace()
                .nth(1)
                .ok_or_else(|| format!("invalid FROM line in {}: {}", dockerfile.display(), trimmed))?;
            let uses_latest = from_spec.ends_with(":latest") || from_spec == "latest";
            let is_digest_pinned = from_spec.contains("@sha256:");
            let is_allowlisted = exceptions.iter().any(|e| e == &from_spec);
            if uses_latest || (!is_digest_pinned && !is_allowlisted) {
                violations += 1;
                rows.push(serde_json::json!({
                    "kind":"base_image_policy_violation",
                    "line": idx + 1,
                    "image": from_spec
                }));
            }
        }
        rows.push(serde_json::json!({
            "kind":"summary",
            "dockerfile": dockerfile.display().to_string(),
            "violations": violations
        }));
        Ok(rows)
    }

    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        let emit = |common: &DockerCommonArgs, payload: serde_json::Value, code: i32| {
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, code))
        };
        match command {
            DockerCommand::Validate(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let rows = validate_runtime_dockerfile(&repo_root)?;
                let violations = rows
                    .iter()
                    .filter(|r| r["kind"] != "summary")
                    .count();
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "status": if violations == 0 { "ok" } else { "failed" },
                    "text": if violations == 0 { "docker validate passed" } else { "docker validate found violations" },
                    "rows": rows,
                    "summary": {"errors": violations, "warnings": 0},
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
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
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "text": "docker build wrapper is defined (subprocess-gated)",
                    "rows": [{"action":"build","repo_root": repo_root.display().to_string()}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Check(common) => {
                if !common.allow_subprocess {
                    return Err("docker check requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker check wrapper is defined (subprocess-gated)",
                    "rows": [{"action":"check","repo_root": repo_root.display().to_string()}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Smoke(common) => {
                if !common.allow_subprocess {
                    return Err("docker smoke requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker smoke wrapper is defined (subprocess-gated)",
                    "rows": [{"action":"smoke","repo_root": repo_root.display().to_string()}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Scan(common) => {
                if !common.allow_subprocess {
                    return Err("docker scan requires --allow-subprocess".to_string());
                }
                if !common.allow_network {
                    return Err("docker scan requires --allow-network".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker scan wrapper is defined (subprocess and network gated)",
                    "rows": [{"action":"scan","repo_root": repo_root.display().to_string(),"scanner":"trivy_or_grype"}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Sbom(common) => {
                if !common.allow_subprocess {
                    return Err("docker sbom requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker sbom wrapper is defined (subprocess-gated)",
                    "rows": [{"action":"sbom","repo_root": repo_root.display().to_string(),"tool":"syft"}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, 0)
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
                let lock_payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "docker_image_lock",
                    "images": [],
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
                    let repo_root = resolve_repo_root(common.repo_root.clone())?;
                    let docker_root = repo_root.join("docker");
                    let mut rows = Vec::new();
                    let mut errors = 0usize;
                    if docker_root.exists() {
                        for file in walk_files(&docker_root) {
                            if let Ok(text) = fs::read_to_string(&file) {
                                let rel = file.strip_prefix(&repo_root).unwrap_or(&file);
                                for line in text.lines() {
                                    let trimmed = line.trim();
                                    if trimmed.contains(":latest") {
                                        errors += 1;
                                        rows.push(serde_json::json!({"path": rel.display().to_string(), "violation": "floating_latest_tag", "line": trimmed}));
                                    }
                                }
                            }
                        }
                    }
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "action": "policy_check",
                        "status": if errors == 0 { "ok" } else { "failed" },
                        "text": "docker policy check for floating tags",
                        "rows": rows,
                        "summary": {"errors": errors, "warnings": 0},
                        "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                        "duration_ms": started.elapsed().as_millis() as u64
                    });
                    emit(&common, payload, if errors == 0 { 0 } else { 1 })
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
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker push wrapper is defined (explicit release gate)",
                    "rows": [{"action":"push","repo_root": repo_root.display().to_string()}],
                    "capabilities": {"subprocess": args.common.allow_subprocess, "fs_write": args.common.allow_write, "network": args.common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&args.common, payload, 0)
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
                let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker release wrapper is defined (explicit release gate)",
                    "rows": [{"action":"release","repo_root": repo_root.display().to_string()}],
                    "capabilities": {"subprocess": args.common.allow_subprocess, "fs_write": args.common.allow_write, "network": args.common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&args.common, payload, 0)
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
