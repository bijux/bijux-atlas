use crate::cli::{
    BuildCleanArgs, BuildCommand, BuildCommonArgs, DockerCommand, DockerCommonArgs,
    DockerPolicyCommand, PoliciesCommand,
};
use crate::*;
use bijux_dev_atlas_model::CONTRACT_SCHEMA_VERSION;
use bijux_dev_atlas_policies::{canonical_policy_json, DevAtlasPolicySet};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

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
        let mut paths = entries.filter_map(Result::ok).map(|e| e.path()).collect::<Vec<_>>();
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
    doc: &bijux_dev_atlas_policies::DevAtlasPolicySetDocument,
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
    let run: Result<(String, i32), String> = {
        let started = std::time::Instant::now();
        match command {
            BuildCommand::Bin(common) => run_build_bin(&common, started),
            BuildCommand::Plan(common) => run_build_plan(&common, started),
            BuildCommand::Verify(common) => run_build_verify(&common, started),
            BuildCommand::Meta(common) => run_build_meta(&common, started),
            BuildCommand::Dist(common) => run_build_dist(&common, started),
            BuildCommand::Doctor(common) => run_build_doctor(&common, started),
            BuildCommand::Clean(args) => run_build_clean(args, started),
            BuildCommand::InstallLocal(common) => run_build_bin(&common, started),
        }
    };
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

