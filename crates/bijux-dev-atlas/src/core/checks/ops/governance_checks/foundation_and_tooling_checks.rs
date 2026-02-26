// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn checks_ops_no_scripts_areas_or_xtask_refs(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let targets = [
        Path::new("makefiles/_ops.mk"),
        Path::new(".github/workflows/ci-pr.yml"),
        Path::new("ops/README.md"),
        Path::new("ops/INDEX.md"),
    ];
    let needles = ["scripts/areas", "xtask"];
    let mut violations = Vec::new();
    for rel in targets {
        let path = ctx.repo_root.join(rel);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('!') && trimmed.contains("rg -n") {
                continue;
            }
            for needle in needles {
                if trimmed.contains(needle) {
                    violations.push(violation(
                        "OPS_LEGACY_REFERENCE_FOUND",
                        format!(
                            "forbidden retired reference `{needle}` found in {}: `{trimmed}`",
                            rel.display()
                        ),
                        "remove scripts/areas and xtask references from ops-owned surfaces",
                        Some(rel),
                    ));
                }
            }
        }
    }
    let canonical_docs = [
        Path::new("ops/CONTRACT.md"),
        Path::new("ops/INDEX.md"),
        Path::new("ops/README.md"),
        Path::new("docs/operations/ops-system/INDEX.md"),
    ];
    for rel in canonical_docs {
        let path = ctx.repo_root.join(rel);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        if content.contains("ops/obs/") {
            violations.push(violation(
                "OPS_LEGACY_REFERENCE_FOUND",
                format!(
                    "legacy observability path `ops/obs/` found in canonical document {}",
                    rel.display()
                ),
                "use canonical `ops/observe/` path and keep migration notes in dedicated migration docs only",
                Some(rel),
            ));
        }
    }
    let ops_docs_root = ctx.repo_root.join("ops");
    if ops_docs_root.exists() {
        for file in walk_files(&ops_docs_root) {
            if file.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            if content.contains("ops/schema/obs/") || content.contains("ops/obs/") {
                violations.push(violation(
                    "OPS_LEGACY_REFERENCE_FOUND",
                    format!(
                        "retired observability path reference found in {}",
                        rel.display()
                    ),
                    "replace legacy observability paths with canonical ops/observe paths",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

pub(super) fn checks_ops_artifacts_gitignore_policy(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".gitignore");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    if content
        .lines()
        .any(|line| line.trim() == "artifacts/" || line.trim() == "/artifacts/")
    {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "OPS_ARTIFACTS_GITIGNORE_MISSING",
            "artifacts/ must be ignored in .gitignore".to_string(),
            "add `artifacts/` to .gitignore",
            Some(rel),
        )])
    }
}

pub(super) fn checks_ops_workflow_routes_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let workflows_root = ctx.repo_root.join(".github/workflows");
    if !workflows_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&workflows_root) {
        if file.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        if !text.contains("RUN_ID:") {
            if text.contains("ISO_ROOT: artifacts/isolates/") {
                for required_tmp in ["TMPDIR:", "TMP:", "TEMP:"] {
                    if !text.contains(required_tmp) {
                        violations.push(violation(
                            "WORKFLOW_ISOLATION_TEMP_ENV_MISSING",
                            format!(
                                "workflow `{}` defines ISO_ROOT isolation but is missing `{required_tmp}` temp environment binding",
                                rel.display()
                            ),
                            "bind TMPDIR, TMP, and TEMP under the workflow isolate tmp directory",
                            Some(rel),
                        ));
                    }
                }
            }
            continue;
        }
        if !text.contains("github.run_attempt") {
            violations.push(violation(
                "WORKFLOW_RUN_ID_ATTEMPT_SUFFIX_MISSING",
                format!(
                    "workflow `{}` RUN_ID must include github.run_attempt for retry-safe isolation",
                    rel.display()
                ),
                "append `${{ github.run_attempt }}` to workflow RUN_ID definitions",
                Some(rel),
            ));
        }
        if !text.contains("ISO_ROOT: artifacts/isolates/") {
            violations.push(violation(
                "WORKFLOW_ARTIFACT_ISOLATION_ROOT_MISSING",
                format!(
                    "workflow `{}` defines RUN_ID but is missing ISO_ROOT under artifacts/isolates/",
                    rel.display()
                ),
                "declare ISO_ROOT under artifacts/isolates/<lane> for workflows that emit run-scoped artifacts",
                Some(rel),
            ));
        }
        for required_tmp in ["TMPDIR:", "TMP:", "TEMP:"] {
            if !text.contains(required_tmp) {
                violations.push(violation(
                    "WORKFLOW_ISOLATION_TEMP_ENV_MISSING",
                    format!(
                        "workflow `{}` is missing `{required_tmp}` temp environment binding",
                        rel.display()
                    ),
                    "bind TMPDIR, TMP, and TEMP under the workflow isolate tmp directory",
                    Some(rel),
                ));
            }
        }
        if text.contains("TMPDIR:") && !text.contains("TMPDIR: artifacts/isolates/") {
            violations.push(violation(
                "WORKFLOW_ISOLATION_TMPDIR_LAYOUT_MISSING",
                format!(
                    "workflow `{}` must bind TMPDIR under its isolate tmp path",
                    rel.display()
                ),
                "set TMPDIR/TMP/TEMP to artifacts/isolates/<lane>/tmp",
                Some(rel),
            ));
        }
        if !text.contains("artifacts/${RUN_ID}/") {
            violations.push(violation(
                "WORKFLOW_RUN_ID_ARTIFACT_LAYOUT_MISSING",
                format!(
                    "workflow `{}` defines RUN_ID but does not write summary/log/report paths under artifacts/${{RUN_ID}}/",
                    rel.display()
                ),
                "write workflow reports and logs under artifacts/${RUN_ID}/...",
                Some(rel),
            ));
        }
        if !text.contains("rm -rf \"artifacts/${RUN_ID}\"")
            || !text.contains("mkdir -p \"artifacts/${RUN_ID}\"")
        {
            violations.push(violation(
                "WORKFLOW_RUN_ID_ARTIFACT_CLEANUP_MISSING",
                format!(
                    "workflow `{}` must clean and recreate artifacts/${{RUN_ID}} before lane execution",
                    rel.display()
                ),
                "add a shell step that runs `rm -rf \"artifacts/${RUN_ID}\"` and `mkdir -p \"artifacts/${RUN_ID}\"`",
                Some(rel),
            ));
        }
        if text.contains("actions/upload-artifact@")
            && !text.contains("path: artifacts/${{ env.RUN_ID }}")
        {
            violations.push(violation(
                "WORKFLOW_ARTIFACT_UPLOAD_PATH_NOT_RUN_SCOPED",
                format!(
                    "workflow `{}` uploads artifacts but not from path artifacts/${{ env.RUN_ID }}",
                    rel.display()
                ),
                "upload the run-scoped artifact directory path artifacts/${{ env.RUN_ID }}",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn checks_ops_workflows_github_actions_pinned(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let toolchain_rel = Path::new("ops/inventory/toolchain.json");
    let toolchain_path = ctx.repo_root.join(toolchain_rel);
    let toolchain_text = fs::read_to_string(&toolchain_path)
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", toolchain_rel.display())))?;
    let toolchain_json: serde_json::Value = serde_json::from_str(&toolchain_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", toolchain_rel.display())))?;
    let Some(actions_obj) = toolchain_json
        .get("github_actions")
        .and_then(|v| v.as_object())
    else {
        return Ok(vec![violation(
            "OPS_TOOLCHAIN_ACTIONS_PIN_SET_MISSING",
            "ops/inventory/toolchain.json is missing github_actions pin set".to_string(),
            "declare workflow action refs and immutable SHAs under github_actions",
            Some(toolchain_rel),
        )]);
    };

    let mut allowed_shas: BTreeMap<String, String> = BTreeMap::new();
    for (name, entry) in actions_obj {
        let Some(sha) = entry.get("sha").and_then(|v| v.as_str()) else {
            continue;
        };
        allowed_shas.insert(name.clone(), sha.to_string());
    }

    let workflows_root = ctx.repo_root.join(".github/workflows");
    if !workflows_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&workflows_root) {
        if file.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("- uses:") && !trimmed.starts_with("uses:") {
                continue;
            }
            let Some((_, spec_raw)) = trimmed.split_once(':') else {
                continue;
            };
            let spec = spec_raw.trim();
            if spec.starts_with("docker://") {
                continue;
            }
            let Some((action_path, sha)) = spec.rsplit_once('@') else {
                violations.push(violation(
                    "WORKFLOW_ACTION_PIN_MISSING",
                    format!(
                        "workflow `{}` line {} uses action `{spec}` without an immutable SHA",
                        rel.display(),
                        line_idx + 1
                    ),
                    "pin GitHub Actions to full 40-hex commit SHAs",
                    Some(rel),
                ));
                continue;
            };
            if sha.len() != 40 || !sha.chars().all(|ch| ch.is_ascii_hexdigit()) {
                violations.push(violation(
                    "WORKFLOW_ACTION_NOT_SHA_PINNED",
                    format!(
                        "workflow `{}` line {} action `{action_path}` is pinned to `{sha}`, not a 40-hex SHA",
                        rel.display(),
                        line_idx + 1
                    ),
                    "replace action version tags with immutable commit SHAs",
                    Some(rel),
                ));
                continue;
            }
            match allowed_shas.get(action_path) {
                Some(expected_sha) if expected_sha == sha => {}
                Some(expected_sha) => violations.push(violation(
                    "WORKFLOW_ACTION_SHA_MISMATCH",
                    format!(
                        "workflow `{}` line {} action `{action_path}` sha `{sha}` does not match ops/inventory/toolchain.json expected `{expected_sha}`",
                        rel.display(),
                        line_idx + 1
                    ),
                    "sync workflow action pin to the canonical toolchain github_actions pin set",
                    Some(rel),
                )),
                None => violations.push(violation(
                    "WORKFLOW_ACTION_NOT_ALLOWLISTED",
                    format!(
                        "workflow `{}` line {} action `{action_path}` is not declared in ops/inventory/toolchain.json github_actions",
                        rel.display(),
                        line_idx + 1
                    ),
                    "declare every workflow action in ops/inventory/toolchain.json github_actions before use",
                    Some(rel),
                )),
            }
        }
    }
    Ok(violations)
}

pub(super) fn checks_ops_image_references_digest_pinned(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();

    let toolchain_rel = Path::new("ops/inventory/toolchain.json");
    let toolchain_path = ctx.repo_root.join(toolchain_rel);
    if let Ok(text) = fs::read_to_string(&toolchain_path) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(images) = value.get("images").and_then(|v| v.as_object()) {
                for (name, image_ref) in images {
                    if name == "generated_by" {
                        continue;
                    }
                    let Some(image_ref) = image_ref.as_str() else {
                        continue;
                    };
                    if !image_ref.contains("@sha256:") {
                        violations.push(violation(
                            "OPS_IMAGE_TAG_ONLY_REFERENCE",
                            format!(
                                "ops/inventory/toolchain.json images.{name} is not digest pinned: `{image_ref}`"
                            ),
                            "pin all canonical ops image references to immutable digests",
                            Some(toolchain_rel),
                        ));
                    }
                }
            }
        }
    }

    let compose_rel = Path::new("ops/observe/pack/compose/docker-compose.yml");
    let compose_path = ctx.repo_root.join(compose_rel);
    if let Ok(text) = fs::read_to_string(&compose_path) {
        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("image: ") {
                continue;
            }
            let image_ref = trimmed.trim_start_matches("image: ").trim();
            if !image_ref.contains("@sha256:") {
                violations.push(violation(
                    "OPS_COMPOSE_IMAGE_TAG_ONLY_REFERENCE",
                    format!(
                        "{} line {} image is not digest pinned: `{image_ref}`",
                        compose_rel.display(),
                        line_idx + 1
                    ),
                    "pin compose images to immutable digests",
                    Some(compose_rel),
                ));
            }
        }
    }

    Ok(violations)
}

