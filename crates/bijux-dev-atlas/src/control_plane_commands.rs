use crate::cli::{
    BuildCleanArgs, BuildCommand, BuildCommonArgs, DockerCommand, DockerCommonArgs, PoliciesCommand,
};
use crate::*;
use bijux_dev_atlas_model::CONTRACT_SCHEMA_VERSION;
use bijux_dev_atlas_policies::{canonical_policy_json, DevAtlasPolicySet};

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
                println!("{rendered}");
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas policies failed: {err}");
            1
        }
    }
}

fn policies_inventory_rows(doc: &bijux_dev_atlas_policies::DevAtlasPolicySetDocument) -> Vec<serde_json::Value> {
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
    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        let emit = |common: &DockerCommonArgs, payload: serde_json::Value, code: i32| {
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, code))
        };
        match command {
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
            DockerCommand::Push(args) => {
                if !args.i_know_what_im_doing {
                    return Err("docker push requires --i-know-what-im-doing".to_string());
                }
                if !args.common.allow_subprocess {
                    return Err("docker push requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker push wrapper is defined (explicit release gate)",
                    "rows": [{"action":"push","repo_root": repo_root.display().to_string()}],
                    "capabilities": {"subprocess": args.common.allow_subprocess, "fs_write": args.common.allow_write},
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
                let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker release wrapper is defined (explicit release gate)",
                    "rows": [{"action":"release","repo_root": repo_root.display().to_string()}],
                    "capabilities": {"subprocess": args.common.allow_subprocess, "fs_write": args.common.allow_write},
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
                    println!("{rendered}");
                } else {
                    eprintln!("{rendered}");
                }
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas docker failed: {err}");
            1
        }
    }
}

pub(crate) fn run_build_command(quiet: bool, command: BuildCommand) -> i32 {
    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        match command {
            BuildCommand::Bin(common) => build_emit(
                &common,
                started,
                "bin",
                "build bin wrapper is defined (artifacts/bin)",
                vec!["bijux", "bijux-atlas", "bijux-dev-atlas"],
            ),
            BuildCommand::Dist(common) => build_emit(
                &common,
                started,
                "dist",
                "build dist wrapper is defined (artifacts/dist)",
                vec!["bijux-atlas", "bijux-dev-atlas"],
            ),
            BuildCommand::Doctor(common) => build_emit(
                &common,
                started,
                "doctor",
                "build doctor wrapper is defined",
                Vec::new(),
            ),
            BuildCommand::Clean(args) => run_build_clean(args, started),
        }
    })();
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                println!("{rendered}");
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas build failed: {err}");
            1
        }
    }
}

fn build_emit(
    common: &BuildCommonArgs,
    started: std::time::Instant,
    action: &str,
    text: &str,
    binaries: Vec<&str>,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = common
        .run_id
        .as_ref()
        .map(|v| RunId::parse(v))
        .transpose()?
        .unwrap_or_else(|| RunId::from_seed(&format!("build_{action}")));
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": action,
        "text": text,
        "repo_root": repo_root.display().to_string(),
        "run_id": run_id.as_str(),
        "artifacts": {
            "bin_dir": repo_root.join("artifacts/bin").display().to_string(),
            "dist_dir": repo_root.join("artifacts/dist").display().to_string(),
            "build_dir": repo_root.join("artifacts/build/cargo").display().to_string()
        },
        "binaries": binaries,
        "capabilities": {
            "subprocess": common.allow_subprocess,
            "fs_write": common.allow_write
        },
        "duration_ms": started.elapsed().as_millis() as u64
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_build_clean(
    args: BuildCleanArgs,
    started: std::time::Instant,
) -> Result<(String, i32), String> {
    if !args.common.allow_write {
        return Err("build clean requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let mut removed = vec![repo_root.join("artifacts/build/cargo")];
    if args.include_bin {
        removed.push(repo_root.join("artifacts/bin"));
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "clean",
        "text": "build clean wrapper is defined (scoped artifacts only)",
        "repo_root": repo_root.display().to_string(),
        "removed_paths": removed.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        "capabilities": {
            "subprocess": args.common.allow_subprocess,
            "fs_write": args.common.allow_write
        },
        "duration_ms": started.elapsed().as_millis() as u64
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
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
