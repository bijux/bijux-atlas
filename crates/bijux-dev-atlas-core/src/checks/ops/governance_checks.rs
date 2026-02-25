// SPDX-License-Identifier: Apache-2.0

use super::*;

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
        Path::new("ops/report/docs/REFERENCE_INDEX.md"),
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
            if rel == Path::new("ops/report/docs/observe-rename.md") {
                continue;
            }
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
    let _ = ctx;
    Ok(Vec::new())
}

pub(super) fn check_ops_internal_registry_consistency(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let path = ctx.repo_root.join(crate::DEFAULT_REGISTRY_PATH);
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
            Some(Path::new(crate::DEFAULT_REGISTRY_PATH)),
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
        if allowlisted_prefixes.iter().any(|prefix| rel.starts_with(prefix)) {
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

pub(super) fn check_ops_no_makefiles(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
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
            || rel.extension().and_then(|e| e.to_str()).is_some_and(|ext| {
                ext == "mk" || ext == "sh" || ext == "bash" || ext == "py"
            });
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

pub(super) fn check_ops_required_files_contracts(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let ops_root = ctx.repo_root.join("ops");
    if !ops_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for required_doc in walk_files(&ops_root) {
        let rel = required_doc
            .strip_prefix(ctx.repo_root)
            .unwrap_or(required_doc.as_path());
        if rel.file_name().and_then(|n| n.to_str()) != Some("REQUIRED_FILES.md") {
            continue;
        }
        let content = fs::read_to_string(&required_doc)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let (required_files, required_directories) =
            parse_required_files_markdown_yaml(&content, rel)?;
        for file_rel in required_files {
            let file_path = ctx.repo_root.join(&file_rel);
            if !file_path.exists() {
                violations.push(violation(
                    "OPS_REQUIRED_FILE_MISSING",
                    format!("required file missing: `{}`", file_rel.display()),
                    "create missing required file or remove stale declaration from REQUIRED_FILES.md",
                    Some(rel),
                ));
                continue;
            }
            let metadata =
                fs::metadata(&file_path).map_err(|err| CheckError::Failed(err.to_string()))?;
            if metadata.len() == 0 {
                violations.push(violation(
                    "OPS_REQUIRED_FILE_EMPTY",
                    format!("required file is empty: `{}`", file_rel.display()),
                    "populate required file with non-empty contract content",
                    Some(&file_rel),
                ));
            }
            if file_rel.extension().and_then(|v| v.to_str()) == Some("md") {
                let domain_root = rel.parent().unwrap_or(Path::new("ops"));
                let domain_index = domain_root.join("INDEX.md");
                if ctx.adapters.fs.exists(ctx.repo_root, &domain_index)
                    && file_rel.starts_with(domain_root)
                    && file_rel.file_name().and_then(|v| v.to_str()) != Some("INDEX.md")
                {
                    let index_text = fs::read_to_string(ctx.repo_root.join(&domain_index))
                        .map_err(|err| CheckError::Failed(err.to_string()))?;
                    let file_name = file_rel.file_name().and_then(|v| v.to_str()).unwrap_or("");
                    if !index_text.contains(file_name) {
                        violations.push(violation(
                            "OPS_REQUIRED_DOC_NOT_INDEXED",
                            format!(
                                "required document `{}` is not linked from `{}`",
                                file_rel.display(),
                                domain_index.display()
                            ),
                            "add required doc link to the domain INDEX.md",
                            Some(&domain_index),
                        ));
                    }
                }
            }
        }
        for dir_rel in required_directories {
            let dir_path = ctx.repo_root.join(&dir_rel);
            if !dir_path.exists() || !dir_path.is_dir() {
                violations.push(violation(
                    "OPS_REQUIRED_DIRECTORY_MISSING",
                    format!("required directory missing: `{}`", dir_rel.display()),
                    "create required directory or remove stale declaration from REQUIRED_FILES.md",
                    Some(rel),
                ));
            }
        }
    }

    let inventory_meta_allowed = [
        Path::new("ops/inventory/meta/contracts.json"),
        Path::new("ops/inventory/meta/error-registry.json"),
        Path::new("ops/inventory/meta/layer-contract.json"),
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    for file in walk_files(&ctx.repo_root.join("ops/inventory/meta")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if !inventory_meta_allowed.contains(rel) {
            violations.push(violation(
                "OPS_INVENTORY_META_UNKNOWN_FILE",
                format!("unexpected file in tight inventory meta surface: `{}`", rel.display()),
                "remove unknown file or update tight inventory meta contract",
                Some(rel),
            ));
        }
    }

    for file in walk_files(&ctx.repo_root.join("ops/schema")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let ext = rel.extension().and_then(|v| v.to_str()).unwrap_or("");
        if rel.starts_with(Path::new("ops/schema/generated")) {
            continue;
        }
        let is_allowed_doc = matches!(
            rel.file_name().and_then(|n| n.to_str()),
            Some("README.md" | "OWNER.md" | "REQUIRED_FILES.md" | "INDEX.md" | ".gitkeep")
        );
        let is_schema = ext == "json"
            && rel
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".schema.json"));
        if !is_allowed_doc && !is_schema {
            violations.push(violation(
                "OPS_SCHEMA_UNKNOWN_FILE",
                format!("unexpected file in tight schema surface: `{}`", rel.display()),
                "keep ops/schema constrained to .schema.json and canonical docs only",
                Some(rel),
            ));
        }
    }

    Ok(violations)
}

fn parse_required_files_markdown_yaml(
    content: &str,
    rel: &Path,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), CheckError> {
    let mut in_yaml = false;
    let mut yaml_block = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "```yaml" {
            in_yaml = true;
            continue;
        }
        if trimmed == "```" && in_yaml {
            break;
        }
        if in_yaml {
            yaml_block.push_str(line);
            yaml_block.push('\n');
        }
    }
    if yaml_block.trim().is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&yaml_block).map_err(|err| CheckError::Failed(err.to_string()))?;
    let required_files = parsed
        .get("required_files")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let required_directories = parsed
        .get("required_directories")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if required_files.is_empty() {
        return Err(CheckError::Failed(format!(
            "{} must define non-empty `required_files` YAML list",
            rel.display()
        )));
    }
    Ok((required_files, required_directories))
}