pub(super) fn check_ops_internal_registry_consistency(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let path = ctx.repo_root.join(crate::core::DEFAULT_REGISTRY_PATH);
    let output = ctx
        .adapters
        .process
        .run(
            "git",
            &[
                "status".to_string(),
                "--short".to_string(),
                path.display().to_string(),
            ],
            ctx.repo_root,
        )
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if output == 0 {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "OPS_INTERNAL_REGISTRY_GIT_STATUS_FAILED",
            "unable to query git status for registry".to_string(),
            "ensure git is available and repository is valid",
            Some(Path::new(crate::core::DEFAULT_REGISTRY_PATH)),
        )])
    }
}

pub(super) fn check_root_python_toolchain_toml_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("packages").join("python-toolchain.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, &rel) {
        Ok(vec![violation(
            "ROOT_PYTHON_TOOLCHAIN_TOML_PRESENT",
            "retired python toolchain SSOT file still exists".to_string(),
            "delete the retired python toolchain file after control-plane transition",
            Some(&rel),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_root_uv_lock_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("uv.lock");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(vec![violation(
            "ROOT_UV_LOCK_PRESENT",
            "retired root uv.lock exists".to_string(),
            "remove uv.lock if it is no longer required by repository tooling",
            Some(rel),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_crates_plugin_conformance_binaries(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let commands = [
        (
            "bijux-dev-atlas",
            vec![
                "run".to_string(),
                "-q".to_string(),
                "-p".to_string(),
                "bijux-dev-atlas".to_string(),
                "--".to_string(),
                "--bijux-plugin-metadata".to_string(),
            ],
        ),
        (
            "bijux-atlas-cli",
            vec![
                "run".to_string(),
                "-q".to_string(),
                "-p".to_string(),
                "bijux-atlas-cli".to_string(),
                "--".to_string(),
                "--bijux-plugin-metadata".to_string(),
            ],
        ),
    ];
    let mut violations = Vec::new();
    for (binary_name, args) in commands {
        let exit = ctx
            .adapters
            .process
            .run("cargo", &args, ctx.repo_root)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if exit != 0 {
            violations.push(violation(
                "PLUGIN_CONFORMANCE_SUBPROCESS_FAILED",
                format!("failed to run plugin metadata handshake for `{binary_name}`"),
                "ensure binary builds and supports --bijux-plugin-metadata JSON output",
                None,
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_artifacts_bin_binaries_executable_and_version_printable(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let dir = ctx.repo_root.join("artifacts/bin");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(vec![violation(
            "ARTIFACTS_BIN_READ_FAILED",
            format!("unable to read {}", dir.display()),
            "ensure artifacts/bin is readable",
            Some(Path::new("artifacts/bin")),
        )]);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path)
                .map_err(|err| CheckError::Failed(err.to_string()))?
                .permissions()
                .mode();
            if mode & 0o111 == 0 {
                violations.push(violation(
                    "ARTIFACTS_BIN_NOT_EXECUTABLE",
                    format!("binary is not executable: {}", path.display()),
                    "chmod +x generated binaries in artifacts/bin",
                    path.strip_prefix(ctx.repo_root)
                        .ok()
                        .map_or(Some(Path::new("artifacts/bin")), Some),
                ));
                continue;
            }
        }
        let path_str = path.display().to_string();
        let exit = ctx
            .adapters
            .process
            .run(&path_str, &["--version".to_string()], ctx.repo_root)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if exit != 0 {
            violations.push(violation(
                "ARTIFACTS_BIN_VERSION_FAILED",
                format!(
                    "binary did not print version successfully: {}",
                    path.display()
                ),
                "ensure copied binaries support `--version` and remain runnable",
                path.strip_prefix(ctx.repo_root).ok(),
            ));
        }
    }
    Ok(violations)
}
pub(super) fn check_workflows_no_direct_ops_script_execution(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_any_string_references_under(
        ctx,
        ".github/workflows",
        &["bash ops/", "sh ops/", "./ops/"],
        "WORKFLOW_DIRECT_OPS_SCRIPT_EXECUTION_FOUND",
    )
}
pub(super) fn check_make_no_direct_ops_script_execution(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_any_string_references_under(
        ctx,
        "makefiles",
        &["bash ops/", "sh ops/", "./ops/"],
        "MAKE_DIRECT_OPS_SCRIPT_EXECUTION_FOUND",
    )
}

pub(super) fn check_makefiles_no_cd_invocations(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_any_string_references_under(
        ctx,
        "makefiles",
        &["\tcd ", "; cd ", "&& cd "],
        "MAKEFILES_CD_INVOCATION_FOUND",
    )
}

pub(super) fn check_makefiles_no_direct_tool_invocations(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_any_string_references_under(
        ctx,
        "makefiles",
        &[
            "\tpython ",
            "\tpython3 ",
            "\tbash ",
            "\tsh ",
            "\tnode ",
            "\tkubectl ",
            "\thelm ",
            "\tk6 ",
        ],
        "MAKEFILES_DIRECT_TOOL_INVOCATION_FOUND",
    )
}

pub(super) fn check_makefiles_no_direct_fetch_commands(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_any_string_references_under(
        ctx,
        "makefiles",
        &["\tcurl ", "\twget "],
        "MAKEFILES_DIRECT_FETCH_COMMAND_FOUND",
    )
}

pub(super) fn check_makefiles_no_multiline_recipes(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let makefiles_root = ctx.repo_root.join("makefiles");
    if !makefiles_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&makefiles_root) {
        if file.extension().and_then(|e| e.to_str()) != Some("mk") {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let lines = text.lines().collect::<Vec<_>>();
        for idx in 0..lines.len() {
            let line = lines[idx];
            if !line.starts_with('\t') {
                continue;
            }
            if line.trim_start().starts_with('#') {
                continue;
            }
            if idx + 1 < lines.len() && lines[idx + 1].starts_with('\t') {
                violations.push(violation(
                    "MAKEFILES_MULTILINE_RECIPE_FOUND",
                    format!(
                        "multiline recipe detected in {} near line {}",
                        rel.display(),
                        idx + 1
                    ),
                    "keep wrapper targets to one recipe line; move logic into bijux dev atlas commands",
                    Some(rel),
                ));
                break;
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_root_dockerignore_context_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".dockerignore");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let required = [".git", "artifacts", "**/target"];
    let mut violations = Vec::new();
    for entry in required {
        if !text.lines().any(|line| line.trim() == entry) {
            violations.push(violation(
                "ROOT_DOCKERIGNORE_ENTRY_MISSING",
                format!(".dockerignore is missing required context exclusion `{entry}`"),
                "exclude repository noise and local build outputs from docker build context",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_root_dockerfile_pointer_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("Dockerfile");
    if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let non_comment_lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>();
    let looks_like_pointer = non_comment_lines.len() <= 3
        && non_comment_lines
            .iter()
            .any(|line| line.contains("docker/") || line.contains("see "));
    if looks_like_pointer {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "ROOT_DOCKERFILE_FORBIDDEN",
            "root Dockerfile must be absent or a tiny pointer to canonical docker/ definitions"
                .to_string(),
            "move real container build logic under docker/ and leave only a pointer doc if needed",
            Some(rel),
        )])
    }
}

pub(super) fn check_dockerfiles_under_canonical_directory_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for file in walk_files(ctx.repo_root) {
        let Some(name) = file.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name != "Dockerfile" {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        if rel == Path::new("Dockerfile") {
            continue;
        }
        if !rel.starts_with("docker/") {
            violations.push(violation(
                "DOCKERFILE_LOCATION_INVALID",
                format!(
                    "Dockerfile outside canonical docker/ directory: {}",
                    rel.display()
                ),
                "move Dockerfiles under docker/ or replace with pointer docs",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_workflows_no_direct_docker_build_execution(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_any_string_references_under(
        ctx,
        ".github/workflows",
        &["docker build ", " docker buildx ", "docker buildx build "],
        "WORKFLOW_DIRECT_DOCKER_BUILD_EXECUTION_FOUND",
    )
}

pub(super) fn check_ops_no_executable_bit_files(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let ops_root = ctx.repo_root.join("ops");
    if !ops_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&ops_root) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&file)
                .map_err(|err| CheckError::Failed(err.to_string()))?
                .permissions()
                .mode();
            if mode & 0o111 != 0 {
                let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
                violations.push(violation(
                    "OPS_EXECUTABLE_FILE_PRESENT",
                    format!("ops file has executable bit set: {}", rel.display()),
                    "ops/ stores data and contracts only; remove executable bits from committed files",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_ops_no_behavior_source_files(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let ops_root = ctx.repo_root.join("ops");
    if !ops_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    let allowlisted_prefixes = [
        Path::new("ops/datasets/fixtures"),
        Path::new("ops/e2e/fixtures"),
    ];
    for file in walk_files(&ops_root) {
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if ext != "sh" && ext != "py" {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        if allowlisted_prefixes
            .iter()
            .any(|prefix| rel.starts_with(prefix))
        {
            continue;
        }
        violations.push(violation(
            "OPS_BEHAVIOR_SOURCE_FILE_PRESENT",
            format!("ops contains behavior source file: {}", rel.display()),
            "move behavior into crates/bijux-dev-atlas; keep ops/ for manifests, schemas, and docs",
            Some(rel),
        ));
    }
    Ok(violations)
}

pub(super) fn check_ops_no_makefiles(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let ops_root = ctx.repo_root.join("ops");
    if !ops_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&ops_root) {
        let Some(name) = file.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name != "Makefile" {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        violations.push(violation(
            "OPS_MAKEFILE_FORBIDDEN",
            format!("ops must not contain Makefiles: {}", rel.display()),
            "remove Makefile from ops/ and delegate through makefiles/*.mk wrappers",
            Some(rel),
        ));
    }
    Ok(violations)
}

pub(super) fn check_ops_no_direct_tool_invocations(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let ops_root = ctx.repo_root.join("ops");
    if !ops_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    let patterns = ["kubectl ", "helm ", "k6 ", "kind "];
    for file in walk_files(&ops_root) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        let Some(name) = rel.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let is_behavior_surface = name == "Makefile"
            || rel
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ext == "mk" || ext == "sh" || ext == "bash" || ext == "py");
        if !is_behavior_surface {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        for pattern in patterns {
            if content.contains(pattern) {
                violations.push(violation(
                    "OPS_DIRECT_TOOL_INVOCATION_FORBIDDEN",
                    format!(
                        "ops behavior file contains direct tool invocation `{pattern}` in {}",
                        rel.display()
                    ),
                    "route tool execution through `bijux dev atlas ...` wrappers",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}
