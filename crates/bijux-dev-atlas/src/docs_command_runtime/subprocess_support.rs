fn docs_build_or_serve_subprocess(
    args: &[String],
    common: &DocsCommonArgs,
    label: &str,
) -> Result<(serde_json::Value, i32), String> {
    if !common.allow_subprocess {
        return Err(format!("{label} requires --allow-subprocess"));
    }
    if label == "docs build" && !common.allow_write {
        return Err("docs build requires --allow-write".to_string());
    }
    let ctx = docs_context(common)?;
    let output_dir = ctx
        .artifacts_root
        .join("dist")
        .join("docs-site")
        .join(ctx.run_id.as_str());
    if label == "docs build" {
        fs::create_dir_all(&output_dir)
            .map_err(|e| format!("failed to create {}: {e}", output_dir.display()))?;
    }
    let mut cmd = ProcessCommand::new("mkdocs");
    cmd.args(args).current_dir(&ctx.repo_root);
    if label == "docs build" {
        cmd.args([
            "--site-dir",
            output_dir.to_str().unwrap_or("artifacts/dist/docs-site"),
        ]);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("failed to run mkdocs: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let code = out.status.code().unwrap_or(1);
    let mut files = Vec::<serde_json::Value>::new();
    if label == "docs build" && output_dir.exists() {
        for path in walk_files_local(&output_dir) {
            let Ok(bytes) = fs::read(&path) else { continue };
            let rel = path
                .strip_prefix(&output_dir)
                .unwrap_or(&path)
                .display()
                .to_string();
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            files.push(serde_json::json!({
                "path": rel,
                "sha256": format!("{:x}", hasher.finalize()),
                "bytes": bytes.len()
            }));
        }
        files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
        let index_path = ctx
            .artifacts_root
            .join("dist")
            .join("docs-site")
            .join(ctx.run_id.as_str())
            .join("build.index.json");
        if common.allow_write {
            if let Some(parent) = index_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(
                &index_path,
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": 1,
                    "run_id": ctx.run_id.as_str(),
                    "files": files
                }))
                .unwrap_or_default(),
            );
        }
    }
    Ok((
        serde_json::json!({
            "schema_version":1,
            "run_id": ctx.run_id.as_str(),
            "error_code": if code == 0 { serde_json::Value::Null } else { serde_json::Value::String("DOCS_BUILD_ERROR".to_string()) },
            "text": format!("{label} {}", if code==0 {"ok"} else {"failed"}),
            "rows":[{"command": args, "exit_code": code, "stdout": stdout, "stderr": stderr, "site_dir": output_dir.display().to_string()}],
            "artifacts": {"site_dir": output_dir.display().to_string(), "build_index": ctx.artifacts_root.join("dist").join("docs-site").join(ctx.run_id.as_str()).join("build.index.json").display().to_string(), "files": files},
            "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
            "options": {"strict": common.strict, "include_drafts": common.include_drafts}
        }),
        code,
    ))
}