fn run_build_bin(
    common: &BuildCommonArgs,
    started: std::time::Instant,
) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("build bin requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("build bin requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = common
        .run_id
        .as_ref()
        .map(|v| RunId::parse(v))
        .transpose()?
        .unwrap_or_else(|| RunId::from_seed("build_bin"));
    let cargo_target_dir = repo_root.join("artifacts/build/cargo/build");
    let artifacts_bin_dir = repo_root.join("artifacts/dist/bin");
    fs::create_dir_all(&cargo_target_dir).map_err(|e| {
        format!(
            "cannot create cargo target dir {}: {e}",
            cargo_target_dir.display()
        )
    })?;
    fs::create_dir_all(&artifacts_bin_dir)
        .map_err(|e| format!("cannot create {}: {e}", artifacts_bin_dir.display()))?;

    let binary_specs = [
        ("bijux-atlas-cli", "bijux-atlas"),
        ("bijux-dev-atlas", "bijux-dev-atlas"),
    ];
    let mut built_rows = Vec::new();
    for (package, bin_name) in binary_specs {
        let status = ProcessCommand::new("cargo")
            .current_dir(&repo_root)
            .env("CARGO_TARGET_DIR", &cargo_target_dir)
            .args(["build", "-q", "-p", package, "--bin", bin_name])
            .status()
            .map_err(|e| format!("failed to run cargo build for {bin_name}: {e}"))?;
        if !status.success() {
            return Err(format!(
                "cargo build failed for {bin_name} (package {package})"
            ));
        }
        let src = cargo_target_dir
            .join("debug")
            .join(binary_with_ext(bin_name));
        let dest = artifacts_bin_dir.join(binary_with_ext(bin_name));
        fs::copy(&src, &dest).map_err(|e| {
            format!(
                "cannot copy built binary {} -> {}: {e}",
                src.display(),
                dest.display()
            )
        })?;
        built_rows.push(serde_json::json!({
            "package": package,
            "bin": bin_name,
            "source": src.display().to_string(),
            "path": dest.display().to_string()
        }));
    }
    let manifest_path = artifacts_bin_dir.join("manifest.json");
    let manifest = serde_json::json!({
        "schema_version": 1,
        "kind": "build_bin_manifest",
        "version": env!("CARGO_PKG_VERSION"),
        "git_hash": option_env!("BIJUX_GIT_HASH"),
        "profile": "debug",
        "cargo_target_dir": cargo_target_dir.display().to_string(),
        "binaries": built_rows,
        "run_id": run_id.as_str()
    });
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| format!("cannot write {}: {e}", manifest_path.display()))?;

    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "bin",
        "text": "built binaries and wrote artifacts/dist/bin/manifest.json",
        "repo_root": repo_root.display().to_string(),
        "run_id": run_id.as_str(),
        "artifacts": {
            "bin_dir": artifacts_bin_dir.display().to_string(),
            "manifest": manifest_path.display().to_string(),
            "cargo_target_dir": cargo_target_dir.display().to_string()
        },
        "rows": manifest.get("binaries").cloned().unwrap_or_else(|| serde_json::json!([])),
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
    let mut removed = Vec::new();
    let build_dir = repo_root.join("artifacts/build/cargo");
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)
            .map_err(|e| format!("cannot remove {}: {e}", build_dir.display()))?;
        removed.push(build_dir);
    }
    let dist_bin_dir = repo_root.join("artifacts/dist/bin");
    if dist_bin_dir.exists() {
        fs::remove_dir_all(&dist_bin_dir)
            .map_err(|e| format!("cannot remove {}: {e}", dist_bin_dir.display()))?;
        removed.push(dist_bin_dir);
    }
    let dist_release_dir = repo_root.join("artifacts/dist/release");
    if dist_release_dir.exists() {
        fs::remove_dir_all(&dist_release_dir)
            .map_err(|e| format!("cannot remove {}: {e}", dist_release_dir.display()))?;
        removed.push(dist_release_dir);
    }
    let dist_meta = repo_root.join("artifacts/dist/build.json");
    if dist_meta.exists() {
        fs::remove_file(&dist_meta).map_err(|e| format!("cannot remove {}: {e}", dist_meta.display()))?;
        removed.push(dist_meta);
    }
    let checksum = repo_root.join("artifacts/dist/sha256sum.txt");
    if checksum.exists() {
        fs::remove_file(&checksum).map_err(|e| format!("cannot remove {}: {e}", checksum.display()))?;
        removed.push(checksum);
    }
    if args.include_bin {
        let legacy_bin_dir = repo_root.join("artifacts/bin");
        if legacy_bin_dir.exists() {
            fs::remove_dir_all(&legacy_bin_dir)
                .map_err(|e| format!("cannot remove {}: {e}", legacy_bin_dir.display()))?;
            removed.push(legacy_bin_dir);
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "clean",
        "text": "build clean removed scoped dist and build artifacts",
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

fn binary_with_ext(name: &str) -> String {
    #[cfg(windows)]
    {
        format!("{name}.exe")
    }
    #[cfg(not(windows))]
    {
        name.to_string()
    }
}

fn run_build_dist(
    common: &BuildCommonArgs,
    started: std::time::Instant,
) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("build dist requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("build dist requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let artifacts_bin = repo_root.join("artifacts/dist/bin");
    let bin_manifest = artifacts_bin.join("manifest.json");
    if !bin_manifest.exists() {
        let nested = BuildCommonArgs {
            repo_root: Some(repo_root.clone()),
            format: FormatArg::Json,
            out: None,
            run_id: common.run_id.clone(),
            allow_write: true,
            allow_subprocess: true,
        };
        let _ = run_build_bin(&nested, std::time::Instant::now())?;
    }
    let dist_dir = repo_root.join("artifacts/dist/release");
    fs::create_dir_all(&dist_dir)
        .map_err(|e| format!("cannot create {}: {e}", dist_dir.display()))?;
    let archive_name = format!(
        "bijux-atlas-dev-tools_{}_{}_{}.tar.gz",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let archive_path = dist_dir.join(archive_name);
    let status = ProcessCommand::new("tar")
        .current_dir(&repo_root)
        .args([
            "-czf",
            archive_path.to_string_lossy().as_ref(),
            "artifacts/dist/bin",
            "README.md",
        ])
        .status()
        .map_err(|e| format!("failed to run tar for dist bundle: {e}"))?;
    if !status.success() {
        return Err("tar failed while creating build dist bundle".to_string());
    }
    let bytes = fs::read(&archive_path)
        .map_err(|e| format!("cannot read {}: {e}", archive_path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let checksum = format!("{:x}", hasher.finalize());
    let checksum_path = dist_dir.join("sha256sum.txt");
    let checksum_line = format!(
        "{}  {}\n",
        checksum,
        archive_path.file_name().unwrap().to_string_lossy()
    );
    fs::write(&checksum_path, checksum_line)
        .map_err(|e| format!("cannot write {}: {e}", checksum_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "dist",
        "text": "created release bundle under artifacts/dist/release",
        "repo_root": repo_root.display().to_string(),
        "archive": archive_path.display().to_string(),
        "sha256sum": checksum_path.display().to_string(),
        "checksum": checksum,
        "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "duration_ms": started.elapsed().as_millis() as u64
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_build_doctor(
    common: &BuildCommonArgs,
    started: std::time::Instant,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let mut rows = Vec::new();
    for tool in ["cargo", "tar"] {
        let found = ProcessCommand::new("sh")
            .arg("-c")
            .arg(format!("command -v {tool} >/dev/null 2>&1"))
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        rows.push(serde_json::json!({"kind":"tool","name":tool,"found":found}));
    }
    rows.push(serde_json::json!({"kind":"path","name":"artifacts_dist_bin","path": repo_root.join("artifacts/dist/bin").display().to_string()}));
    rows.push(serde_json::json!({"kind":"path","name":"artifacts_dist","path": repo_root.join("artifacts/dist").display().to_string()}));
    let errors = rows
        .iter()
        .filter(|row| {
            row.get("kind").and_then(|v| v.as_str()) == Some("tool")
                && row.get("found").and_then(|v| v.as_bool()) == Some(false)
        })
        .count();
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "doctor",
        "status": if errors == 0 { "ok" } else { "failed" },
        "text": "build doctor toolchain checks",
        "rows": rows,
        "summary": {"total": 4, "errors": errors, "warnings": 0},
        "capabilities": {"subprocess": true, "fs_write": false},
        "duration_ms": started.elapsed().as_millis() as u64
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if errors == 0 { 0 } else { 1 }))
}

fn build_binary_specs() -> [(&'static str, &'static str); 2] {
    [("bijux-atlas-cli", "bijux-atlas"), ("bijux-dev-atlas", "bijux-dev-atlas")]
}

fn run_build_plan(
    common: &BuildCommonArgs,
    started: std::time::Instant,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let cargo_target_dir = repo_root.join("artifacts/build/cargo/build");
    let dist_bin_dir = repo_root.join("artifacts/dist/bin");
    let rows = build_binary_specs()
        .into_iter()
        .map(|(package, bin_name)| {
            serde_json::json!({
                "package": package,
                "bin": bin_name,
                "cargo_target_dir": cargo_target_dir.display().to_string(),
                "output": dist_bin_dir.join(binary_with_ext(bin_name)).display().to_string()
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "plan",
        "text": "build plan outputs under artifacts/dist/bin",
        "repo_root": repo_root.display().to_string(),
        "rows": rows,
        "outputs": {
            "bin_dir": dist_bin_dir.display().to_string(),
            "dist_dir": repo_root.join("artifacts/dist").display().to_string()
        },
        "capabilities": {"subprocess": false, "fs_write": false},
        "duration_ms": started.elapsed().as_millis() as u64
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_build_verify(
    common: &BuildCommonArgs,
    started: std::time::Instant,
) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("build verify requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let dist_bin_dir = repo_root.join("artifacts/dist/bin");
    let mut rows = Vec::new();
    let mut errors = 0usize;
    for (_, bin_name) in build_binary_specs() {
        let path = dist_bin_dir.join(binary_with_ext(bin_name));
        let exists = path.exists();
        let version_ok = if exists {
            ProcessCommand::new(&path)
                .arg("--version")
                .current_dir(&repo_root)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            false
        };
        if !(exists && version_ok) {
            errors += 1;
        }
        rows.push(serde_json::json!({
            "bin": bin_name,
            "path": path.display().to_string(),
            "exists": exists,
            "version_ok": version_ok
        }));
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "verify",
        "status": if errors == 0 { "ok" } else { "failed" },
        "text": "build outputs verification under artifacts/dist/bin",
        "repo_root": repo_root.display().to_string(),
        "rows": rows,
        "summary": {"total": build_binary_specs().len(), "errors": errors, "warnings": 0},
        "capabilities": {"subprocess": common.allow_subprocess, "fs_write": false},
        "duration_ms": started.elapsed().as_millis() as u64
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if errors == 0 { 0 } else { 1 }))
}

fn run_build_meta(
    common: &BuildCommonArgs,
    started: std::time::Instant,
) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("build meta requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let dist_dir = repo_root.join("artifacts/dist");
    fs::create_dir_all(&dist_dir)
        .map_err(|e| format!("cannot create {}: {e}", dist_dir.display()))?;
    let meta_path = dist_dir.join("build.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "build_metadata",
        "version": env!("CARGO_PKG_VERSION"),
        "git_hash": option_env!("BIJUX_GIT_HASH"),
        "timestamp_policy": "forbidden_by_default",
        "toolchain_pin_file": "rust-toolchain.toml",
        "outputs": {
            "bin_dir": repo_root.join("artifacts/dist/bin").display().to_string(),
            "dist_dir": dist_dir.display().to_string()
        }
    });
    fs::write(
        &meta_path,
        serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| format!("cannot write {}: {e}", meta_path.display()))?;
    let response = serde_json::json!({
        "schema_version": 1,
        "action": "meta",
        "text": "build metadata written under artifacts/dist/build.json",
        "path": meta_path.display().to_string(),
        "metadata": payload,
        "capabilities": {"subprocess": false, "fs_write": true},
        "duration_ms": started.elapsed().as_millis() as u64
    });
    let rendered = emit_payload(common.format, common.out.clone(), &response)?;
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
