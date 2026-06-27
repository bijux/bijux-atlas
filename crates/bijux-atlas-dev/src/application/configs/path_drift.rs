// SPDX-License-Identifier: Apache-2.0

use crate::*;

fn extract_copy_sources(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with("COPY ") {
        return None;
    }
    let rest = trimmed.strip_prefix("COPY ")?.trim();
    if rest.starts_with('[') {
        let parsed: serde_json::Value = serde_json::from_str(rest).ok()?;
        let arr = parsed.as_array()?;
        if arr.len() < 2 {
            return None;
        }
        return Some(
            arr[..arr.len() - 1]
                .iter()
                .filter_map(|v| v.as_str())
                .map(str::to_string)
                .collect::<Vec<_>>(),
        );
    }
    let parts = rest.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    let mut filtered = Vec::<&str>::new();
    for part in parts {
        if filtered.is_empty() && part.starts_with("--") {
            continue;
        }
        filtered.push(part);
    }
    if filtered.len() < 2 {
        return None;
    }
    Some(
        filtered[..filtered.len() - 1]
            .iter()
            .map(|v| (*v).to_string())
            .collect::<Vec<_>>(),
    )
}

fn dockerfile_paths_under(root: &Path) -> Vec<PathBuf> {
    walk_files_local(root)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|name| name.starts_with("Dockerfile"))
        })
        .collect()
}

fn validate_root_rustfmt_mirror(
    ctx: &ConfigsContext,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let root_rustfmt = ctx.repo_root.join("rustfmt.toml");
    let governed_rustfmt = ctx
        .repo_root
        .join("configs/sources/repository/rust-tooling/rustfmt.toml");

    if !root_rustfmt.exists() {
        errors.push(
            "CONFIGS_DRIFT_ERROR: root rustfmt.toml is required so plain cargo fmt matches make fmt"
                .to_string(),
        );
        return Ok(());
    }

    if !governed_rustfmt.exists() {
        return Ok(());
    }

    let root_text = fs::read_to_string(&root_rustfmt)
        .map_err(|e| format!("failed to read {}: {e}", root_rustfmt.display()))?;
    let governed_text = fs::read_to_string(&governed_rustfmt)
        .map_err(|e| format!("failed to read {}: {e}", governed_rustfmt.display()))?;

    if root_text != governed_text {
        errors.push(
            "CONFIGS_DRIFT_ERROR: rustfmt.toml must mirror configs/sources/repository/rust-tooling/rustfmt.toml so cargo fmt and make fmt stay equivalent".to_string(),
        );
    }

    Ok(())
}