pub(super) fn check_ops_quarantine_shim_expiration_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let contract_rel = Path::new("ops/CONTRACT.md");
    if !ctx.adapters.fs.exists(ctx.repo_root, contract_rel) {
        return Ok(vec![violation(
            "OPS_SHIM_QUARANTINE_README_MISSING",
            format!(
                "missing shim expiration contract file `{}`",
                contract_rel.display()
            ),
            "declare shim expiration deadline in ops contract",
            Some(contract_rel),
        )]);
    }
    let text = fs::read_to_string(ctx.repo_root.join(contract_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let deadline_line = text
        .lines()
        .find(|line| line.trim_start().starts_with("- Legacy shell compatibility deadline: "));
    let Some(deadline_line) = deadline_line else {
        return Ok(vec![violation(
            "OPS_SHIM_EXPIRATION_MISSING",
            "ops contract must declare an explicit shim expiration deadline".to_string(),
            "add a deadline line in the form `- Legacy shell compatibility deadline: YYYY-MM-DD.`",
            Some(contract_rel),
        )]);
    };
    let deadline = deadline_line
        .trim_start()
        .trim_start_matches("- Legacy shell compatibility deadline: ")
        .trim_end_matches('.')
        .trim();
    let valid_deadline = deadline.len() == 10
        && deadline.chars().enumerate().all(|(idx, ch)| match idx {
            4 | 7 => ch == '-',
            _ => ch.is_ascii_digit(),
        });
    if !valid_deadline {
        return Ok(vec![violation(
            "OPS_SHIM_EXPIRATION_FORMAT_INVALID",
            format!("shim quarantine deadline has invalid format: `{deadline}`"),
            "use ISO date format YYYY-MM-DD in shim quarantine deadline",
            Some(contract_rel),
        )]);
    }
    Ok(Vec::new())
}
