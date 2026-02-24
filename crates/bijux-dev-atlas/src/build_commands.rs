// SPDX-License-Identifier: Apache-2.0

use crate::cli::{BuildCleanArgs, BuildCommand, BuildCommonArgs, FormatArg};
use crate::*;
use sha2::{Digest, Sha256};

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

    let mut built_rows = Vec::new();
    for (package, bin_name) in build_binary_specs() {
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
        fs::remove_file(&dist_meta)
            .map_err(|e| format!("cannot remove {}: {e}", dist_meta.display()))?;
        removed.push(dist_meta);
    }
    let checksum = repo_root.join("artifacts/dist/sha256sum.txt");
    if checksum.exists() {
        fs::remove_file(&checksum)
            .map_err(|e| format!("cannot remove {}: {e}", checksum.display()))?;
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
    [
        ("bijux-atlas-cli", "bijux-atlas"),
        ("bijux-dev-atlas", "bijux-dev-atlas"),
    ]
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