fn config_reference_path_errors(ctx: &ConfigsContext) -> Result<Vec<String>, String> {
    let mut errors = Vec::<String>::new();
    let forbidden_root_refs = [
        "rustfmt.toml",
        "clippy.toml",
        "deny.toml",
        "/rustfmt.toml",
        "/clippy.toml",
        "/deny.toml",
    ];

    for dockerfile in dockerfile_paths_under(&ctx.repo_root.join("docker")) {
        let rel = dockerfile
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&dockerfile)
            .display()
            .to_string();
        let text = fs::read_to_string(&dockerfile)
            .map_err(|e| format!("failed to read {}: {e}", dockerfile.display()))?;
        for (line_idx, line) in text.lines().enumerate() {
            if let Some(sources) = extract_copy_sources(line) {
                for src in sources {
                    if src == "." || src.starts_with('/') {
                        continue;
                    }
                    if src.ends_with('/') {
                        if !ctx.repo_root.join(src.trim_end_matches('/')).exists() {
                            errors.push(format!(
                                "CONFIGS_DRIFT_ERROR: {rel}:{} COPY source path missing `{}`",
                                line_idx + 1,
                                src
                            ));
                        }
                        continue;
                    }
                    if !ctx.repo_root.join(&src).exists() {
                        errors.push(format!(
                            "CONFIGS_DRIFT_ERROR: {rel}:{} COPY source path missing `{}`",
                            line_idx + 1,
                            src
                        ));
                    }
                    if forbidden_root_refs.iter().any(|pat| pat == &src) {
                        errors.push(format!(
                            "CONFIGS_DRIFT_ERROR: {rel}:{} COPY source must use configs/** not `{}`",
                            line_idx + 1,
                            src
                        ));
                    }
                }
            }
        }
    }

    for root_path in ["clippy.toml", "deny.toml", "audit.toml"] {
        if ctx.repo_root.join(root_path).exists() {
            errors.push(format!(
                "CONFIGS_DRIFT_ERROR: root shim `{root_path}` is forbidden; use configs/** SSOT only"
            ));
        }
    }
    validate_root_rustfmt_mirror(ctx, &mut errors)?;

    let mut check_text_ref = |path: &Path| -> Result<(), String> {
        let rel = path
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        let text = fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        for (idx, line) in text.lines().enumerate() {
            if forbidden_root_refs.iter().any(|pat| line.contains(pat))
                && !(line.contains("configs/sources/repository/rust-tooling/rustfmt.toml")
                    || line.contains("configs/rust/clippy.toml")
                    || line.contains("configs/rust/deny.toml"))
            {
                errors.push(format!(
                    "CONFIGS_DRIFT_ERROR: {rel}:{} config reference must be under configs/**",
                    idx + 1
                ));
            }
        }
        Ok(())
    };

    for workflow in walk_files_local(&ctx.repo_root.join(".github/workflows"))
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("yml"))
    {
        check_text_ref(&workflow)?;
    }
    for mk in walk_files_local(&ctx.repo_root.join("makefiles"))
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("mk"))
    {
        check_text_ref(&mk)?;
    }

    errors.sort();
    errors.dedup();
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_context(repo_root: PathBuf) -> ConfigsContext {
        ConfigsContext {
            configs_root: repo_root.join("configs"),
            repo_root,
            artifacts_root: PathBuf::from("artifacts"),
            run_id: RunId::from_seed("configs_path_drift_test"),
        }
    }

    #[test]
    fn root_rustfmt_mirror_accepts_matching_files() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().to_path_buf();
        let governed = repo_root.join("configs/sources/repository/rust-tooling/rustfmt.toml");
        fs::create_dir_all(governed.parent().expect("governed parent")).expect("mkdirs");
        let contents = "edition = \"2021\"\nnewline_style = \"Unix\"\n";
        fs::write(repo_root.join("rustfmt.toml"), contents).expect("write root");
        fs::write(&governed, contents).expect("write governed");

        let ctx = test_context(repo_root);
        let mut errors = Vec::new();
        validate_root_rustfmt_mirror(&ctx, &mut errors).expect("validate rustfmt");
        assert!(
            errors.is_empty(),
            "expected no drift errors, got {errors:?}"
        );
    }

    #[test]
    fn root_rustfmt_mirror_rejects_drift() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().to_path_buf();
        let governed = repo_root.join("configs/sources/repository/rust-tooling/rustfmt.toml");
        fs::create_dir_all(governed.parent().expect("governed parent")).expect("mkdirs");
        fs::write(repo_root.join("rustfmt.toml"), "edition = \"2021\"\n").expect("write root");
        fs::write(&governed, "edition = \"2021\"\nmax_width = 100\n").expect("write governed");

        let ctx = test_context(repo_root);
        let mut errors = Vec::new();
        validate_root_rustfmt_mirror(&ctx, &mut errors).expect("validate rustfmt");
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cargo fmt and make fmt stay equivalent")),
            "expected root rustfmt drift error, got {errors:?}"
        );
    }
}

pub(super) fn configs_verify_payload(
    ctx: &ConfigsContext,
    common: &ConfigsCommonArgs,
) -> Result<serde_json::Value, String> {
    let errors = config_reference_path_errors(ctx)?;
    let rows = serde_json::json!([
        {"name": "dockerfile_copy_path_drift", "status": if errors.is_empty() { "ok" } else { "failed" }},
        {"name": "ci_makefile_config_path_drift", "status": if errors.is_empty() { "ok" } else { "failed" }}
    ]);
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": if errors.is_empty() { "configs verify passed" } else { "configs verify failed" },
        "rows": rows,
        "errors": errors,
        "warnings": [],
        "error_code": if errors.is_empty() { serde_json::Value::Null } else { serde_json::Value::String("CONFIGS_DRIFT_ERROR".to_string()) },
        "capabilities": {"fs_write": common.allow_write, "subprocess": common.allow_subprocess, "network": common.allow_network},
        "options": {"strict": common.strict}
    }))
}

pub(super) fn configs_files(ctx: &ConfigsContext) -> Vec<PathBuf> {
    if !ctx.configs_root.exists() {
        return Vec::new();
    }
    walk_files_local(&ctx.configs_root)
}

pub(crate) fn configs_context(common: &ConfigsCommonArgs) -> Result<ConfigsContext, String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let artifacts_root = common
        .artifacts_root
        .clone()
        .unwrap_or_else(|| repo_root.join("artifacts"));
    let run_id = common
        .run_id
        .as_ref()
        .map(|v| RunId::parse(v))
        .transpose()?
        .unwrap_or_else(|| RunId::from_seed("configs_run"));
    Ok(ConfigsContext {
        configs_root: repo_root.join("configs"),
        repo_root,
        artifacts_root,
        run_id,
    })
}

pub(crate) fn parse_config_file(path: &Path) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    let text =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    match ext {
        "json" | "schema" => {
            serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
        }
        "toml" => {
            toml::from_str::<toml::Value>(&text)
                .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
        }
        "yaml" | "yml" => {
            serde_yaml::from_str::<serde_yaml::Value>(&text)
                .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
        }
        _ => {}
    }
    Ok(())
}
