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
        let content =
            fs::read_to_string(&required_doc).map_err(|err| CheckError::Failed(err.to_string()))?;
        let required_contract = parse_required_files_markdown_yaml(&content, rel)?;
        let required_files = required_contract.required_files.clone();
        let required_directories = required_contract.required_dirs.clone();
        let domain_root = rel.parent().unwrap_or(Path::new("ops"));
        let domain_readme_rel = domain_root.join("README.md");
        let domain_readme_text = if ctx.adapters.fs.exists(ctx.repo_root, &domain_readme_rel) {
            Some(
                fs::read_to_string(ctx.repo_root.join(&domain_readme_rel))
                    .map_err(|err| CheckError::Failed(err.to_string()))?,
            )
        } else {
            None
        };

        for forbidden in [
            "ops/obs/",
            "ops/schema/obs/",
            "ops/load/k6/manifests/suites.json",
        ] {
            if content.contains(forbidden) {
                violations.push(violation(
                    "OPS_REQUIRED_FILES_FORBIDDEN_REFERENCE",
                    format!(
                        "`{}` contains forbidden reference `{forbidden}`",
                        rel.display()
                    ),
                    "remove retired path references from REQUIRED_FILES.md",
                    Some(rel),
                ));
            }
        }
        for forbidden in &required_contract.forbidden_patterns {
            if forbidden.is_empty() {
                continue;
            }
            for domain_file in walk_files(&ctx.repo_root.join(domain_root)) {
                let domain_rel = domain_file
                    .strip_prefix(ctx.repo_root)
                    .unwrap_or(domain_file.as_path());
                if domain_rel == rel {
                    continue;
                }
                let Ok(domain_text) = fs::read_to_string(&domain_file) else {
                    continue;
                };
                if domain_text.contains(forbidden) {
                    violations.push(violation(
                        "OPS_REQUIRED_FILES_FORBIDDEN_PATTERN_MATCHED",
                        format!(
                            "forbidden pattern `{}` found in `{}`",
                            forbidden,
                            domain_rel.display()
                        ),
                        "remove forbidden path/pattern references from domain files",
                        Some(domain_rel),
                    ));
                }
            }
        }
        if content.contains("TODO") || content.contains("TBD") {
            violations.push(violation(
                "OPS_REQUIRED_FILES_PLACEHOLDER_FORBIDDEN",
                format!("`{}` contains TODO/TBD placeholder markers", rel.display()),
                "replace TODO/TBD placeholders with concrete required file contracts",
                Some(rel),
            ));
        }
        for header_name in ["OWNER.md", "README.md", "INDEX.md", "CONTRACT.md"] {
            let header_rel = domain_root.join(header_name);
            if ctx.adapters.fs.exists(ctx.repo_root, &header_rel)
                && !required_files.iter().any(|file| file == &header_rel)
            {
                violations.push(violation(
                    "OPS_REQUIRED_FILES_DOMAIN_HEADER_MISSING",
                    format!(
                        "`{}` must include domain header `{}` in required_files",
                        rel.display(),
                        header_rel.display()
                    ),
                    "list domain header docs explicitly in required_files",
                    Some(rel),
                ));
            }
        }
        if !required_contract
            .notes
            .iter()
            .any(|note| note.starts_with("authored_root:"))
        {
            violations.push(violation(
                "OPS_REQUIRED_FILES_AUTHORED_ROOT_MISSING",
                format!(
                    "`{}` must include at least one `authored_root:` note",
                    rel.display()
                ),
                "add authored_root notes that point at canonical authored SSOT artifacts",
                Some(rel),
            ));
        }
        let generated_dir = domain_root.join("generated");
        if ctx.adapters.fs.exists(ctx.repo_root, &generated_dir)
            && !required_contract
                .notes
                .iter()
                .any(|note| note.starts_with("generated_output:"))
        {
            violations.push(violation(
                "OPS_REQUIRED_FILES_GENERATED_OUTPUT_MISSING",
                format!(
                    "`{}` must include at least one `generated_output:` note",
                    rel.display()
                ),
                "add generated_output notes for generated artifacts produced in the domain",
                Some(rel),
            ));
        }
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
        for dir_rel in &required_directories {
            let dir_path = ctx.repo_root.join(dir_rel);
            if !dir_path.exists() || !dir_path.is_dir() {
                violations.push(violation(
                    "OPS_REQUIRED_DIRECTORY_MISSING",
                    format!("required directory missing: `{}`", dir_rel.display()),
                    "create required directory or remove stale declaration from REQUIRED_FILES.md",
                    Some(rel),
                ));
            } else {
                let mut entries = fs::read_dir(&dir_path)
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
                if entries.next().is_none() {
                    violations.push(violation(
                        "OPS_EMPTY_DIRECTORY_WITHOUT_GITKEEP",
                        format!(
                            "required directory `{}` is empty and missing `.gitkeep`",
                            dir_rel.display()
                        ),
                        "add `.gitkeep` to empty required directories or remove the stale directory",
                        Some(rel),
                    ));
                }
            }
        }
        for file in walk_files(&ctx.repo_root.join(domain_root)) {
            if file.file_name().and_then(|n| n.to_str()) != Some(".gitkeep") {
                continue;
            }
            let Some(keep_dir) = file
                .parent()
                .and_then(|p| p.strip_prefix(ctx.repo_root).ok())
                .map(PathBuf::from)
            else {
                continue;
            };
            if !required_directories.iter().any(|dir| dir == &keep_dir) {
                violations.push(violation(
                    "OPS_REQUIRED_FILES_GITKEEP_DIR_UNDECLARED",
                    format!(
                        "directory with .gitkeep must be declared in required_dirs: `{}`",
                        keep_dir.display()
                    ),
                    "declare placeholder directories in required_dirs",
                    Some(rel),
                ));
            }
            if let Some(readme_text) = &domain_readme_text {
                let keep_dir_str = keep_dir.display().to_string();
                if !readme_text.contains(&keep_dir_str) {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_README_NOTE_MISSING",
                        format!(
                            "placeholder directory `{}` is not documented in `{}`",
                            keep_dir.display(),
                            domain_readme_rel.display()
                        ),
                        "document each placeholder extension directory in the domain README",
                        Some(&domain_readme_rel),
                    ));
                }
            }
        }
    }

    let actual_gitkeep_dirs = walk_files(&ops_root)
        .into_iter()
        .filter(|p| p.file_name().and_then(|n| n.to_str()) == Some(".gitkeep"))
        .filter_map(|p| {
            p.parent()
                .and_then(|parent| parent.strip_prefix(ctx.repo_root).ok())
                .map(PathBuf::from)
        })
        .collect::<BTreeSet<_>>();
    let placeholder_allowlist_rel = Path::new("ops/inventory/placeholder-dirs.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, placeholder_allowlist_rel) {
        violations.push(violation(
            "OPS_PLACEHOLDER_DIR_ALLOWLIST_MISSING",
            "missing inventory placeholder-dir allowlist `ops/inventory/placeholder-dirs.json`"
                .to_string(),
            "add and maintain ops/inventory/placeholder-dirs.json as the single placeholder-dir allowlist",
            Some(placeholder_allowlist_rel),
        ));
    } else {
        let allowlist_text = fs::read_to_string(ctx.repo_root.join(placeholder_allowlist_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let allowlist_json: serde_json::Value = serde_json::from_str(&allowlist_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let mut allowlisted_dirs = BTreeSet::new();
        if let Some(entries) = allowlist_json
            .get("placeholder_entries")
            .and_then(|v| v.as_array())
        {
            for entry in entries {
                let path = entry
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let owner = entry
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let purpose = entry
                    .get("purpose")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let expected_contents = entry
                    .get("expected_contents")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let lifecycle_policy = entry
                    .get("lifecycle_policy")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();

                if path.is_empty() {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_PATH_MISSING",
                        "placeholder entry is missing `path`".to_string(),
                        "set placeholder_entries[].path in ops/inventory/placeholder-dirs.json",
                        Some(placeholder_allowlist_rel),
                    ));
                    continue;
                }
                allowlisted_dirs.insert(PathBuf::from(path));
                if owner.is_empty() {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_OWNER_MISSING",
                        format!("placeholder entry `{path}` is missing owner"),
                        "set placeholder_entries[].owner for every placeholder directory",
                        Some(placeholder_allowlist_rel),
                    ));
                }
                if purpose.is_empty() {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_PURPOSE_MISSING",
                        format!("placeholder entry `{path}` is missing purpose"),
                        "set placeholder_entries[].purpose for every placeholder directory",
                        Some(placeholder_allowlist_rel),
                    ));
                }
                if expected_contents.is_empty() {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_EXPECTED_CONTENTS_MISSING",
                        format!("placeholder entry `{path}` is missing expected_contents"),
                        "set placeholder_entries[].expected_contents for every placeholder directory",
                        Some(placeholder_allowlist_rel),
                    ));
                }
                let has_permanent = lifecycle_policy.contains("permanent extension point");
                let has_sunset = lifecycle_policy.contains("sunset");
                if !has_permanent && !has_sunset {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_LIFECYCLE_INVALID",
                        format!(
                            "placeholder entry `{path}` lifecycle_policy must declare sunset or permanent extension point"
                        ),
                        "set placeholder_entries[].lifecycle_policy to include `sunset` or `permanent extension point`",
                        Some(placeholder_allowlist_rel),
                    ));
                }
            }
        }
        if allowlisted_dirs.is_empty() {
            allowlisted_dirs = allowlist_json
                .get("placeholder_dirs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str())
                        .map(PathBuf::from)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default();
        }
        for dir in &actual_gitkeep_dirs {
            if !allowlisted_dirs.contains(dir) {
                violations.push(violation(
                    "OPS_PLACEHOLDER_DIR_NOT_ALLOWLISTED",
                    format!(
                        "placeholder directory `{}` is not declared in `{}`",
                        dir.display(),
                        placeholder_allowlist_rel.display()
                    ),
                    "add the directory to ops/inventory/placeholder-dirs.json or remove `.gitkeep`",
                    Some(placeholder_allowlist_rel),
                ));
            }
        }
        for dir in &allowlisted_dirs {
            if !actual_gitkeep_dirs.contains(dir) {
                violations.push(violation(
                    "OPS_PLACEHOLDER_DIR_STALE_ALLOWLIST_ENTRY",
                    format!(
                        "allowlisted placeholder directory `{}` has no `.gitkeep` directory",
                        dir.display()
                    ),
                    "remove stale placeholder allowlist entries or recreate the directory with `.gitkeep`",
                    Some(placeholder_allowlist_rel),
                ));
            }
        }
    }

    let placeholder_report_rel = Path::new("ops/_generated.example/placeholder-dirs-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, placeholder_report_rel) {
        violations.push(violation(
            "OPS_PLACEHOLDER_DIR_REPORT_MISSING",
            format!(
                "missing placeholder directory report `{}`",
                placeholder_report_rel.display()
            ),
            "generate and commit ops/_generated.example/placeholder-dirs-report.json",
            Some(placeholder_report_rel),
        ));
    } else {
        let report_text = fs::read_to_string(ctx.repo_root.join(placeholder_report_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let report_json: serde_json::Value =
            serde_json::from_str(&report_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        if report_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_PLACEHOLDER_DIR_REPORT_BLOCKING",
                "placeholder-dirs-report.json status is not `pass`".to_string(),
                "resolve placeholder directory drift and regenerate placeholder-dirs-report.json",
                Some(placeholder_report_rel),
            ));
        }
        let report_dirs = report_json
            .get("placeholder_dirs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str())
                    .map(PathBuf::from)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        if report_dirs != actual_gitkeep_dirs {
            violations.push(violation(
                "OPS_PLACEHOLDER_DIR_REPORT_DRIFT",
                "placeholder-dirs-report.json does not match current ops .gitkeep directory set"
                    .to_string(),
                "regenerate placeholder-dirs-report.json with deterministic sorted placeholder directories",
                Some(placeholder_report_rel),
            ));
        }
        if report_json.get("placeholder_debt_score").is_none() {
            violations.push(violation(
                "OPS_PLACEHOLDER_DIR_REPORT_DEBT_SCORE_MISSING",
                "placeholder-dirs-report.json must include placeholder_debt_score".to_string(),
                "add placeholder_debt_score metrics to the placeholder directory report",
                Some(placeholder_report_rel),
            ));
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
                format!(
                    "unexpected file in tight inventory meta surface: `{}`",
                    rel.display()
                ),
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
                format!(
                    "unexpected file in tight schema surface: `{}`",
                    rel.display()
                ),
                "keep ops/schema constrained to .schema.json and canonical docs only",
                Some(rel),
            ));
        }
    }

    let ops_root = ctx.repo_root.join("ops");
    for generated_dir in walk_files(&ops_root)
        .into_iter()
        .filter_map(|p| p.parent().map(PathBuf::from))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|p| p.ends_with("generated"))
    {
        let rel_dir = generated_dir
            .strip_prefix(ctx.repo_root)
            .unwrap_or(generated_dir.as_path());
        let readme_rel = rel_dir.join("README.md");
        if !ctx.adapters.fs.exists(ctx.repo_root, &readme_rel) {
            violations.push(violation(
                "OPS_GENERATED_DIRECTORY_README_MISSING",
                format!(
                    "generated directory `{}` is missing README.md",
                    rel_dir.display()
                ),
                "add README.md with generated-only contract in each ops/**/generated directory",
                Some(rel_dir),
            ));
        }
        let domain_root = rel_dir.parent().unwrap_or(Path::new("ops"));
        let domain_required_rel = domain_root.join("REQUIRED_FILES.md");
        if ctx.adapters.fs.exists(ctx.repo_root, &domain_required_rel) {
            let required_text = fs::read_to_string(ctx.repo_root.join(&domain_required_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let rel_dir_str = rel_dir.display().to_string();
            if !required_text.contains(&rel_dir_str) {
                violations.push(violation(
                    "OPS_GENERATED_DIRECTORY_NOT_DECLARED",
                    format!(
                        "generated directory `{}` is not declared in `{}`",
                        rel_dir.display(),
                        domain_required_rel.display()
                    ),
                    "declare generated directory in domain REQUIRED_FILES.md",
                    Some(&domain_required_rel),
                ));
            }
        }
        for file in walk_files(&generated_dir) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            if rel.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let text =
                fs::read_to_string(&file).map_err(|err| CheckError::Failed(err.to_string()))?;
            let value: serde_json::Value =
                serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
            if value.get("generated_by").is_none() {
                violations.push(violation(
                    "OPS_GENERATED_METADATA_MISSING",
                    format!(
                        "generated artifact `{}` must include `generated_by` metadata",
                        rel.display()
                    ),
                    "add generated_by field to generated JSON artifacts",
                    Some(rel),
                ));
            }
            if value.get("schema_version").is_none() {
                violations.push(violation(
                    "OPS_GENERATED_SCHEMA_VERSION_MISSING",
                    format!(
                        "generated artifact `{}` must include `schema_version` metadata",
                        rel.display()
                    ),
                    "add schema_version field to generated JSON artifacts",
                    Some(rel),
                ));
            }
        }
    }

    Ok(violations)
}

pub(super) fn check_ops_domain_contract_structure(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let contract_files = [
        "ops/datasets/CONTRACT.md",
        "ops/e2e/CONTRACT.md",
        "ops/env/CONTRACT.md",
        "ops/inventory/CONTRACT.md",
        "ops/k8s/CONTRACT.md",
        "ops/load/CONTRACT.md",
        "ops/observe/CONTRACT.md",
        "ops/report/CONTRACT.md",
        "ops/schema/CONTRACT.md",
        "ops/stack/CONTRACT.md",
    ];
    let mut violations = Vec::new();
    for rel_str in contract_files {
        let rel = Path::new(rel_str);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_MISSING",
                format!("missing domain contract `{}`", rel.display()),
                "add missing domain CONTRACT.md file",
                Some(rel),
            ));
            continue;
        }
        let text = fs::read_to_string(ctx.repo_root.join(rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if !text.contains("- contract_version: `") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_VERSION_METADATA_MISSING",
                format!(
                    "domain contract `{}` must include `- contract_version: ` metadata",
                    rel.display()
                ),
                "add explicit contract_version metadata in domain CONTRACT.md header",
                Some(rel),
            ));
        }
        let taxonomy = text
            .lines()
            .find_map(|line| {
                let trimmed = line.trim();
                trimmed
                    .strip_prefix("- contract_taxonomy: `")
                    .and_then(|value| value.strip_suffix('`'))
            })
            .unwrap_or_default()
            .to_string();
        if taxonomy.is_empty() {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_TAXONOMY_METADATA_MISSING",
                format!(
                    "domain contract `{}` must include `- contract_taxonomy: ` metadata",
                    rel.display()
                ),
                "set contract_taxonomy to structural, behavioral, or hybrid",
                Some(rel),
            ));
        } else if !matches!(taxonomy.as_str(), "structural" | "behavioral" | "hybrid") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_TAXONOMY_INVALID",
                format!(
                    "domain contract `{}` has invalid contract_taxonomy `{taxonomy}`",
                    rel.display()
                ),
                "use one of: structural, behavioral, hybrid",
                Some(rel),
            ));
        }
        if !text.contains("## Contract Taxonomy") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_TAXONOMY_SECTION_MISSING",
                format!(
                    "domain contract `{}` must include `## Contract Taxonomy`",
                    rel.display()
                ),
                "add structural/behavioral taxonomy section",
                Some(rel),
            ));
        }
        if !text.contains("## Authored vs Generated") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_AUTHORED_GENERATED_SECTION_MISSING",
                format!(
                    "domain contract `{}` must include `## Authored vs Generated`",
                    rel.display()
                ),
                "add an authored-vs-generated table with explicit file paths",
                Some(rel),
            ));
        }
        if !text.contains("## Invariants") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_INVARIANTS_SECTION_MISSING",
                format!(
                    "domain contract `{}` must include `## Invariants`",
                    rel.display()
                ),
                "add an invariants section with explicit, enforceable rules",
                Some(rel),
            ));
            continue;
        }
        if !text.contains("## Enforcement Links") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_ENFORCEMENT_LINKS_MISSING",
                format!(
                    "domain contract `{}` must include `## Enforcement Links`",
                    rel.display()
                ),
                "add enforcement links section that references concrete check ids",
                Some(rel),
            ));
        }
        if !text.contains("## Runtime Evidence Mapping") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_RUNTIME_EVIDENCE_SECTION_MISSING",
                format!(
                    "domain contract `{}` must include `## Runtime Evidence Mapping`",
                    rel.display()
                ),
                "map contract invariants to concrete runtime/generated evidence artifacts",
                Some(rel),
            ));
        }
        if text.contains("locked") || text.contains("Locked") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_STALE_LOCKED_REFERENCE",
                format!(
                    "domain contract `{}` contains stale `locked` wording",
                    rel.display()
                ),
                "remove stale locked-list language from authored domain contracts",
                Some(rel),
            ));
        }
        if !text.contains("checks_") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_ENFORCEMENT_LINK_EMPTY",
                format!(
                    "domain contract `{}` must reference at least one concrete check id",
                    rel.display()
                ),
                "add at least one `checks_*` identifier under enforcement links",
                Some(rel),
            ));
        }
        let mut in_invariants = false;
        let mut invariant_count = 0usize;
        for line in text.lines() {
            if line.starts_with("## ") {
                in_invariants = line == "## Invariants";
                continue;
            }
            if in_invariants {
                let trimmed = line.trim_start();
                if trimmed.starts_with("- ") {
                    invariant_count += 1;
                }
            }
        }
        if invariant_count < 8 {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_INVARIANT_COUNT_TOO_LOW",
                format!(
                    "domain contract `{}` must define at least 8 invariants; found {}",
                    rel.display(),
                    invariant_count
                ),
                "add concrete invariants until the minimum count is satisfied",
                Some(rel),
            ));
        }
    }
    let coverage_rel = Path::new("ops/_generated.example/contract-coverage-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, coverage_rel) {
        violations.push(violation(
            "OPS_CONTRACT_COVERAGE_REPORT_MISSING",
            format!(
                "missing contract coverage report `{}`",
                coverage_rel.display()
            ),
            "generate and commit contract coverage report artifact",
            Some(coverage_rel),
        ));
    } else {
        let coverage_text = fs::read_to_string(ctx.repo_root.join(coverage_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let coverage_json: serde_json::Value = serde_json::from_str(&coverage_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in ["schema_version", "generated_by", "contracts"] {
            if coverage_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_CONTRACT_COVERAGE_REPORT_INVALID",
                    format!(
                        "contract coverage report `{}` is missing `{key}`",
                        coverage_rel.display()
                    ),
                    "include schema_version, generated_by, and contracts fields in coverage report",
                    Some(coverage_rel),
                ));
            }
        }
        let contracts = coverage_json
            .get("contracts")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if contracts.is_empty() {
            violations.push(violation(
                "OPS_CONTRACT_COVERAGE_EMPTY",
                "contract coverage report has no contracts entries".to_string(),
                "populate contract-coverage-report.json with domain contract entries",
                Some(coverage_rel),
            ));
        } else {
            let covered = contracts
                .iter()
                .filter(|entry| {
                    entry
                        .get("authored_vs_generated")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                        && entry
                            .get("invariants")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0)
                            >= 8
                        && entry
                            .get("enforcement_links")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0)
                            >= 1
                })
                .count();
            let threshold = 80usize;
            let coverage_percent = covered * 100 / contracts.len();
            if coverage_percent < threshold {
                violations.push(violation(
                    "OPS_CONTRACT_COVERAGE_THRESHOLD_NOT_MET",
                    format!(
                        "contract coverage threshold not met: {}% < {}%",
                        coverage_percent, threshold
                    ),
                    "raise contract coverage evidence to at least 80% before merge",
                    Some(coverage_rel),
                ));
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_ops_inventory_contract_integrity(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let contracts_map_rel = Path::new("ops/inventory/contracts-map.json");
    let contracts_rel = Path::new("ops/inventory/contracts.json");
    let authority_index_rel = Path::new("ops/inventory/authority-index.json");
    let contracts_meta_rel = Path::new("ops/inventory/meta/contracts.json");
    let namespaces_rel = Path::new("ops/inventory/namespaces.json");
    let layers_rel = Path::new("ops/inventory/layers.json");
    let gates_rel = Path::new("ops/inventory/gates.json");
    let drill_links_rel = Path::new("ops/inventory/drill-contract-links.json");
    let control_graph_rel = Path::new("ops/inventory/control-graph.json");
    let surfaces_rel = Path::new("ops/inventory/surfaces.json");
    let owners_rel = Path::new("ops/inventory/owners.json");
    let policy_rel = Path::new("ops/inventory/policies/dev-atlas-policy.json");
    let policy_schema_rel = Path::new("ops/inventory/policies/dev-atlas-policy.schema.json");
    let pins_rel = Path::new("ops/inventory/pins.yaml");
    let stack_manifest_rel = Path::new("ops/stack/generated/version-manifest.json");
    let stack_toml_rel = Path::new("ops/stack/stack.toml");
    let stack_dependency_graph_rel = Path::new("ops/stack/generated/dependency-graph.json");
    let stack_service_contract_rel = Path::new("ops/stack/service-dependency-contract.json");
    let stack_evolution_policy_rel = Path::new("ops/stack/evolution-policy.json");
    let k8s_install_matrix_rel = Path::new("ops/k8s/install-matrix.json");
    let k8s_rollout_contract_rel = Path::new("ops/k8s/rollout-safety-contract.json");
    let registry_rel = Path::new("ops/inventory/registry.toml");
    let tools_rel = Path::new("ops/inventory/tools.toml");
    let inventory_index_rel = Path::new("ops/_generated.example/inventory-index.json");
    let stack_drift_rel = Path::new("ops/_generated.example/stack-drift-report.json");

    let contracts_map_text = fs::read_to_string(ctx.repo_root.join(contracts_map_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_map: serde_json::Value = serde_json::from_str(&contracts_map_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if contracts_map.get("authoritative").and_then(|v| v.as_bool()) != Some(true) {
        violations.push(violation(
            "OPS_INVENTORY_CONTRACTS_MAP_NOT_AUTHORITATIVE",
            "ops/inventory/contracts-map.json must declare `authoritative: true`".to_string(),
            "mark contracts-map as the authoritative inventory contract manifest",
            Some(contracts_map_rel),
        ));
    }
    let items = contracts_map
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut seen_paths = std::collections::BTreeSet::new();
    let mut item_paths = std::collections::BTreeSet::new();
    for item in &items {
        let Some(path) = item.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        let path_buf = PathBuf::from(path);
        if !seen_paths.insert(path.to_string()) {
            violations.push(violation(
                "OPS_INVENTORY_CONTRACTS_MAP_DUPLICATE_PATH",
                format!("duplicate contracts-map item path `{path}`"),
                "keep unique paths in contracts-map items",
                Some(contracts_map_rel),
            ));
        }
        item_paths.insert(path_buf.clone());
        if !ctx.adapters.fs.exists(ctx.repo_root, &path_buf) {
            violations.push(violation(
                "OPS_INVENTORY_CONTRACTS_MAP_PATH_MISSING",
                format!("contracts-map references missing path `{path}`"),
                "remove stale path or restore referenced inventory artifact",
                Some(contracts_map_rel),
            ));
        }
        let schema = item
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        if schema != "none" && !ctx.adapters.fs.exists(ctx.repo_root, Path::new(schema)) {
            violations.push(violation(
                "OPS_INVENTORY_SCHEMA_REFERENCE_MISSING",
                format!("contracts-map references missing schema `{schema}` for `{path}`"),
                "restore schema path or fix schema pointer in contracts-map",
                Some(contracts_map_rel),
            ));
        }
        let consumer = item
            .get("consumer")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim();
        if consumer.is_empty() {
            violations.push(violation(
                "OPS_INVENTORY_CONTRACTS_MAP_CONSUMER_MISSING",
                format!("contracts-map item `{path}` is missing non-empty `consumer`"),
                "add contracts-map.items[].consumer with the enforcing check or runtime consumer id",
                Some(contracts_map_rel),
            ));
        }
    }

    let contracts_text = fs::read_to_string(ctx.repo_root.join(contracts_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_json: serde_json::Value =
        serde_json::from_str(&contracts_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if contracts_json
        .get("generated_from")
        .and_then(|v| v.as_str())
        != Some("ops/inventory/contracts-map.json")
    {
        violations.push(violation(
            "OPS_INVENTORY_CONTRACTS_GENERATION_METADATA_MISSING",
            "ops/inventory/contracts.json must declare `generated_from: ops/inventory/contracts-map.json`"
                .to_string(),
            "mark contracts.json as a generated mirror of contracts-map",
            Some(contracts_rel),
        ));
    }
    if !item_paths.contains(authority_index_rel) {
        violations.push(violation(
            "OPS_INVENTORY_AUTHORITY_INDEX_NOT_REGISTERED",
            "ops/inventory/authority-index.json must be declared in contracts-map items".to_string(),
            "register ops/inventory/authority-index.json in contracts-map with schema and consumer metadata",
            Some(contracts_map_rel),
        ));
    }
    if !ctx.adapters.fs.exists(ctx.repo_root, authority_index_rel) {
        violations.push(violation(
            "OPS_INVENTORY_AUTHORITY_INDEX_MISSING",
            "missing ops/inventory/authority-index.json".to_string(),
            "restore authority-index.json with authoritative and generated registry roles",
            Some(authority_index_rel),
        ));
    } else {
        let authority_index_text = fs::read_to_string(ctx.repo_root.join(authority_index_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let authority_index_json: serde_json::Value = serde_json::from_str(&authority_index_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;

        let hierarchy = authority_index_json
            .get("authority_hierarchy")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let expected_hierarchy = ["ops/inventory", "ops/schema", "docs"];
        for (position, expected) in expected_hierarchy.iter().enumerate() {
            let actual = hierarchy
                .get(position)
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if actual != *expected {
                violations.push(violation(
                    "OPS_INVENTORY_AUTHORITY_HIERARCHY_DRIFT",
                    format!(
                        "authority_hierarchy[{position}] must be `{expected}` but found `{actual}`"
                    ),
                    "keep authority hierarchy stable as ops/inventory -> ops/schema -> docs",
                    Some(authority_index_rel),
                ));
            }
        }

        let entries = authority_index_json
            .get("authoritative_files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut has_contracts_map_authoritative = false;
        let mut has_contracts_generated = false;
        for entry in entries {
            let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or_default();
            let role = entry.get("role").and_then(|v| v.as_str()).unwrap_or_default();
            if path == "ops/inventory/contracts-map.json" && role == "authoritative" {
                has_contracts_map_authoritative = true;
            }
            if path == "ops/inventory/contracts.json" && role == "generated" {
                let derived_from = entry
                    .get("derived_from")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if derived_from == "ops/inventory/contracts-map.json" {
                    has_contracts_generated = true;
                }
            }
        }
        if !has_contracts_map_authoritative {
            violations.push(violation(
                "OPS_INVENTORY_AUTHORITY_INDEX_CONTRACTS_MAP_ROLE_MISSING",
                "authority-index must mark ops/inventory/contracts-map.json as authoritative"
                    .to_string(),
                "add contracts-map authoritative role to authority-index",
                Some(authority_index_rel),
            ));
        }
        if !has_contracts_generated {
            violations.push(violation(
                "OPS_INVENTORY_AUTHORITY_INDEX_CONTRACTS_MIRROR_ROLE_MISSING",
                "authority-index must mark ops/inventory/contracts.json as generated from contracts-map"
                    .to_string(),
                "set contracts.json role=generated and derived_from=ops/inventory/contracts-map.json",
                Some(authority_index_rel),
            ));
        }
    }
    if ctx.adapters.fs.exists(ctx.repo_root, contracts_meta_rel) {
        violations.push(violation(
            "OPS_INVENTORY_DUPLICATE_CONTRACT_REGISTRY_FORBIDDEN",
            "legacy duplicate contract registry `ops/inventory/meta/contracts.json` is forbidden"
                .to_string(),
            "remove ops/inventory/meta/contracts.json and keep ops/inventory/contracts.json as the single contracts registry",
            Some(contracts_meta_rel),
        ));
    }

    let inventory_root = ctx.repo_root.join("ops/inventory");
    let allowed_unmapped = [
        PathBuf::from("ops/inventory/OWNER.md"),
        PathBuf::from("ops/inventory/README.md"),
        PathBuf::from("ops/inventory/REQUIRED_FILES.md"),
        PathBuf::from("ops/inventory/registry.toml"),
        PathBuf::from("ops/inventory/tools.toml"),
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    for file in walk_files(&inventory_root) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if rel.starts_with(Path::new("ops/inventory/contracts/"))
            || rel.starts_with(Path::new("ops/inventory/meta/"))
            || rel.starts_with(Path::new("ops/inventory/policies/"))
        {
            continue;
        }
        if allowed_unmapped.contains(rel) {
            continue;
        }
        if !item_paths.contains(rel) {
            violations.push(violation(
                "OPS_INVENTORY_ORPHAN_FILE",
                format!(
                    "orphan inventory file not tracked by contracts-map: `{}`",
                    rel.display()
                ),
                "add file to contracts-map or remove orphan artifact",
                Some(rel),
            ));
        }
    }

    let namespaces_text = fs::read_to_string(ctx.repo_root.join(namespaces_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let namespaces_json: serde_json::Value = serde_json::from_str(&namespaces_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let namespace_keys = namespaces_json
        .get("namespaces")
        .and_then(|v| v.as_object())
        .map(|v| v.keys().cloned().collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();

    let layers_text = fs::read_to_string(ctx.repo_root.join(layers_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let layers_json: serde_json::Value =
        serde_json::from_str(&layers_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if layers_text.contains("\"obs\"") {
        violations.push(violation(
            "OPS_INVENTORY_LAYER_LEGACY_OBS_REFERENCE",
            "ops/inventory/layers.json contains legacy `obs` references".to_string(),
            "replace `obs` with canonical `observe` layer naming",
            Some(layers_rel),
        ));
    }
    let layer_namespace_keys = layers_json
        .get("namespaces")
        .and_then(|v| v.as_object())
        .map(|v| v.keys().cloned().collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();
    if namespace_keys != layer_namespace_keys {
        violations.push(violation(
            "OPS_INVENTORY_NAMESPACE_LAYER_DRIFT",
            format!(
                "namespace key mismatch between namespaces.json and layers.json: namespaces={namespace_keys:?} layers={layer_namespace_keys:?}"
            ),
            "keep namespace keys synchronized between namespaces and layer policy",
            Some(namespaces_rel),
        ));
    }

    let gates_text = fs::read_to_string(ctx.repo_root.join(gates_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let gates_json: serde_json::Value =
        serde_json::from_str(&gates_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_text = fs::read_to_string(ctx.repo_root.join(surfaces_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_json: serde_json::Value =
        serde_json::from_str(&surfaces_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let action_ids = surfaces_json
        .get("actions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let gate_ids = gates_json
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|gate| gate.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();

    let control_graph_text = fs::read_to_string(ctx.repo_root.join(control_graph_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let control_graph_json: serde_json::Value = serde_json::from_str(&control_graph_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let nodes = control_graph_json
        .get("nodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let edges = control_graph_json
        .get("edges")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut node_ids = std::collections::BTreeSet::new();
    for node in &nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let node_type = node.get("type").and_then(|v| v.as_str()).unwrap_or_default();
        if id.is_empty() || !node_ids.insert(id.to_string()) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_NODE_INVALID",
                format!("control graph node id is empty or duplicated: `{id}`"),
                "ensure control-graph nodes have unique non-empty ids",
                Some(control_graph_rel),
            ));
        }
        if let Some(path) = node.get("path").and_then(|v| v.as_str()) {
            if !ctx.adapters.fs.exists(ctx.repo_root, Path::new(path)) {
                violations.push(violation(
                    "OPS_INVENTORY_CONTROL_GRAPH_NODE_PATH_MISSING",
                    format!("control graph node path does not exist: `{path}`"),
                    "fix node.path to an existing ops artifact path",
                    Some(control_graph_rel),
                ));
            }
        }
        if node_type == "action" {
            let action_id = id.strip_prefix("action.").unwrap_or(id);
            if !action_ids.contains(action_id) {
                violations.push(violation(
                    "OPS_INVENTORY_CONTROL_GRAPH_ACTION_NOT_IN_SURFACES",
                    format!("control graph action node `{id}` is not defined in surfaces actions"),
                    "align action nodes with ops/inventory/surfaces.json action ids",
                    Some(control_graph_rel),
                ));
            }
        }
        if node_type == "gate" {
            let gate_id = id.strip_prefix("gate.").unwrap_or(id);
            if !gate_ids.contains(gate_id) {
                violations.push(violation(
                    "OPS_INVENTORY_CONTROL_GRAPH_GATE_NOT_IN_GATES",
                    format!("control graph gate node `{id}` is not defined in gates.json"),
                    "align gate nodes with ops/inventory/gates.json ids",
                    Some(control_graph_rel),
                ));
            }
        }
    }
    let mut seen_kinds = std::collections::BTreeSet::new();
    let mut adjacency = std::collections::BTreeMap::<String, Vec<String>>::new();
    for edge in &edges {
        let from = edge.get("from").and_then(|v| v.as_str()).unwrap_or_default();
        let to = edge.get("to").and_then(|v| v.as_str()).unwrap_or_default();
        let kind = edge.get("kind").and_then(|v| v.as_str()).unwrap_or_default();
        if !node_ids.contains(from) || !node_ids.contains(to) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_EDGE_REFERENCE_INVALID",
                format!("control graph edge references unknown node: from=`{from}` to=`{to}`"),
                "ensure edge endpoints reference existing node ids",
                Some(control_graph_rel),
            ));
            continue;
        }
        seen_kinds.insert(kind.to_string());
        if matches!(
            kind,
            "dependency" | "consumer" | "producer" | "lifecycle" | "drift"
        ) {
            adjacency
                .entry(from.to_string())
                .or_default()
                .push(to.to_string());
        }
    }
    for required_kind in ["dependency", "consumer", "producer", "lifecycle", "drift"] {
        if !seen_kinds.contains(required_kind) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_EDGE_KIND_MISSING",
                format!("control graph is missing required edge kind `{required_kind}`"),
                "add at least one edge for each required control graph edge kind",
                Some(control_graph_rel),
            ));
        }
    }
    fn has_cycle(
        node: &str,
        adjacency: &std::collections::BTreeMap<String, Vec<String>>,
        visiting: &mut std::collections::BTreeSet<String>,
        visited: &mut std::collections::BTreeSet<String>,
    ) -> bool {
        if visiting.contains(node) {
            return true;
        }
        if visited.contains(node) {
            return false;
        }
        visiting.insert(node.to_string());
        if let Some(neighbors) = adjacency.get(node) {
            for next in neighbors {
                if has_cycle(next, adjacency, visiting, visited) {
                    return true;
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.to_string());
        false
    }
    let mut visiting = std::collections::BTreeSet::new();
    let mut visited = std::collections::BTreeSet::new();
    for node in node_ids.iter() {
        if has_cycle(node, &adjacency, &mut visiting, &mut visited) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_CYCLE_DETECTED",
                "control graph contains a cycle in dependency/consumer/producer/lifecycle/drift edges"
                    .to_string(),
                "break cyclic control graph edges to keep inventory graph acyclic",
                Some(control_graph_rel),
            ));
            break;
        }
    }

    let drills_text = fs::read_to_string(ctx.repo_root.join(Path::new("ops/inventory/drills.json")))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let drills_json: serde_json::Value =
        serde_json::from_str(&drills_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let drill_ids = drills_json
        .get("drills")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let drill_links_text = fs::read_to_string(ctx.repo_root.join(drill_links_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let drill_links_json: serde_json::Value = serde_json::from_str(&drill_links_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let linked_drills = drill_links_json
        .get("links")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("drill_id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    for drill_id in &drill_ids {
        if !linked_drills.contains(drill_id) {
            violations.push(violation(
                "OPS_INVENTORY_DRILL_CONTRACT_LINK_MISSING",
                format!("drill id `{drill_id}` has no mapping in drill-contract-links"),
                "add drill-contract-links entry for each drill id with at least one contract path",
                Some(drill_links_rel),
            ));
        }
    }
    if let Some(links) = drill_links_json.get("links").and_then(|v| v.as_array()) {
        for link in links {
            let drill_id = link
                .get("drill_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !drill_ids.contains(drill_id) {
                violations.push(violation(
                    "OPS_INVENTORY_DRILL_CONTRACT_LINK_UNKNOWN_DRILL",
                    format!("drill-contract-links references unknown drill id `{drill_id}`"),
                    "remove stale drill link entries or restore missing drill ids",
                    Some(drill_links_rel),
                ));
            }
            for contract_path in link
                .get("contract_paths")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter_map(|item| item.as_str())
            {
                if !ctx
                    .adapters
                    .fs
                    .exists(ctx.repo_root, Path::new(contract_path))
                {
                    violations.push(violation(
                        "OPS_INVENTORY_DRILL_CONTRACT_LINK_PATH_MISSING",
                        format!(
                            "drill-contract-links references missing contract path `{contract_path}`"
                        ),
                        "fix contract_paths to existing ops domain CONTRACT.md files",
                        Some(drill_links_rel),
                    ));
                }
            }
        }
    }
    if let Some(gates) = gates_json.get("gates").and_then(|v| v.as_array()) {
        let gate_ids = gates
            .iter()
            .filter_map(|gate| gate.get("id").and_then(|v| v.as_str()))
            .collect::<std::collections::BTreeSet<_>>();
        let required_release_gates = [
            "ops.gate.ssot",
            "ops.gate.validate",
            "ops.gate.structure",
            "ops.gate.docs",
            "ops.gate.generated",
            "ops.gate.evidence",
            "ops.gate.fixtures",
            "ops.gate.naming",
            "ops.gate.inventory",
            "ops.gate.schema",
        ];
        for required_gate in required_release_gates {
            if !gate_ids.contains(required_gate) {
                violations.push(violation(
                    "OPS_INVENTORY_RELEASE_GATE_MISSING",
                    format!("required release gate id missing from gates.json: `{required_gate}`"),
                    "add the missing release gate id to ops/inventory/gates.json",
                    Some(gates_rel),
                ));
            }
        }
        for gate in gates {
            let Some(action_id) = gate.get("action_id").and_then(|v| v.as_str()) else {
                continue;
            };
            if !action_ids.contains(action_id) {
                violations.push(violation(
                    "OPS_INVENTORY_GATE_ACTION_NOT_FOUND",
                    format!("gate action id `{action_id}` is not present in surfaces actions"),
                    "align ops/inventory/gates.json action_id fields with ops/inventory/surfaces.json",
                    Some(gates_rel),
                ));
            }
        }
    }

    let owners_text = fs::read_to_string(ctx.repo_root.join(owners_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let owners_json: serde_json::Value =
        serde_json::from_str(&owners_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let owner_areas = owners_json
        .get("areas")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let required_owner_prefixes = [
        "ops/datasets",
        "ops/e2e",
        "ops/env",
        "ops/inventory",
        "ops/k8s",
        "ops/load",
        "ops/observe",
        "ops/report",
        "ops/schema",
        "ops/stack",
    ];
    for prefix in required_owner_prefixes {
        if !owner_areas.contains_key(prefix) {
            violations.push(violation(
                "OPS_OWNER_DOMAIN_MAPPING_MISSING",
                format!("owners.json is missing required area mapping `{prefix}`"),
                "add owner mapping for each top-level ops domain",
                Some(owners_rel),
            ));
        }
    }
    let owner_prefixes = owner_areas.keys().cloned().collect::<Vec<_>>();
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let rel_str = rel.display().to_string();
        if rel_str.starts_with("ops/_generated/") {
            continue;
        }
        let has_owner = owner_prefixes.iter().any(|prefix| {
            rel_str == *prefix
                || rel_str
                    .strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        });
        if !has_owner {
            violations.push(violation(
                "OPS_OWNER_ASSIGNMENT_MISSING",
                format!("ops file has no owner mapping in owners.json: `{rel_str}`"),
                "add an owners.json area mapping that covers this file path",
                Some(owners_rel),
            ));
        }
    }

    let registry_text = fs::read_to_string(ctx.repo_root.join(registry_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let tools_text = fs::read_to_string(ctx.repo_root.join(tools_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if registry_text.contains("[[tools]]") {
        violations.push(violation(
            "OPS_INVENTORY_REGISTRY_TOOLS_SURFACE_COLLISION",
            "ops/inventory/registry.toml must not define [[tools]] entries".to_string(),
            "keep registry.toml for checks/actions and tools.toml for tool probes",
            Some(registry_rel),
        ));
    }
    if tools_text.contains("[[checks]]") || tools_text.contains("[[actions]]") {
        violations.push(violation(
            "OPS_INVENTORY_TOOLS_REGISTRY_SURFACE_COLLISION",
            "ops/inventory/tools.toml must not define [[checks]] or [[actions]] entries"
                .to_string(),
            "keep tools.toml limited to [[tools]] entries",
            Some(tools_rel),
        ));
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, policy_schema_rel) {
        violations.push(violation(
            "OPS_INVENTORY_POLICY_SCHEMA_MISSING",
            format!("missing policy schema `{}`", policy_schema_rel.display()),
            "restore dev-atlas policy schema file",
            Some(policy_schema_rel),
        ));
    }
    let policy_text = fs::read_to_string(ctx.repo_root.join(policy_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let policy_json: serde_json::Value =
        serde_json::from_str(&policy_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if policy_json.get("schema_version").is_none() || policy_json.get("mode").is_none() {
        violations.push(violation(
            "OPS_INVENTORY_POLICY_REQUIRED_KEYS_MISSING",
            "dev-atlas policy is missing required top-level keys".to_string(),
            "ensure dev-atlas policy includes at least schema_version and mode",
            Some(policy_rel),
        ));
    }

    let pins_text = fs::read_to_string(ctx.repo_root.join(pins_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let pins_yaml: serde_yaml::Value =
        serde_yaml::from_str(&pins_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let pins_images = pins_yaml
        .get("images")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();
    let stack_manifest_text = fs::read_to_string(ctx.repo_root.join(stack_manifest_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let stack_manifest_json: serde_json::Value = serde_json::from_str(&stack_manifest_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if let Some(obj) = stack_manifest_json.as_object() {
        for (key, value) in obj {
            if key == "schema_version" {
                continue;
            }
            let image_value = value.as_str().unwrap_or_default();
            let pin_value = pins_images
                .get(serde_yaml::Value::String(key.clone()))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if pin_value != image_value {
                violations.push(violation(
                    "OPS_INVENTORY_PIN_STACK_DRIFT",
                    format!("stack manifest image `{key}` differs from inventory pin value"),
                    "regenerate stack generated version-manifest from inventory pins",
                    Some(stack_manifest_rel),
                ));
            }
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, inventory_index_rel) {
        violations.push(violation(
            "OPS_INVENTORY_INDEX_ARTIFACT_MISSING",
            format!(
                "missing generated inventory index artifact `{}`",
                inventory_index_rel.display()
            ),
            "generate and commit ops/_generated.example/inventory-index.json",
            Some(inventory_index_rel),
        ));
    } else {
        let inventory_index_text = fs::read_to_string(ctx.repo_root.join(inventory_index_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let inventory_index_json: serde_json::Value =
            serde_json::from_str(&inventory_index_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let indexed_paths = inventory_index_json
            .get("items")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("path").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<std::collections::BTreeSet<_>>()
            })
            .unwrap_or_default();
        let expected_inventory_paths = walk_files(&inventory_root)
            .into_iter()
            .filter_map(|file| file.strip_prefix(ctx.repo_root).ok().map(PathBuf::from))
            .filter(|rel| {
                rel.extension()
                    .and_then(|v| v.to_str())
                    .is_some_and(|ext| matches!(ext, "json" | "yaml" | "yml" | "toml"))
            })
            .map(|rel| rel.display().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        for rel in expected_inventory_paths.difference(&indexed_paths) {
            violations.push(violation(
                "OPS_INVENTORY_INDEX_COVERAGE_MISSING",
                format!("inventory-index.json is missing inventory artifact `{rel}`"),
                "regenerate ops/_generated.example/inventory-index.json to include every ops/inventory data artifact",
                Some(inventory_index_rel),
            ));
        }
    }

    if ctx.adapters.fs.exists(ctx.repo_root, stack_toml_rel)
        && ctx.adapters.fs.exists(ctx.repo_root, stack_dependency_graph_rel)
    {
        let stack_toml_text = fs::read_to_string(ctx.repo_root.join(stack_toml_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let stack_toml: toml::Value =
            toml::from_str(&stack_toml_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let graph_text = fs::read_to_string(ctx.repo_root.join(stack_dependency_graph_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let graph_json: serde_json::Value =
            serde_json::from_str(&graph_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let toml_profiles = stack_toml
            .get("profiles")
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();
        let graph_profiles = graph_json
            .get("profiles")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let toml_profile_keys = toml_profiles.keys().cloned().collect::<BTreeSet<_>>();
        let graph_profile_keys = graph_profiles.keys().cloned().collect::<BTreeSet<_>>();
        if toml_profile_keys != graph_profile_keys {
            violations.push(violation(
                "OPS_STACK_DEPENDENCY_GRAPH_PROFILE_DRIFT",
                format!(
                    "dependency-graph profile keys drift from stack.toml: stack={toml_profile_keys:?} graph={graph_profile_keys:?}"
                ),
                "regenerate ops/stack/generated/dependency-graph.json from ops/stack/stack.toml",
                Some(stack_dependency_graph_rel),
            ));
        }

        if ctx.adapters.fs.exists(ctx.repo_root, stack_service_contract_rel) {
            let service_contract_text =
                fs::read_to_string(ctx.repo_root.join(stack_service_contract_rel))
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
            let service_contract_json: serde_json::Value =
                serde_json::from_str(&service_contract_text)
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
            let services = service_contract_json
                .get("services")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut has_critical_service = false;
            for service in services {
                let service_id = service
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown-service");
                let component = service
                    .get("component")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if component.is_empty() {
                    continue;
                }
                if service
                    .get("critical")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    has_critical_service = true;
                }
                let component_rel = Path::new(component);
                if !ctx.adapters.fs.exists(ctx.repo_root, component_rel) {
                    violations.push(violation(
                        "OPS_STACK_SERVICE_COMPONENT_MISSING",
                        format!(
                            "stack service `{service_id}` references missing component `{component}`"
                        ),
                        "fix component path in ops/stack/service-dependency-contract.json",
                        Some(stack_service_contract_rel),
                    ));
                    continue;
                }
                for profile in service
                    .get("required_profiles")
                    .and_then(|v| v.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str())
                {
                    let Some(profile_value) = toml_profiles.get(profile) else {
                        violations.push(violation(
                            "OPS_STACK_SERVICE_REQUIRED_PROFILE_MISSING",
                            format!(
                                "stack service `{service_id}` requires unknown profile `{profile}`"
                            ),
                            "align required_profiles with ops/stack/stack.toml profile keys",
                            Some(stack_service_contract_rel),
                        ));
                        continue;
                    };
                    let profile_components = profile_value
                        .get("components")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let profile_component_paths = profile_components
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<BTreeSet<_>>();
                    if !profile_component_paths.contains(component) {
                        violations.push(violation(
                            "OPS_STACK_SERVICE_PROFILE_COMPONENT_DRIFT",
                            format!(
                                "stack service `{service_id}` component `{component}` missing from profile `{profile}` components"
                            ),
                            "keep service-dependency-contract required_profiles aligned with stack.toml components",
                            Some(stack_service_contract_rel),
                        ));
                    }
                }
            }
            if !has_critical_service {
                violations.push(violation(
                    "OPS_STACK_SERVICE_CRITICAL_COVERAGE_MISSING",
                    "service-dependency-contract must define at least one critical service"
                        .to_string(),
                    "mark core stack services as critical in ops/stack/service-dependency-contract.json",
                    Some(stack_service_contract_rel),
                ));
            }
        } else {
            violations.push(violation(
                "OPS_STACK_SERVICE_CONTRACT_MISSING",
                format!(
                    "missing stack service dependency contract `{}`",
                    stack_service_contract_rel.display()
                ),
                "restore ops/stack/service-dependency-contract.json",
                Some(stack_service_contract_rel),
            ));
        }
    }

    if ctx.adapters.fs.exists(ctx.repo_root, stack_evolution_policy_rel) {
        let evolution_text = fs::read_to_string(ctx.repo_root.join(stack_evolution_policy_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let evolution_json: serde_json::Value = serde_json::from_str(&evolution_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in [
            "schema_version",
            "policy_version",
            "freeze_rules",
            "change_controls",
            "compatibility",
        ] {
            if evolution_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_STACK_EVOLUTION_POLICY_FIELD_MISSING",
                    format!("stack evolution policy missing required key `{key}`"),
                    "add missing required keys to ops/stack/evolution-policy.json",
                    Some(stack_evolution_policy_rel),
                ));
            }
        }
    } else {
        violations.push(violation(
            "OPS_STACK_EVOLUTION_POLICY_MISSING",
            format!(
                "missing stack evolution policy `{}`",
                stack_evolution_policy_rel.display()
            ),
            "restore ops/stack/evolution-policy.json",
            Some(stack_evolution_policy_rel),
        ));
    }

    if ctx.adapters.fs.exists(ctx.repo_root, k8s_install_matrix_rel)
        && ctx.adapters.fs.exists(ctx.repo_root, k8s_rollout_contract_rel)
    {
        let install_matrix_text = fs::read_to_string(ctx.repo_root.join(k8s_install_matrix_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let install_matrix_json: serde_json::Value = serde_json::from_str(&install_matrix_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let matrix_profiles = install_matrix_json
            .get("profiles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let matrix_by_name = matrix_profiles
            .iter()
            .filter_map(|entry| {
                let name = entry.get("name").and_then(|v| v.as_str())?;
                Some((name.to_string(), entry.clone()))
            })
            .collect::<BTreeMap<_, _>>();

        let rollout_contract_text =
            fs::read_to_string(ctx.repo_root.join(k8s_rollout_contract_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
        let rollout_contract_json: serde_json::Value =
            serde_json::from_str(&rollout_contract_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
        let rollout_profiles = rollout_contract_json
            .get("profiles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for profile in &rollout_profiles {
            let profile_name = profile
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if profile_name.is_empty() {
                continue;
            }
            let Some(matrix_entry) = matrix_by_name.get(profile_name) else {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_PROFILE_MISSING_FROM_INSTALL_MATRIX",
                    format!(
                        "rollout-safety-contract profile `{profile_name}` is missing from install-matrix"
                    ),
                    "align rollout-safety-contract profiles with ops/k8s/install-matrix.json",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            };
            let contract_suite = profile.get("suite").and_then(|v| v.as_str()).unwrap_or_default();
            let matrix_suite = matrix_entry
                .get("suite")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if contract_suite != matrix_suite {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_SUITE_DRIFT",
                    format!(
                        "rollout-safety-contract suite drift for profile `{profile_name}`: contract=`{contract_suite}` matrix=`{matrix_suite}`"
                    ),
                    "align rollout-safety-contract suite values with install-matrix",
                    Some(k8s_rollout_contract_rel),
                ));
            }
            let values_file = profile
                .get("values_file")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let matrix_values_file = matrix_entry
                .get("values_file")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if values_file != matrix_values_file {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_VALUES_FILE_DRIFT",
                    format!(
                        "rollout-safety-contract values_file drift for profile `{profile_name}`: contract=`{values_file}` matrix=`{matrix_values_file}`"
                    ),
                    "align rollout-safety-contract values_file with install-matrix",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            }
            let values_rel = Path::new(values_file);
            if !ctx.adapters.fs.exists(ctx.repo_root, values_rel) {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_VALUES_FILE_MISSING",
                    format!("rollout-safety-contract references missing values file `{values_file}`"),
                    "restore missing values file or update rollout-safety-contract",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            }
            let values_text = fs::read_to_string(ctx.repo_root.join(values_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let values_yaml: serde_yaml::Value = serde_yaml::from_str(&values_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let warmup_required = profile
                .get("warmup_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if warmup_required {
                let warmup_enabled = values_yaml
                    .get("cache")
                    .and_then(|v| v.get("warmupEnabled"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !warmup_enabled {
                    violations.push(violation(
                        "OPS_K8S_WARMUP_REQUIRED_BUT_DISABLED",
                        format!(
                            "profile `{profile_name}` requires warmup but values file disables cache.warmupEnabled"
                        ),
                        "enable cache.warmupEnabled for warmup-required rollout profiles",
                        Some(values_rel),
                    ));
                }
            }
            let readiness_required = profile
                .get("readiness_path_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if readiness_required {
                let readiness_path = values_yaml
                    .get("server")
                    .and_then(|v| v.get("readinessProbePath"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if readiness_path.trim().is_empty() {
                    violations.push(violation(
                        "OPS_K8S_READINESS_PATH_REQUIRED_MISSING",
                        format!(
                            "profile `{profile_name}` requires readiness probe path but server.readinessProbePath is missing"
                        ),
                        "define server.readinessProbePath in profile values file",
                        Some(values_rel),
                    ));
                }
            }
        }
    } else if !ctx.adapters.fs.exists(ctx.repo_root, k8s_rollout_contract_rel) {
        violations.push(violation(
            "OPS_K8S_ROLLOUT_CONTRACT_MISSING",
            format!(
                "missing k8s rollout safety contract `{}`",
                k8s_rollout_contract_rel.display()
            ),
            "restore ops/k8s/rollout-safety-contract.json",
            Some(k8s_rollout_contract_rel),
        ));
    }

    let registry_drift_rel = Path::new("ops/_generated.example/registry-drift-report.json");
    let control_graph_diff_rel = Path::new("ops/_generated.example/control-graph-diff-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, registry_drift_rel) {
        violations.push(violation(
            "OPS_INVENTORY_REGISTRY_DRIFT_REPORT_MISSING",
            format!(
                "missing registry drift report `{}`",
                registry_drift_rel.display()
            ),
            "generate and commit ops/_generated.example/registry-drift-report.json",
            Some(registry_drift_rel),
        ));
    } else {
        let registry_drift_text = fs::read_to_string(ctx.repo_root.join(registry_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let registry_drift_json: serde_json::Value = serde_json::from_str(&registry_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if registry_drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_INVENTORY_REGISTRY_DRIFT_REPORT_BLOCKING",
                "registry-drift-report.json status is not `pass`".to_string(),
                "resolve inventory registry drift and regenerate registry-drift-report.json",
                Some(registry_drift_rel),
            ));
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, control_graph_diff_rel) {
        violations.push(violation(
            "OPS_INVENTORY_CONTROL_GRAPH_DIFF_REPORT_MISSING",
            format!(
                "missing control graph diff report `{}`",
                control_graph_diff_rel.display()
            ),
            "generate and commit ops/_generated.example/control-graph-diff-report.json",
            Some(control_graph_diff_rel),
        ));
    } else {
        let control_graph_diff_text = fs::read_to_string(ctx.repo_root.join(control_graph_diff_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let control_graph_diff_json: serde_json::Value =
            serde_json::from_str(&control_graph_diff_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
        if control_graph_diff_json
            .get("status")
            .and_then(|v| v.as_str())
            != Some("pass")
        {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_DIFF_REPORT_BLOCKING",
                "control-graph-diff-report.json status is not `pass`".to_string(),
                "resolve control graph drift and regenerate control-graph-diff-report.json",
                Some(control_graph_diff_rel),
            ));
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, stack_drift_rel) {
        violations.push(violation(
            "OPS_STACK_DRIFT_REPORT_MISSING",
            format!("missing stack drift report `{}`", stack_drift_rel.display()),
            "generate and commit ops/_generated.example/stack-drift-report.json",
            Some(stack_drift_rel),
        ));
    } else {
        let stack_drift_text = fs::read_to_string(ctx.repo_root.join(stack_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let stack_drift_json: serde_json::Value = serde_json::from_str(&stack_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if stack_drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_STACK_DRIFT_REPORT_BLOCKING",
                "stack-drift-report.json status is not `pass`".to_string(),
                "resolve stack drift and regenerate stack-drift-report.json",
                Some(stack_drift_rel),
            ));
        }
    }

    Ok(violations)
}

pub(super) fn check_ops_file_usage_and_orphan_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let ledger_rel = Path::new("ops/_generated.example/ops-ledger.json");
    let ledger_text = fs::read_to_string(ctx.repo_root.join(ledger_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let ledger_json: serde_json::Value =
        serde_json::from_str(&ledger_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if ledger_json.get("schema_version").and_then(|v| v.as_i64()) != Some(1)
        || ledger_json.get("generated_by").is_none()
    {
        violations.push(violation(
            "OPS_LEDGER_METADATA_INVALID",
            "ops-ledger.json must include schema_version=1 and generated_by".to_string(),
            "regenerate ops/_generated.example/ops-ledger.json with required metadata",
            Some(ledger_rel),
        ));
    }
    let ledger_entries = ledger_json
        .get("entries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let ledger_paths = ledger_entries
        .iter()
        .filter_map(|entry| entry.get("path").and_then(|v| v.as_str()))
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();

    let usage_report_rel = Path::new("ops/_generated.example/file-usage-report.json");
    let usage_report_text = fs::read_to_string(ctx.repo_root.join(usage_report_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let usage_report_json: serde_json::Value = serde_json::from_str(&usage_report_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if usage_report_json.get("schema_version").and_then(|v| v.as_i64()) != Some(1)
        || usage_report_json.get("generated_by").is_none()
    {
        violations.push(violation(
            "OPS_FILE_USAGE_REPORT_METADATA_MISSING",
            "ops/_generated.example/file-usage-report.json must include schema_version=1 and generated_by"
                .to_string(),
            "add schema_version and generated_by to file usage report",
            Some(usage_report_rel),
        ));
    }
    let orphan_allowlist_prefixes = usage_report_json
        .get("orphan_allowlist_prefixes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for prefix in &orphan_allowlist_prefixes {
        let is_fixture_prefix = prefix.contains("/fixtures/");
        let is_curated_example_prefix = prefix.contains("/examples/");
        if !is_fixture_prefix && !is_curated_example_prefix {
            violations.push(violation(
                "OPS_FILE_USAGE_ALLOWLIST_SCOPE_INVALID",
                format!(
                    "orphan allowlist prefix `{prefix}` is outside fixture/example scope"
                ),
                "limit orphan allowlist prefixes to fixture payloads and curated examples",
                Some(usage_report_rel),
            ));
        }
    }

    let contracts_map_rel = Path::new("ops/inventory/contracts-map.json");
    let contracts_map_text = fs::read_to_string(ctx.repo_root.join(contracts_map_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_map_json: serde_json::Value = serde_json::from_str(&contracts_map_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let registry_inputs = contracts_map_json
        .get("items")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("path").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let schema_index_rel = Path::new("ops/schema/generated/schema-index.json");
    let schema_index_text = fs::read_to_string(ctx.repo_root.join(schema_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let schema_index_json: serde_json::Value = serde_json::from_str(&schema_index_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let schema_files = schema_index_json
        .get("files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let mut required_file_refs = BTreeSet::new();
    for req in walk_files(&ctx.repo_root.join("ops")) {
        let rel = req.strip_prefix(ctx.repo_root).unwrap_or(req.as_path());
        if rel.file_name().and_then(|name| name.to_str()) != Some("REQUIRED_FILES.md") {
            continue;
        }
        let content = fs::read_to_string(&req).map_err(|err| CheckError::Failed(err.to_string()))?;
        let parsed = parse_required_files_markdown_yaml(&content, rel)?;
        for required in parsed.required_files {
            required_file_refs.insert(required.display().to_string());
        }
    }

    let mut docs_refs = BTreeSet::new();
    for root in ["docs", "ops"] {
        for doc in walk_files(&ctx.repo_root.join(root)) {
            if doc.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let text = fs::read_to_string(&doc).map_err(|err| CheckError::Failed(err.to_string()))?;
            docs_refs.extend(extract_ops_data_paths(&text));
        }
    }

    let mut computed_orphans = Vec::new();
    let mut ops_all_files = BTreeSet::new();
    let mut registry_count_by_domain = BTreeMap::<String, usize>::new();
    let mut generated_count_by_domain = BTreeMap::<String, usize>::new();
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let rel_str = rel.display().to_string();
        ops_all_files.insert(rel_str.clone());
        let Some(ext) = rel.extension().and_then(|v| v.to_str()) else {
            continue;
        };
        if !matches!(ext, "json" | "yaml" | "yml" | "toml") {
            continue;
        }
        let domain = rel
            .components()
            .nth(1)
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("root")
            .to_string();
        let is_schema = rel_str.starts_with("ops/schema/");
        let is_generated =
            rel_str.contains("/generated/") || rel_str.starts_with("ops/_generated.example/");
        let is_registry_input = registry_inputs.contains(&rel_str)
            || rel_str.starts_with("ops/inventory/contracts/")
            || rel_str.starts_with("ops/inventory/policies/")
            || rel_str.starts_with("ops/k8s/charts/")
            || rel_str.starts_with("ops/k8s/values/")
            || rel_str.starts_with("ops/observe/pack/")
            || rel_str.starts_with("ops/observe/alerts/")
            || rel_str.starts_with("ops/observe/rules/")
            || rel_str.starts_with("ops/observe/dashboards/")
            || rel_str.starts_with("ops/load/compose/")
            || rel_str.starts_with("ops/load/baselines/")
            || rel_str.starts_with("ops/load/thresholds/")
            || rel_str.starts_with("ops/e2e/manifests/")
            || rel_str.starts_with("ops/stack/")
            || rel_str.contains("/contracts/")
            || rel_str.contains("/scenarios/")
            || rel_str.contains("/suites/");
        let is_docs_ref = docs_refs.contains(&rel_str);
        let is_required_ref = required_file_refs.contains(&rel_str);
        let is_schema_ref = schema_files.contains(&rel_str);
        let is_fixture_or_test = rel_str.contains("/fixtures/")
            || rel_str.contains("/tests/")
            || rel_str.contains("/goldens/")
            || rel_str.contains("/realdata/");
        if is_generated {
            *generated_count_by_domain.entry(domain).or_insert(0) += 1;
        } else {
            *registry_count_by_domain.entry(domain).or_insert(0) += 1;
        }
        if !(is_schema
            || is_generated
            || is_registry_input
            || is_docs_ref
            || is_required_ref
            || is_schema_ref
            || is_fixture_or_test)
        {
            computed_orphans.push(rel_str);
        }
    }

    let effective_orphans = computed_orphans
        .iter()
        .filter(|path| {
            !orphan_allowlist_prefixes
                .iter()
                .any(|prefix| path.starts_with(prefix))
        })
        .cloned()
        .collect::<Vec<_>>();

    if !effective_orphans.is_empty() {
        violations.push(violation(
            "OPS_DATA_FILE_ORPHAN_FOUND",
            format!(
                "orphan ops data artifacts detected: {}",
                effective_orphans.join(", ")
            ),
            "remove orphan data files or classify them through contracts-map, schema-index, docs, and REQUIRED_FILES",
            Some(Path::new("ops")),
        ));
    }

    let ledger_missing = ops_all_files
        .iter()
        .filter(|path| !ledger_paths.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    if !ledger_missing.is_empty() {
        violations.push(violation(
            "OPS_LEDGER_FILE_MISSING",
            format!(
                "ops ledger is missing {} file entries",
                ledger_missing.len()
            ),
            "regenerate ops-ledger.json and include every ops file",
            Some(ledger_rel),
        ));
    }
    for entry in &ledger_entries {
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        if path.is_empty() || !ctx.adapters.fs.exists(ctx.repo_root, Path::new(path)) {
            violations.push(violation(
                "OPS_LEDGER_REFERENCE_MISSING",
                format!("ops ledger references missing path `{path}`"),
                "remove stale ledger entries or restore referenced files",
                Some(ledger_rel),
            ));
            continue;
        }
        let reasons = entry
            .get("reasons")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if reasons.is_empty() {
            violations.push(violation(
                "OPS_LEDGER_REASON_MISSING",
                format!("ops ledger entry has empty reasons: `{path}`"),
                "add at least one necessity reason per ledger entry",
                Some(ledger_rel),
            ));
        }
        if let Some(schema_ref) = entry.get("schema_ref").and_then(|v| v.as_str()) {
            if !schema_ref.is_empty() && !ctx.adapters.fs.exists(ctx.repo_root, Path::new(schema_ref)) {
                violations.push(violation(
                    "OPS_LEDGER_SCHEMA_REFERENCE_MISSING",
                    format!("ledger schema reference is missing: `{schema_ref}`"),
                    "fix or remove stale schema_ref values in ops ledger",
                    Some(ledger_rel),
                ));
            }
        }
        for required_by in entry
            .get("required_by")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|v| v.as_str())
        {
            if !required_by.is_empty() && !ctx.adapters.fs.exists(ctx.repo_root, Path::new(required_by)) {
                violations.push(violation(
                    "OPS_LEDGER_REQUIRED_BY_MISSING",
                    format!("ledger required_by reference is missing: `{required_by}`"),
                    "fix required_by references in ops ledger",
                    Some(ledger_rel),
                ));
            }
        }
        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or_default();
        if entry_type == "authored" && path.contains("/generated/") {
            violations.push(violation(
                "OPS_LEDGER_AUTHORED_IN_GENERATED_PATH",
                format!("authored ledger entry lives under generated path: `{path}`"),
                "mark generated-path entries as generated or move authored files out of generated directories",
                Some(ledger_rel),
            ));
        }
        if entry_type == "generated"
            && !path.contains("/generated/")
            && !path.starts_with("ops/_generated.example/")
        {
            violations.push(violation(
                "OPS_LEDGER_GENERATED_OUTSIDE_GENERATED_PATH",
                format!("generated ledger entry is outside generated paths: `{path}`"),
                "move generated artifacts under ops/**/generated or ops/_generated.example",
                Some(ledger_rel),
            ));
        }
    }

    let allowlist_rel = Path::new("ops/_generated.example/ALLOWLIST.json");
    let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_json: serde_json::Value = serde_json::from_str(&allowlist_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_set = allowlist_json
        .get("allowed_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    for entry in &ledger_entries {
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or_default();
        if entry_type == "curated_evidence" && !allowlist_set.contains(path) {
            violations.push(violation(
                "OPS_LEDGER_CURATED_NOT_ALLOWLISTED",
                format!("curated evidence path is not in allowlist: `{path}`"),
                "add curated evidence paths to ops/_generated.example/ALLOWLIST.json",
                Some(allowlist_rel),
            ));
        }
    }

    let binary_allowed = [".json", ".yaml", ".yml", ".toml", ".md", ".txt", ".lock", ".js"];
    for path in &ops_all_files {
        if path.contains("/assets/") {
            continue;
        }
        let ext = Path::new(path)
            .extension()
            .and_then(|v| v.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        if !binary_allowed.iter().any(|allowed| *allowed == ext) && !path.ends_with(".gitkeep") {
            violations.push(violation(
                "OPS_BINARY_OUTSIDE_ASSETS_FORBIDDEN",
                format!("potential binary or unsupported file outside assets: `{path}`"),
                "keep non-text artifacts only under ops/**/assets and declare them in fixture contracts",
                Some(Path::new(path)),
            ));
        }
    }

    if let Some(orphan_arr) = usage_report_json.get("orphans").and_then(|v| v.as_array()) {
        let report_orphans = orphan_arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let computed_orphan_set = effective_orphans.into_iter().collect::<BTreeSet<_>>();
        if report_orphans != computed_orphan_set {
            violations.push(violation(
                "OPS_FILE_USAGE_REPORT_ORPHAN_MISMATCH",
                "ops/_generated.example/file-usage-report.json orphan list is stale".to_string(),
                "regenerate and commit file-usage-report.json after updating ops artifacts",
                Some(usage_report_rel),
            ));
        }
    }

    let registry_budget = BTreeMap::from([
        ("inventory".to_string(), 35usize),
        ("load".to_string(), 70usize),
        ("observe".to_string(), 50usize),
        ("k8s".to_string(), 45usize),
        ("datasets".to_string(), 30usize),
        ("e2e".to_string(), 20usize),
        ("stack".to_string(), 20usize),
        ("env".to_string(), 10usize),
        ("report".to_string(), 10usize),
        ("schema".to_string(), 120usize),
    ]);
    for (domain, count) in registry_count_by_domain {
        if let Some(max) = registry_budget.get(&domain) {
            if count > *max {
                violations.push(violation(
                    "OPS_REGISTRY_FILE_BUDGET_EXCEEDED",
                    format!(
                        "registry/config file budget exceeded for `{domain}`: {count} > {max}"
                    ),
                    "consolidate or remove registry/config files before adding new ones",
                    Some(Path::new("ops")),
                ));
            }
        }
    }
    let generated_budget = BTreeMap::from([
        ("_generated.example".to_string(), 20usize),
        ("stack".to_string(), 10usize),
        ("report".to_string(), 10usize),
        ("k8s".to_string(), 10usize),
        ("datasets".to_string(), 10usize),
        ("load".to_string(), 10usize),
        ("schema".to_string(), 10usize),
        ("e2e".to_string(), 10usize),
        ("observe".to_string(), 10usize),
    ]);
    for (domain, count) in generated_count_by_domain {
        if let Some(max) = generated_budget.get(&domain) {
            if count > *max {
                violations.push(violation(
                    "OPS_GENERATED_FILE_BUDGET_EXCEEDED",
                    format!("generated file budget exceeded for `{domain}`: {count} > {max}"),
                    "consolidate generated outputs and avoid adding redundant generated artifacts",
                    Some(Path::new("ops")),
                ));
            }
        }
    }

    Ok(violations)
}

pub(super) fn check_ops_docs_governance(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let forbidden_transitional_tokens = ["phase", "task"];
    for root in ["ops", "docs"] {
        for file in walk_files(&ctx.repo_root.join(root)) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            let has_forbidden_segment = rel
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .any(|segment| {
                    let lowercase = segment.to_ascii_lowercase();
                    forbidden_transitional_tokens.iter().any(|token| {
                        lowercase == *token
                            || lowercase.starts_with(&format!("{token}-"))
                            || lowercase.ends_with(&format!("-{token}"))
                            || lowercase.contains(&format!("-{token}-"))
                            || lowercase.starts_with(&format!("{token}_"))
                            || lowercase.ends_with(&format!("_{token}"))
                            || lowercase.contains(&format!("_{token}_"))
                    })
                });
            if has_forbidden_segment {
                violations.push(violation(
                    "OPS_NAMING_TRANSITIONAL_TOKEN_FORBIDDEN",
                    format!(
                        "path uses transitional naming token (`phase`/`task`): `{rel_str}`"
                    ),
                    "rename files/directories to durable intent-based names",
                    Some(rel),
                ));
            }
        }
    }

    let domain_dirs = [
        "ops/datasets",
        "ops/e2e",
        "ops/k8s",
        "ops/load",
        "ops/observe",
        "ops/report",
        "ops/stack",
        "ops/env",
        "ops/inventory",
        "ops/schema",
    ];
    for domain in domain_dirs {
        let index_rel = Path::new(domain).join("INDEX.md");
        if ctx.adapters.fs.exists(ctx.repo_root, &index_rel) {
            let index_text = fs::read_to_string(ctx.repo_root.join(&index_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            for line in index_text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if !trimmed.starts_with("- ") {
                    violations.push(violation(
                        "OPS_DOC_INDEX_NON_LINK_CONTENT",
                        format!(
                            "domain index must be links-only; found non-link content in `{}`: `{trimmed}`",
                            index_rel.display()
                        ),
                        "keep domain INDEX.md files links-only with headings and bullet links",
                        Some(&index_rel),
                    ));
                }
            }

            for required_doc in ["README.md", "CONTRACT.md", "REQUIRED_FILES.md", "OWNER.md"] {
                let doc_rel = Path::new(domain).join(required_doc);
                if ctx.adapters.fs.exists(ctx.repo_root, &doc_rel)
                    && !index_text.contains(required_doc)
                {
                    violations.push(violation(
                        "OPS_DOC_INDEX_REQUIRED_LINK_MISSING",
                        format!(
                            "domain index `{}` must link `{}`",
                            index_rel.display(),
                            doc_rel.display()
                        ),
                        "add README.md and CONTRACT.md links to domain INDEX.md when files exist",
                        Some(&index_rel),
                    ));
                }
            }
        }

        let readme_rel = Path::new(domain).join("README.md");
        if ctx.adapters.fs.exists(ctx.repo_root, &readme_rel) {
            let readme_text = fs::read_to_string(ctx.repo_root.join(&readme_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let line_count = readme_text.lines().count();
            if line_count > 30 {
                violations.push(violation(
                    "OPS_DOC_README_SIZE_BUDGET_EXCEEDED",
                    format!(
                        "domain README exceeds 30 line budget: `{}` has {} lines",
                        readme_rel.display(),
                        line_count
                    ),
                    "keep domain README focused on what it is and where to start within 30 lines",
                    Some(&readme_rel),
                ));
            }
        }
    }

    let reference_index_rel = Path::new("docs/operations/ops-system/INDEX.md");
    let reference_index_text = fs::read_to_string(ctx.repo_root.join(reference_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let docs_root = ctx.repo_root.join("docs/operations/ops-system");
    for doc in walk_files(&docs_root) {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        if rel.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = rel.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name == "INDEX.md" {
            continue;
        }
        if !reference_index_text.contains(&format!("({name})")) {
            violations.push(violation(
                "OPS_REPORT_DOC_ORPHAN",
                format!(
                    "ops doc `{}` is not linked from docs/operations/ops-system/INDEX.md",
                    rel.display()
                ),
                "add doc link to docs/operations/ops-system/INDEX.md or remove orphan ops-system doc",
                Some(reference_index_rel),
            ));
        }
    }
    for target in markdown_link_targets(&reference_index_text) {
        let rel = Path::new("docs/operations/ops-system").join(&target);
        if !ctx.adapters.fs.exists(ctx.repo_root, &rel) {
            violations.push(violation(
                "OPS_REPORT_DOC_REFERENCE_BROKEN_LINK",
                format!(
                    "docs/operations/ops-system/INDEX.md links missing ops doc `{}`",
                    rel.display()
                ),
                "fix broken docs links in docs/operations/ops-system/INDEX.md",
                Some(reference_index_rel),
            ));
        }
    }

    let control_plane_rel = Path::new("ops/CONTROL_PLANE.md");
    let control_plane_snapshot_rel = Path::new("ops/_generated.example/control-plane.snapshot.md");
    let control_plane_drift_rel = Path::new("ops/_generated.example/control-plane-drift-report.json");
    let control_plane_surface_list_rel =
        Path::new("ops/_generated.example/control-plane-surface-list.json");
    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_snapshot_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_SNAPSHOT_MISSING",
            format!(
                "missing control-plane snapshot `{}`",
                control_plane_snapshot_rel.display()
            ),
            "generate and commit control-plane snapshot for drift checks",
            Some(control_plane_snapshot_rel),
        ));
    } else {
        let current = fs::read_to_string(ctx.repo_root.join(control_plane_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let snapshot = fs::read_to_string(ctx.repo_root.join(control_plane_snapshot_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if current != snapshot {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SNAPSHOT_DRIFT",
                "ops/CONTROL_PLANE.md does not match ops/_generated.example/control-plane.snapshot.md"
                    .to_string(),
                "refresh control-plane snapshot to match current control-plane contract",
                Some(control_plane_snapshot_rel),
            ));
        }
        for line in current.lines() {
            let lower = line.to_ascii_lowercase();
            if (lower.contains("example") || lower.contains("examples")) || !line.contains("bijux-")
            {
                continue;
            }
            violations.push(violation(
                "OPS_CONTROL_PLANE_CRATE_LIST_FORBIDDEN",
                format!(
                    "ops/CONTROL_PLANE.md contains crate reference outside example context: `{}`",
                    line.trim()
                ),
                "keep ops/CONTROL_PLANE.md policy-only; move current crate inventory to ops/_generated.example/control-plane.snapshot.md",
                Some(control_plane_rel),
            ));
            break;
        }
    }

    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_drift_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_DRIFT_REPORT_MISSING",
            format!(
                "missing control-plane drift report `{}`",
                control_plane_drift_rel.display()
            ),
            "generate and commit control-plane drift report artifact",
            Some(control_plane_drift_rel),
        ));
    } else {
        let drift_text = fs::read_to_string(ctx.repo_root.join(control_plane_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let drift_json: serde_json::Value =
            serde_json::from_str(&drift_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        if drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_CONTROL_PLANE_DRIFT_REPORT_BLOCKING",
                "control-plane-drift-report.json status is not `pass`".to_string(),
                "resolve control-plane drift and regenerate control-plane-drift-report.json",
                Some(control_plane_drift_rel),
            ));
        }
        let has_surface_check = drift_json
            .get("checks")
            .and_then(|v| v.as_array())
            .map(|checks| {
                checks.iter().any(|item| {
                    item.get("id").and_then(|v| v.as_str())
                        == Some("control-plane-surface-list-present")
                        && item.get("status").and_then(|v| v.as_str()) == Some("pass")
                })
            })
            .unwrap_or(false);
        if !has_surface_check {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SURFACE_LIST_CHECK_MISSING",
                "control-plane-drift-report.json must include passing `control-plane-surface-list-present` check"
                    .to_string(),
                "regenerate control-plane drift report with control-plane surface-list status",
                Some(control_plane_drift_rel),
            ));
        }
    }

    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_surface_list_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_SURFACE_LIST_MISSING",
            format!(
                "missing control-plane surface list report `{}`",
                control_plane_surface_list_rel.display()
            ),
            "generate and commit control-plane-surface-list report",
            Some(control_plane_surface_list_rel),
        ));
    } else {
        let surface_text = fs::read_to_string(ctx.repo_root.join(control_plane_surface_list_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let surface_json: serde_json::Value = serde_json::from_str(&surface_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if surface_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SURFACE_LIST_BLOCKING",
                "control-plane-surface-list.json status is not `pass`".to_string(),
                "resolve control-plane surface-list drift and regenerate the report",
                Some(control_plane_surface_list_rel),
            ));
        }
        let surfaces = surface_json
            .get("surfaces")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let expected = ["check", "docs", "configs", "ops"];
        for required in expected {
            if !surfaces
                .iter()
                .any(|value| value.as_str() == Some(required))
            {
                violations.push(violation(
                    "OPS_CONTROL_PLANE_SURFACE_LIST_INCOMPLETE",
                    format!(
                        "control-plane-surface-list.json missing required surface `{required}`"
                    ),
                    "regenerate control-plane surface list report from command ownership source",
                    Some(control_plane_surface_list_rel),
                ));
            }
        }
    }

    let docs_drift_rel = Path::new("ops/_generated.example/docs-drift-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, docs_drift_rel) {
        violations.push(violation(
            "OPS_DOCS_DRIFT_ARTIFACT_MISSING",
            format!("missing docs drift artifact `{}`", docs_drift_rel.display()),
            "generate and commit docs drift report artifact",
            Some(docs_drift_rel),
        ));
    } else {
        let docs_drift_text = fs::read_to_string(ctx.repo_root.join(docs_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let docs_drift_json: serde_json::Value = serde_json::from_str(&docs_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if docs_drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_DOCS_DRIFT_REPORT_BLOCKING",
                "docs-drift-report.json status is not `pass`".to_string(),
                "resolve docs drift and regenerate docs-drift-report.json",
                Some(docs_drift_rel),
            ));
        }
        if let Some(checks) = docs_drift_json.get("checks").and_then(|v| v.as_array()) {
            for check in checks {
                if check.get("status").and_then(|v| v.as_str()) != Some("pass") {
                    let id = check
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    violations.push(violation(
                        "OPS_DOCS_DRIFT_CHECK_BLOCKING",
                        format!("docs-drift-report check `{id}` is not pass"),
                        "fix the failing docs drift check and regenerate docs-drift-report.json",
                        Some(docs_drift_rel),
                    ));
                }
            }
        }
    }

    let forbidden_doc_refs = [
        "ops/schema/obs/",
        "ops/obs/",
        "ops/k8s/Makefile",
        "ops/load/k6/manifests/suites.json",
        "ops/load/k6/thresholds/",
    ];
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if rel.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&file).map_err(|err| CheckError::Failed(err.to_string()))?;
        for forbidden in forbidden_doc_refs {
            if text.contains(forbidden) {
                violations.push(violation(
                    "OPS_DOC_FORBIDDEN_PATH_REFERENCE",
                    format!(
                        "doc `{}` references retired or forbidden path `{forbidden}`",
                        rel.display()
                    ),
                    "replace with current canonical path and remove retired references",
                    Some(rel),
                ));
            }
        }
        if text.contains("TODO") || text.contains("TBD") {
            violations.push(violation(
                "OPS_DOC_TODO_MARKER_FORBIDDEN",
                format!("doc `{}` contains TODO/TBD marker", rel.display()),
                "remove TODO/TBD markers from ops docs for release-ready contracts",
                Some(rel),
            ));
        }
        if !rel.starts_with("ops/_generated")
            && !rel.starts_with("ops/_generated.example")
            && !rel.starts_with("ops/schema/generated")
        {
            let lower = text.to_ascii_lowercase();
            if lower.contains("final crate set")
                || lower.contains("crate set (locked)")
                || lower.contains("final crate list")
            {
                violations.push(violation(
                    "OPS_STALE_LOCKED_LANGUAGE",
                    format!(
                        "authored ops markdown `{}` contains stale locked/final wording",
                        rel.display()
                    ),
                    "remove stale locked/final claims from authored ops docs and keep current-state lists in generated artifacts",
                    Some(rel),
                ));
            }
        }
    }

    let surfaces_text = fs::read_to_string(ctx.repo_root.join("ops/inventory/surfaces.json"))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_json: serde_json::Value =
        serde_json::from_str(&surfaces_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowed_commands = surfaces_json
        .get("bijux-dev-atlas_commands")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    for doc in walk_files(&ctx.repo_root.join("docs")) {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        if rel.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&doc).map_err(|err| CheckError::Failed(err.to_string()))?;
        for command in extract_ops_command_refs(&text) {
            if !allowed_commands.contains(&command) {
                violations.push(violation(
                    "OPS_DOC_COMMAND_SURFACE_UNKNOWN",
                    format!(
                        "doc `{}` references command not in surfaces.json: `{command}`",
                        rel.display()
                    ),
                    "replace stale command references with canonical surfaces.json commands",
                    Some(rel),
                ));
            }
        }
    }
    let ops_markdown_files = walk_files(&ctx.repo_root.join("ops"))
        .into_iter()
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    let ops_markdown_file_budget = 103usize;
    if ops_markdown_files.len() > ops_markdown_file_budget {
        violations.push(violation(
            "OPS_MARKDOWN_FILE_BUDGET_EXCEEDED",
            format!(
                "ops markdown file budget exceeded: {} > {}",
                ops_markdown_files.len(),
                ops_markdown_file_budget
            ),
            "reduce ops markdown sprawl or move handbook content into docs/",
            Some(Path::new("ops")),
        ));
    }
    let ops_markdown_line_budget = 2800usize;
    let mut ops_markdown_lines = 0usize;
    let allowed_standard_names = BTreeSet::from([
        "README.md".to_string(),
        "INDEX.md".to_string(),
        "CONTRACT.md".to_string(),
        "REQUIRED_FILES.md".to_string(),
        "OWNER.md".to_string(),
    ]);
    let allowed_nonstandard_paths = BTreeSet::from([
        "ops/CONTROL_PLANE.md".to_string(),
        "ops/DIRECTORY_BUDGET_POLICY.md".to_string(),
        "ops/DRIFT.md".to_string(),
        "ops/ERRORS.md".to_string(),
        "ops/GENERATED_LIFECYCLE.md".to_string(),
        "ops/NAMING.md".to_string(),
        "ops/SSOT.md".to_string(),
        "ops/_generated.example/INDEX.example.md".to_string(),
        "ops/_generated.example/MIRROR_POLICY.md".to_string(),
        "ops/_generated.example/control-plane.snapshot.md".to_string(),
        "ops/datasets/FIXTURE_LIFECYCLE.md".to_string(),
        "ops/load/evaluations/POLICY.md".to_string(),
        "ops/observe/drills/templates/incident-template.md".to_string(),
        "ops/schema/BUDGET_POLICY.md".to_string(),
        "ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md".to_string(),
        "ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md".to_string(),
        "ops/schema/VERSIONING_POLICY.md".to_string(),
        "ops/schema/generated/schema-index.md".to_string(),
        "ops/stack/dependencies.md".to_string(),
    ]);
    let ops_markdown_max_depth = 3usize;
    let depth_exception_paths = BTreeSet::from([
        "ops/k8s/charts/bijux-atlas/README.md".to_string(),
        "ops/load/k6/queries/INDEX.md".to_string(),
        "ops/observe/drills/templates/incident-template.md".to_string(),
    ]);
    let ops_markdown_domain_budget = BTreeMap::from([
        ("_generated".to_string(), 3usize),
        ("_generated.example".to_string(), 7usize),
        ("datasets".to_string(), 11usize),
        ("e2e".to_string(), 10usize),
        ("env".to_string(), 4usize),
        ("inventory".to_string(), 4usize),
        ("k8s".to_string(), 8usize),
        ("load".to_string(), 11usize),
        ("observe".to_string(), 9usize),
        ("report".to_string(), 8usize),
        ("schema".to_string(), 12usize),
        ("stack".to_string(), 8usize),
    ]);
    let mut ops_markdown_domain_counts = BTreeMap::<String, usize>::new();
    for doc in &ops_markdown_files {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        let rel_str = rel.display().to_string();
        let name = rel
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        let text = fs::read_to_string(doc).map_err(|err| CheckError::Failed(err.to_string()))?;
        ops_markdown_lines += text.lines().count();
        if let Ok(stripped) = rel.strip_prefix("ops") {
            let domain = stripped
                .components()
                .next()
                .and_then(|component| component.as_os_str().to_str())
                .map(ToString::to_string);
            if let Some(domain) = domain {
                *ops_markdown_domain_counts.entry(domain).or_insert(0) += 1;
            }
        }
        let depth = rel.components().count().saturating_sub(1);
        if depth > ops_markdown_max_depth && !depth_exception_paths.contains(&rel_str) {
            violations.push(violation(
                "OPS_MARKDOWN_DEPTH_BUDGET_EXCEEDED",
                format!(
                    "ops markdown depth exceeded: `{rel_str}` depth={} max={}",
                    depth, ops_markdown_max_depth
                ),
                "move deep ops markdown into docs/operations or add an explicit governance allowlist exception",
                Some(rel),
            ));
        }
        if rel.starts_with(Path::new("ops/report/docs/")) {
            if name != "README.md" && name != "REFERENCE_INDEX.md" {
                violations.push(violation(
                    "OPS_MARKDOWN_FILENAME_FORBIDDEN",
                    format!("non-canonical markdown file under redirect-only area: `{rel_str}`"),
                    "keep only redirect stubs under ops/report/docs or migrate docs to docs/operations",
                    Some(rel),
                ));
            }
        } else if !allowed_standard_names.contains(&name)
            && !allowed_nonstandard_paths.contains(&rel_str)
        {
            violations.push(violation(
                "OPS_MARKDOWN_FILENAME_FORBIDDEN",
                format!("non-canonical markdown file under ops: `{rel_str}`"),
                "rename to canonical doc filenames or add explicit governance allowlist entry",
                Some(rel),
            ));
        }
        for line in text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') && trimmed.to_ascii_lowercase().contains("how to") {
                violations.push(violation(
                    "OPS_MARKDOWN_HOW_TO_HEADING_FORBIDDEN",
                    format!("ops markdown contains workflow-style heading in `{rel_str}`"),
                    "move tutorial/workflow prose to docs/operations and keep ops markdown normative",
                    Some(rel),
                ));
                break;
            }
            let line_lower = trimmed.to_ascii_lowercase();
            let is_markdown_link_line = trimmed.starts_with("- [") && trimmed.contains("](");
            if line_lower.contains("how to") && !is_markdown_link_line {
                violations.push(violation(
                    "OPS_MARKDOWN_HOW_TO_PHRASE_FORBIDDEN",
                    format!("ops markdown contains \"How to\" prose in `{rel_str}`"),
                    "move workflow prose to docs/operations and keep ops markdown normative",
                    Some(rel),
                ));
                break;
            }
        }
        for command in extract_ops_command_refs(&text) {
            if !allowed_commands.contains(&command) {
                violations.push(violation(
                    "OPS_MARKDOWN_COMMAND_SURFACE_UNKNOWN",
                    format!(
                        "ops markdown `{}` references command not in surfaces.json: `{command}`",
                        rel.display()
                    ),
                    "replace stale command references with canonical surfaces.json commands",
                    Some(rel),
                ));
            }
        }
    }
    for (domain, max) in ops_markdown_domain_budget {
        let count = ops_markdown_domain_counts.get(&domain).copied().unwrap_or(0);
        if count > max {
            violations.push(violation(
                "OPS_MARKDOWN_DOMAIN_BUDGET_EXCEEDED",
                format!(
                    "ops markdown domain budget exceeded for `{domain}`: {count} > {max}"
                ),
                "move handbook-style docs out of ops domain surfaces and keep only canonical headers",
                Some(Path::new("ops")),
            ));
        }
    }
    if ops_markdown_lines > ops_markdown_line_budget {
        violations.push(violation(
            "OPS_MARKDOWN_LINE_BUDGET_EXCEEDED",
            format!(
                "ops markdown line budget exceeded: {} > {}",
                ops_markdown_lines, ops_markdown_line_budget
            ),
            "move handbook-style content into docs/ and keep ops markdown concise",
            Some(Path::new("ops")),
        ));
    }
    let mut seen_docs_dirs = BTreeSet::new();
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let mut parent = file.parent();
        while let Some(dir) = parent {
            let rel = dir.strip_prefix(ctx.repo_root).unwrap_or(dir);
            if rel == Path::new("ops/report/docs") {
                break;
            }
            if rel
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "docs")
                && seen_docs_dirs.insert(rel.to_path_buf())
            {
                violations.push(violation(
                    "OPS_DOCS_DIRECTORY_FORBIDDEN",
                    format!(
                        "forbidden ops docs subtree `{}`; ops docs must live under docs/operations",
                        rel.display()
                    ),
                    "remove ops/**/docs/** subtree or migrate docs into docs/operations",
                    Some(rel),
                ));
            }
            parent = dir.parent();
        }
    }

    let ops_index_rel = Path::new("ops/INDEX.md");
    let ops_index_text = fs::read_to_string(ctx.repo_root.join(ops_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for root_doc in [
        "CONTRACT.md",
        "CONTROL_PLANE.md",
        "DRIFT.md",
        "ERRORS.md",
        "NAMING.md",
        "README.md",
        "SSOT.md",
    ] {
        let rel = Path::new("ops").join(root_doc);
        if ctx.adapters.fs.exists(ctx.repo_root, &rel) && !ops_index_text.contains(root_doc) {
            violations.push(violation(
                "OPS_ROOT_DOC_INDEX_LINK_MISSING",
                format!(
                    "ops root document `{}` must be linked from `ops/INDEX.md`",
                    rel.display()
                ),
                "link all root ops docs from ops/INDEX.md",
                Some(ops_index_rel),
            ));
        }
    }
    let index_line_count = ops_index_text.lines().count();
    if index_line_count > 80 {
        violations.push(violation(
            "OPS_ROOT_INDEX_SIZE_BUDGET_EXCEEDED",
            format!(
                "ops/INDEX.md exceeds max line budget (80): {} lines",
                index_line_count
            ),
            "keep ops/INDEX.md compact and move details to linked docs",
            Some(ops_index_rel),
        ));
    }
    let root_doc_line_budgets = [
        ("ops/README.md", 80usize),
        ("ops/CONTRACT.md", 140usize),
        ("ops/CONTROL_PLANE.md", 80usize),
        ("ops/DRIFT.md", 80usize),
        ("ops/ERRORS.md", 80usize),
        ("ops/NAMING.md", 80usize),
        ("ops/SSOT.md", 80usize),
    ];
    for (rel_str, max_lines) in root_doc_line_budgets {
        let rel = Path::new(rel_str);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            continue;
        }
        let text = fs::read_to_string(ctx.repo_root.join(rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let line_count = text.lines().count();
        if line_count > max_lines {
            violations.push(violation(
                "OPS_ROOT_DOC_SIZE_BUDGET_EXCEEDED",
                format!(
                    "ops root doc exceeds line budget: `{}` has {} lines (max {})",
                    rel.display(),
                    line_count,
                    max_lines
                ),
                "keep root governance docs compact and move extended narrative into docs/",
                Some(rel),
            ));
        }
    }

    Ok(violations)
}

fn markdown_link_targets(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let mut cursor = line;
        while let Some(start) = cursor.find('(') {
            let after_start = &cursor[start + 1..];
            let Some(end) = after_start.find(')') else {
                break;
            };
            let target = &after_start[..end];
            if target.ends_with(".md") && !target.contains("://") {
                out.push(target.to_string());
            }
            cursor = &after_start[end + 1..];
        }
    }
    out
}

fn extract_ops_command_refs(content: &str) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    for line in content.lines() {
        let mut cursor = line;
        while let Some(pos) = cursor.find("bijux dev atlas ops ") {
            let after = &cursor[pos + "bijux dev atlas ops ".len()..];
            let mut tokens = Vec::new();
            for token in after.split_whitespace() {
                if token.starts_with("--")
                    || token.starts_with('`')
                    || token.starts_with('|')
                    || token.starts_with('(')
                {
                    break;
                }
                let clean = token
                    .trim_matches(|ch: char| ",.;:()[]`".contains(ch))
                    .to_string();
                if clean.is_empty() {
                    break;
                }
                if !clean
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                {
                    break;
                }
                tokens.push(clean);
                if tokens.len() >= 3 {
                    break;
                }
            }
            if !tokens.is_empty() {
                commands.insert(format!("bijux dev atlas ops {}", tokens.join(" ")));
            }
            cursor = after;
        }
    }
    commands
}

pub(super) fn check_ops_evidence_bundle_discipline(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let generated_lifecycle_rel = Path::new("ops/GENERATED_LIFECYCLE.md");
    let mirror_policy_rel = Path::new("ops/_generated.example/MIRROR_POLICY.md");
    let allowlist_rel = Path::new("ops/_generated.example/ALLOWLIST.json");
    let ops_index_rel = Path::new("ops/_generated.example/ops-index.json");
    let scorecard_rel = Path::new("ops/_generated.example/scorecard.json");
    let bundle_rel = Path::new("ops/_generated.example/ops-evidence-bundle.json");
    let contract_audit_rel = Path::new("ops/_generated.example/contract-audit-report.json");
    let contract_graph_rel = Path::new("ops/_generated.example/contract-dependency-graph.json");
    let control_graph_diff_rel =
        Path::new("ops/_generated.example/control-graph-diff-report.json");
    let schema_drift_rel = Path::new("ops/_generated.example/schema-drift-report.json");
    let gates_rel = Path::new("ops/inventory/gates.json");

    for rel in [
        generated_lifecycle_rel,
        mirror_policy_rel,
        allowlist_rel,
        ops_index_rel,
        scorecard_rel,
        bundle_rel,
        contract_audit_rel,
        contract_graph_rel,
        control_graph_diff_rel,
        schema_drift_rel,
    ] {
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_EVIDENCE_REQUIRED_ARTIFACT_MISSING",
                format!("missing required evidence artifact `{}`", rel.display()),
                "generate and commit required evidence artifact",
                Some(rel),
            ));
        }
    }

    let mirror_policy_text = fs::read_to_string(ctx.repo_root.join(mirror_policy_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let generated_lifecycle_text = fs::read_to_string(ctx.repo_root.join(generated_lifecycle_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for required in [
        "## Lifecycle Classes",
        "transient_runtime",
        "domain_derived",
        "curated_evidence",
        "## Retention Policy",
        "## Regeneration Triggers",
        "## Deterministic Ordering",
        "## Generator Contract Versioning",
    ] {
        if !generated_lifecycle_text.contains(required) {
            violations.push(violation(
                "OPS_GENERATED_LIFECYCLE_CONTRACT_INCOMPLETE",
                format!(
                    "generated lifecycle contract must include required section or marker `{required}`"
                ),
                "update ops/GENERATED_LIFECYCLE.md with complete lifecycle, retention, trigger, and versioning policy",
                Some(generated_lifecycle_rel),
            ));
        }
    }
    for required in [
        "ops-index.json",
        "ops-evidence-bundle.json",
        "scorecard.json",
        "ALLOWLIST.json",
        "contract-audit-report.json",
        "contract-dependency-graph.json",
        "inventory-index.json",
        "control-plane.snapshot.md",
        "control-graph-diff-report.json",
        "docs-drift-report.json",
        "schema-drift-report.json",
        "stack-drift-report.json",
        "ops/GENERATED_LIFECYCLE.md",
    ] {
        if !mirror_policy_text.contains(required) {
            violations.push(violation(
                "OPS_EVIDENCE_MIRROR_POLICY_INCOMPLETE",
                format!("mirror policy must declare mirrored artifact `{required}`"),
                "update MIRROR_POLICY.md mirrored artifact list",
                Some(mirror_policy_rel),
            ));
        }
    }

    let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_json: serde_json::Value =
        serde_json::from_str(&allowlist_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlisted_files = allowlist_json
        .get("allowed_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if allowlisted_files.is_empty() {
        violations.push(violation(
            "OPS_EVIDENCE_ALLOWLIST_EMPTY",
            "ops/_generated.example/ALLOWLIST.json must declare non-empty `allowed_files`"
                .to_string(),
            "populate ALLOWLIST.json with exact committed files allowed under ops/_generated.example",
            Some(allowlist_rel),
        ));
    }
    let generated_example_root = ctx.repo_root.join("ops/_generated.example");
    if generated_example_root.exists() {
        let mut seen_files = BTreeSet::new();
        for file in walk_files(&generated_example_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            seen_files.insert(rel_str.clone());
            if !allowlisted_files.contains(&rel_str) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_MISSING_FILE",
                    format!(
                        "committed file `{}` is not declared in ops/_generated.example/ALLOWLIST.json",
                        rel.display()
                    ),
                    "update ALLOWLIST.json when adding or removing curated evidence artifacts",
                    Some(allowlist_rel),
                ));
            }
            if is_binary_like_file(&file)? {
                violations.push(violation(
                    "OPS_EVIDENCE_BINARY_FORBIDDEN",
                    format!(
                        "binary file is forbidden under ops/_generated.example: `{}`",
                        rel.display()
                    ),
                    "keep _generated.example text-only curated evidence artifacts",
                    Some(rel),
                ));
            }
            if rel.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let file_name = rel
                    .file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or_default()
                    .to_string();
                let suffix_allowed = file_name.ends_with("-report.json")
                    || file_name.ends_with("-index.json")
                    || file_name.ends_with(".example.json")
                    || matches!(
                        file_name.as_str(),
                        "ALLOWLIST.json"
                            | "ops-ledger.json"
                            | "ops-index.json"
                            | "ops-evidence-bundle.json"
                            | "scorecard.json"
                            | "control-plane-surface-list.json"
                    );
                if !suffix_allowed {
                    violations.push(violation(
                        "OPS_EVIDENCE_FILENAME_SUFFIX_POLICY_VIOLATION",
                        format!(
                            "curated evidence json filename does not match suffix policy: `{}`",
                            rel.display()
                        ),
                        "use -report.json, -index.json, .example.json, or an approved canonical evidence filename",
                        Some(rel),
                    ));
                }
                let text =
                    fs::read_to_string(&file).map_err(|err| CheckError::Failed(err.to_string()))?;
                let json: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
                if json.get("schema_version").is_none() {
                    violations.push(violation(
                        "OPS_EVIDENCE_SCHEMA_VERSION_MISSING",
                        format!(
                            "curated evidence json `{}` must include schema_version",
                            rel.display()
                        ),
                        "add schema_version to curated evidence json artifact",
                        Some(rel),
                    ));
                }
                if json.get("generated_by").is_none() {
                    violations.push(violation(
                        "OPS_EVIDENCE_GENERATED_BY_MISSING",
                        format!(
                            "curated evidence json `{}` must include generated_by",
                            rel.display()
                        ),
                        "add generated_by to curated evidence json artifact",
                        Some(rel),
                    ));
                }
            }
        }
        for allowlisted in &allowlisted_files {
            let rel = Path::new(allowlisted);
            if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_STALE_ENTRY",
                    format!(
                        "allowlist entry points to missing curated artifact `{}`",
                        rel.display()
                    ),
                    "remove stale entry from ALLOWLIST.json or restore the artifact",
                    Some(allowlist_rel),
                ));
            }
            if !seen_files.contains(allowlisted) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_UNUSED_ENTRY",
                    format!(
                        "allowlist entry does not match a committed curated artifact `{}`",
                        rel.display()
                    ),
                    "keep ALLOWLIST.json aligned with committed files in ops/_generated.example",
                    Some(allowlist_rel),
                ));
            }
        }
    }

    let bundle_text = fs::read_to_string(ctx.repo_root.join(bundle_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let bundle_json: serde_json::Value =
        serde_json::from_str(&bundle_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "release",
        "status",
        "hashes",
        "gates",
        "pin_freeze_status",
    ] {
        if bundle_json.get(key).is_none() {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_REQUIRED_KEY_MISSING",
                format!("evidence bundle missing required key `{key}`"),
                "populate required evidence bundle key",
                Some(bundle_rel),
            ));
        }
    }

    if let Some(schema_index) = bundle_json
        .get("hashes")
        .and_then(|v| v.get("schema_index"))
        .and_then(|v| v.as_object())
    {
        let Some(path) = schema_index.get("path").and_then(|v| v.as_str()) else {
            return Ok(violations);
        };
        let Some(sha) = schema_index.get("sha256").and_then(|v| v.as_str()) else {
            return Ok(violations);
        };
        let path_rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, path_rel) {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_SCHEMA_INDEX_PATH_MISSING",
                format!("schema index path in evidence bundle does not exist: `{path}`"),
                "fix hashes.schema_index.path in evidence bundle",
                Some(bundle_rel),
            ));
        } else {
            let actual_sha = sha256_hex(&ctx.repo_root.join(path_rel))?;
            if actual_sha != sha {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_SCHEMA_INDEX_HASH_DRIFT",
                    "schema index hash in evidence bundle is stale".to_string(),
                    "refresh hashes.schema_index.sha256 in ops-evidence-bundle.json",
                    Some(bundle_rel),
                ));
            }
        }
    }
    if let Some(inventory_hashes) = bundle_json
        .get("hashes")
        .and_then(|v| v.get("inventory"))
        .and_then(|v| v.as_array())
    {
        let mut paths = Vec::new();
        for entry in inventory_hashes {
            let path = entry
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let sha = entry
                .get("sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if !path.starts_with("ops/") {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_PATH_INVALID",
                    format!("inventory hash entry path must live under ops/: `{path}`"),
                    "set hashes.inventory[].path to canonical ops paths only",
                    Some(bundle_rel),
                ));
            }
            if !sha.chars().all(|c| c.is_ascii_hexdigit()) || sha.len() != 64 {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_INVALID",
                    format!("inventory hash entry must be 64 lowercase hex chars for `{path}`"),
                    "refresh hashes.inventory sha256 values from deterministic artifact hashing",
                    Some(bundle_rel),
                ));
            }
            paths.push(path);
        }
        let mut sorted = paths.clone();
        sorted.sort();
        if paths != sorted {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_ORDER_NONDETERMINISTIC",
                "hashes.inventory must be sorted by path for deterministic output".to_string(),
                "sort hashes.inventory entries lexicographically by path",
                Some(bundle_rel),
            ));
        }
    }

    let schema_drift_text = fs::read_to_string(ctx.repo_root.join(schema_drift_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let schema_drift_json: serde_json::Value = serde_json::from_str(&schema_drift_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "generated_by",
        "status",
        "summary",
        "drift",
    ] {
        if schema_drift_json.get(key).is_none() {
            violations.push(violation(
                "OPS_SCHEMA_DRIFT_REPORT_INVALID",
                format!("schema drift report is missing required key `{key}`"),
                "populate schema drift report with required governance keys",
                Some(schema_drift_rel),
            ));
        }
    }

    let contract_audit_text = fs::read_to_string(ctx.repo_root.join(contract_audit_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contract_audit_json: serde_json::Value = serde_json::from_str(&contract_audit_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "generated_by",
        "status",
        "summary",
        "contracts",
    ] {
        if contract_audit_json.get(key).is_none() {
            violations.push(violation(
                "OPS_CONTRACT_AUDIT_REPORT_INVALID",
                format!("contract audit report is missing required key `{key}`"),
                "populate contract-audit-report.json with required governance keys",
                Some(contract_audit_rel),
            ));
        }
    }

    let contract_graph_text = fs::read_to_string(ctx.repo_root.join(contract_graph_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contract_graph_json: serde_json::Value = serde_json::from_str(&contract_graph_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in ["schema_version", "generated_by", "nodes", "edges"] {
        if contract_graph_json.get(key).is_none() {
            violations.push(violation(
                "OPS_CONTRACT_DEPENDENCY_GRAPH_INVALID",
                format!("contract dependency graph is missing required key `{key}`"),
                "populate contract-dependency-graph.json with nodes and dependency edges",
                Some(contract_graph_rel),
            ));
        }
    }

    let gates_text = fs::read_to_string(ctx.repo_root.join(gates_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let gates_json: serde_json::Value =
        serde_json::from_str(&gates_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected_gates = gates_json
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let bundle_gates = bundle_json
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if expected_gates != bundle_gates {
        violations.push(violation(
            "OPS_EVIDENCE_BUNDLE_GATE_LIST_DRIFT",
            format!(
                "evidence bundle gates mismatch: expected={expected_gates:?} actual={bundle_gates:?}"
            ),
            "synchronize evidence bundle gates list with ops/inventory/gates.json",
            Some(bundle_rel),
        ));
    }

    let generated_root = ctx.repo_root.join("ops/_generated");
    if generated_root.exists() {
        let allowed = BTreeSet::from([
            "ops/_generated/.gitkeep".to_string(),
            "ops/_generated/OWNER.md".to_string(),
            "ops/_generated/README.md".to_string(),
            "ops/_generated/REQUIRED_FILES.md".to_string(),
        ]);
        for file in walk_files(&generated_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            if !allowed.contains(&rel_str) {
                let is_json = rel.extension().and_then(|v| v.to_str()) == Some("json");
                if is_json {
                    violations.push(violation(
                        "OPS_GENERATED_RUNTIME_JSON_COMMITTED_FORBIDDEN",
                        format!(
                            "runtime json evidence must not be committed under ops/_generated: `{}`",
                            rel.display()
                        ),
                        "delete committed runtime json and regenerate into runtime-only ignored outputs",
                        Some(rel),
                    ));
                }
                violations.push(violation(
                    "OPS_GENERATED_DIRECTORY_COMMITTED_EVIDENCE_FORBIDDEN",
                    format!("ops/_generated contains unexpected committed file `{}`", rel.display()),
                    "keep ops/_generated to marker docs only; store curated evidence under ops/_generated.example",
                    Some(rel),
                ));
            }
        }
    }

    Ok(violations)
}

pub(super) fn check_ops_fixture_governance(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let manifest_rel = Path::new("ops/datasets/manifest.json");
    let manifest_dataset_ids = if ctx.adapters.fs.exists(ctx.repo_root, manifest_rel) {
        let manifest_text = fs::read_to_string(ctx.repo_root.join(manifest_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let manifest_json: serde_json::Value = serde_json::from_str(&manifest_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        manifest_json
            .get("datasets")
            .and_then(|v| v.as_array())
            .map(|datasets| {
                datasets
                    .iter()
                    .filter_map(|entry| entry.get("id").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default()
    } else {
        BTreeSet::new()
    };

    let consumer_list_rel = Path::new("ops/datasets/consumer-list.json");
    if ctx.adapters.fs.exists(ctx.repo_root, consumer_list_rel) {
        let consumer_list_text = fs::read_to_string(ctx.repo_root.join(consumer_list_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let consumer_list_json: serde_json::Value = serde_json::from_str(&consumer_list_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if let Some(consumers) = consumer_list_json.get("consumers").and_then(|v| v.as_array()) {
            for consumer in consumers {
                let consumer_id = consumer
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown-consumer");
                for dataset_id in consumer
                    .get("dataset_ids")
                    .and_then(|v| v.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str())
                {
                    if !manifest_dataset_ids.contains(dataset_id) {
                        violations.push(violation(
                            "OPS_DATASET_CONSUMER_UNKNOWN_DATASET_ID",
                            format!(
                                "dataset consumer `{consumer_id}` references unknown dataset id `{dataset_id}`"
                            ),
                            "align ops/datasets/consumer-list.json dataset_ids with ops/datasets/manifest.json",
                            Some(consumer_list_rel),
                        ));
                    }
                }
            }
        }
    } else {
        violations.push(violation(
            "OPS_DATASET_CONSUMER_LIST_MISSING",
            format!("missing dataset consumer contract `{}`", consumer_list_rel.display()),
            "restore ops/datasets/consumer-list.json",
            Some(consumer_list_rel),
        ));
    }

    let freeze_policy_rel = Path::new("ops/datasets/freeze-policy.json");
    if ctx.adapters.fs.exists(ctx.repo_root, freeze_policy_rel) {
        let freeze_policy_text = fs::read_to_string(ctx.repo_root.join(freeze_policy_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let freeze_policy_json: serde_json::Value = serde_json::from_str(&freeze_policy_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in [
            "schema_version",
            "freeze_mode",
            "immutability",
            "retention",
            "change_controls",
        ] {
            if freeze_policy_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_DATASET_FREEZE_POLICY_FIELD_MISSING",
                    format!("freeze policy missing required key `{key}`"),
                    "add missing required keys to ops/datasets/freeze-policy.json",
                    Some(freeze_policy_rel),
                ));
            }
        }
    } else {
        violations.push(violation(
            "OPS_DATASET_FREEZE_POLICY_MISSING",
            format!("missing dataset freeze policy `{}`", freeze_policy_rel.display()),
            "restore ops/datasets/freeze-policy.json",
            Some(freeze_policy_rel),
        ));
    }

    let fixture_policy_rel = Path::new("ops/datasets/fixture-policy.json");
    let mut allowed_binary_paths = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, fixture_policy_rel) {
        let policy_text = fs::read_to_string(ctx.repo_root.join(fixture_policy_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let policy_json: serde_json::Value = serde_json::from_str(&policy_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in [
            "schema_version",
            "allow_remote_download",
            "fixture_roots",
            "allowed_kinds",
            "allowed_binary_paths",
            "policy",
        ] {
            if policy_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_FIXTURE_POLICY_FIELD_MISSING",
                    format!("fixture policy missing required key `{key}`"),
                    "add missing required fixture policy key",
                    Some(fixture_policy_rel),
                ));
            }
        }
        let configured = policy_json
            .get("allowed_binary_paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        allowed_binary_paths.extend(configured);
    } else {
        violations.push(violation(
            "OPS_FIXTURE_POLICY_MISSING",
            format!(
                "missing fixture policy file `{}`",
                fixture_policy_rel.display()
            ),
            "restore ops/datasets/fixture-policy.json",
            Some(fixture_policy_rel),
        ));
    }

    let fixtures_root = ctx.repo_root.join("ops/datasets/fixtures");
    if fixtures_root.exists() {
        let allowed_root_docs = BTreeSet::from([
            "ops/datasets/fixtures/README.md".to_string(),
            "ops/datasets/fixtures/CONTRACT.md".to_string(),
            "ops/datasets/fixtures/INDEX.md".to_string(),
            "ops/datasets/fixtures/OWNER.md".to_string(),
        ]);
        for file in walk_files(&fixtures_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            if allowed_root_docs.contains(&rel_str) {
                continue;
            }
            if rel_str.contains("/assets/")
                && rel_str.contains("/v")
                && !rel_str.ends_with(".tar.gz")
                && rel_str.starts_with("ops/datasets/fixtures/")
            {
                violations.push(violation(
                    "OPS_FIXTURE_VERSION_ASSET_TARBALL_REQUIRED",
                    format!(
                        "fixture version assets must be .tar.gz archives: `{}`",
                        rel.display()
                    ),
                    "keep version asset payloads under assets/ with .tar.gz extension",
                    Some(rel),
                ));
            }
            if is_binary_like_file(&file)?
                && !rel_str.ends_with(".tar.gz")
                && !allowed_binary_paths.contains(&rel_str)
            {
                violations.push(violation(
                    "OPS_FIXTURE_BINARY_POLICY_VIOLATION",
                    format!(
                        "binary fixture file is not allowlisted and not a fixture tarball: `{}`",
                        rel.display()
                    ),
                    "allowlist the binary in fixture-policy.json or replace with a tarball fixture asset",
                    Some(rel),
                ));
            }
        }

        for entry in
            fs::read_dir(&fixtures_root).map_err(|err| CheckError::Failed(err.to_string()))?
        {
            let entry = entry.map_err(|err| CheckError::Failed(err.to_string()))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            if name == "." || name == ".." {
                continue;
            }
            let mut has_version_dir = false;
            for child in fs::read_dir(&path).map_err(|err| CheckError::Failed(err.to_string()))? {
                let child = child.map_err(|err| CheckError::Failed(err.to_string()))?;
                let child_path = child.path();
                let Some(child_name) = child_path.file_name().and_then(|v| v.to_str()) else {
                    continue;
                };
                if child_path.is_dir() && child_name.starts_with('v') {
                    has_version_dir = true;
                } else if child_path.is_file() {
                    let rel = child_path
                        .strip_prefix(ctx.repo_root)
                        .unwrap_or(child_path.as_path());
                    violations.push(violation(
                        "OPS_FIXTURE_LOOSE_FILE_FORBIDDEN",
                        format!(
                            "fixture family `{name}` has loose file outside versioned subtree: `{}`",
                            rel.display()
                        ),
                        "place fixture files under versioned directories like v1/",
                        Some(rel),
                    ));
                }
            }
            if !has_version_dir {
                let rel = path.strip_prefix(ctx.repo_root).unwrap_or(path.as_path());
                violations.push(violation(
                    "OPS_FIXTURE_VERSION_DIRECTORY_MISSING",
                    format!(
                        "fixture family `{name}` must contain versioned directories (v1, v2, ...)"
                    ),
                    "create versioned fixture subdirectory and move fixture payloads into it",
                    Some(rel),
                ));
            }
        }

        for manifest in walk_files(&fixtures_root)
            .into_iter()
            .filter(|p| p.file_name().and_then(|v| v.to_str()) == Some("manifest.lock"))
        {
            let manifest_rel = manifest
                .strip_prefix(ctx.repo_root)
                .unwrap_or(manifest.as_path());
            let content =
                fs::read_to_string(&manifest).map_err(|err| CheckError::Failed(err.to_string()))?;
            let mut archive_name = None::<String>;
            let mut sha256 = None::<String>;
            for line in content.lines() {
                if let Some(v) = line.strip_prefix("archive=") {
                    archive_name = Some(v.trim().to_string());
                }
                if let Some(v) = line.strip_prefix("sha256=") {
                    sha256 = Some(v.trim().to_string());
                }
            }
            let Some(archive_name) = archive_name else {
                violations.push(violation(
                    "OPS_FIXTURE_MANIFEST_ARCHIVE_MISSING",
                    format!(
                        "manifest lock missing archive= entry: `{}`",
                        manifest_rel.display()
                    ),
                    "add archive=<filename> to fixture manifest.lock",
                    Some(manifest_rel),
                ));
                continue;
            };
            let Some(expected_sha) = sha256 else {
                violations.push(violation(
                    "OPS_FIXTURE_MANIFEST_SHA_MISSING",
                    format!(
                        "manifest lock missing sha256= entry: `{}`",
                        manifest_rel.display()
                    ),
                    "add sha256=<digest> to fixture manifest.lock",
                    Some(manifest_rel),
                ));
                continue;
            };
            let version_dir = manifest
                .parent()
                .ok_or_else(|| CheckError::Failed("manifest.lock parent not found".to_string()))?;
            let tarball_path = version_dir.join("assets").join(&archive_name);
            let tarball_rel = tarball_path
                .strip_prefix(ctx.repo_root)
                .unwrap_or(tarball_path.as_path());
            if !tarball_path.exists() {
                violations.push(violation(
                    "OPS_FIXTURE_TARBALL_MISSING",
                    format!(
                        "fixture tarball declared by manifest.lock is missing: `{}`",
                        tarball_rel.display()
                    ),
                    "restore tarball under versioned assets/ directory",
                    Some(manifest_rel),
                ));
                continue;
            }
            let actual_sha = sha256_hex(&tarball_path)?;
            if actual_sha != expected_sha {
                violations.push(violation(
                    "OPS_FIXTURE_TARBALL_HASH_MISMATCH",
                    format!(
                        "fixture tarball hash mismatch for `{}`: expected={} actual={}",
                        tarball_rel.display(),
                        expected_sha,
                        actual_sha
                    ),
                    "refresh manifest.lock sha256 after tarball update",
                    Some(manifest_rel),
                ));
            }

            let src_dir = version_dir.join("src");
            if !src_dir.exists() || !src_dir.is_dir() {
                violations.push(violation(
                    "OPS_FIXTURE_SRC_DIRECTORY_MISSING",
                    format!(
                        "fixture version missing src/ directory: `{}`",
                        src_dir
                            .strip_prefix(ctx.repo_root)
                            .unwrap_or(src_dir.as_path())
                            .display()
                    ),
                    "add src/ copies for fixture version inputs",
                    Some(manifest_rel),
                ));
            }
            let has_queries = walk_files(version_dir).iter().any(|p| {
                p.file_name()
                    .and_then(|v| v.to_str())
                    .is_some_and(|n| n.contains("queries"))
            });
            let has_responses = walk_files(version_dir).iter().any(|p| {
                p.file_name()
                    .and_then(|v| v.to_str())
                    .is_some_and(|n| n.contains("responses"))
            });
            if !has_queries || !has_responses {
                violations.push(violation(
                    "OPS_FIXTURE_GOLDENS_MISSING",
                    format!(
                        "fixture version must include query/response goldens: `{}`",
                        version_dir
                            .strip_prefix(ctx.repo_root)
                            .unwrap_or(version_dir)
                            .display()
                    ),
                    "add *queries*.json and *responses*.json goldens in fixture version",
                    Some(manifest_rel),
                ));
            }
        }
    }

    let e2e_fixture_root = ctx.repo_root.join("ops/e2e/fixtures");
    let allowlist_rel = Path::new("ops/e2e/fixtures/allowlist.json");
    let lock_rel = Path::new("ops/e2e/fixtures/fixtures.lock");
    if e2e_fixture_root.exists() && ctx.adapters.fs.exists(ctx.repo_root, allowlist_rel) {
        let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let allowlist_json: serde_json::Value = serde_json::from_str(&allowlist_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let allowed_paths = allowlist_json
            .get("allowed_paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|i| i.as_str())
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let actual_fixture_paths = walk_files(&e2e_fixture_root)
            .into_iter()
            .filter_map(|p| {
                p.strip_prefix(ctx.repo_root)
                    .ok()
                    .map(|r| r.display().to_string())
            })
            .collect::<BTreeSet<_>>();
        for path in &actual_fixture_paths {
            if !allowed_paths.contains(path) {
                violations.push(violation(
                    "OPS_E2E_FIXTURE_ALLOWLIST_VIOLATION",
                    format!("e2e fixture file not allowlisted: `{path}`"),
                    "add fixture path to ops/e2e/fixtures/allowlist.json",
                    Some(allowlist_rel),
                ));
            }
        }
        for path in &allowed_paths {
            if !actual_fixture_paths.contains(path) {
                violations.push(violation(
                    "OPS_E2E_FIXTURE_ALLOWLIST_STALE_ENTRY",
                    format!("allowlist references missing e2e fixture file: `{path}`"),
                    "remove stale path from allowlist or restore file",
                    Some(allowlist_rel),
                ));
            }
        }

        if ctx.adapters.fs.exists(ctx.repo_root, lock_rel) {
            let lock_text = fs::read_to_string(ctx.repo_root.join(lock_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let lock_json: serde_json::Value = serde_json::from_str(&lock_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let expected = lock_json
                .get("allowlist_sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let actual = sha256_hex(&ctx.repo_root.join(allowlist_rel))?;
            if expected != actual {
                violations.push(violation(
                    "OPS_E2E_FIXTURE_LOCK_DRIFT",
                    "fixtures.lock allowlist_sha256 does not match allowlist.json".to_string(),
                    "update fixtures.lock allowlist_sha256 when allowlist changes",
                    Some(lock_rel),
                ));
            }
            let expected_inventory_sha = lock_json
                .get("fixture_inventory_sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let fixture_inventory_rel = Path::new("ops/datasets/generated/fixture-inventory.json");
            if ctx.adapters.fs.exists(ctx.repo_root, fixture_inventory_rel) {
                let actual_inventory_sha = sha256_hex(&ctx.repo_root.join(fixture_inventory_rel))?;
                if expected_inventory_sha != actual_inventory_sha {
                    violations.push(violation(
                        "OPS_E2E_FIXTURE_INVENTORY_LOCK_DRIFT",
                        "fixtures.lock fixture_inventory_sha256 does not match fixture-inventory.json"
                            .to_string(),
                        "update fixtures.lock fixture_inventory_sha256 when fixture inventory changes",
                        Some(lock_rel),
                    ));
                }
            }
        }
    }

    let suites_rel = Path::new("ops/e2e/suites/suites.json");
    let scenarios_rel = Path::new("ops/e2e/scenarios/scenarios.json");
    let expectations_rel = Path::new("ops/e2e/expectations/expectations.json");
    let scenario_slo_map_rel = Path::new("ops/inventory/scenario-slo-map.json");
    let mut canonical_e2e_scenario_ids = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, suites_rel) {
        let suites_text = fs::read_to_string(ctx.repo_root.join(suites_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let suites_json: serde_json::Value = serde_json::from_str(&suites_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let suite_ids = suites_json
            .get("suites")
            .and_then(|v| v.as_array())
            .map(|suites| {
                suites
                    .iter()
                    .filter_map(|suite| suite.get("id").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let scenario_ids = if ctx.adapters.fs.exists(ctx.repo_root, scenarios_rel) {
            let scenarios_text = fs::read_to_string(ctx.repo_root.join(scenarios_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let scenarios_json: serde_json::Value = serde_json::from_str(&scenarios_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            scenarios_json
                .get("scenarios")
                .and_then(|v| v.as_array())
                .map(|scenarios| {
                    scenarios
                        .iter()
                        .filter_map(|entry| entry.get("id").and_then(|v| v.as_str()))
                        .map(ToString::to_string)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default()
        } else {
            BTreeSet::new()
        };
        canonical_e2e_scenario_ids.extend(scenario_ids.iter().cloned());
        if ctx.adapters.fs.exists(ctx.repo_root, expectations_rel) {
            let expectations_text = fs::read_to_string(ctx.repo_root.join(expectations_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let expectations_json: serde_json::Value = serde_json::from_str(&expectations_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            for scenario_id in expectations_json
                .get("expectations")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.get("scenario_id").and_then(|v| v.as_str()))
            {
                if !suite_ids.contains(scenario_id) && !scenario_ids.contains(scenario_id) {
                    violations.push(violation(
                        "OPS_E2E_EXPECTATION_REFERENCE_MISSING",
                        format!(
                            "expectations.json scenario_id `{scenario_id}` is missing from suites/scenarios registries"
                        ),
                        "align expectations entries with canonical suite ids or scenario ids",
                        Some(expectations_rel),
                    ));
                }
            }
        }
        if let Some(suites) = suites_json.get("suites").and_then(|v| v.as_array()) {
            for suite in suites {
                let Some(id) = suite.get("id").and_then(|v| v.as_str()) else {
                    continue;
                };
                let maybe_fixture = if id.starts_with("fixture-") {
                    id.strip_prefix("fixture-")
                } else if id.ends_with("-fixture") {
                    id.strip_suffix("-fixture")
                } else {
                    None
                };
                if let Some(name) = maybe_fixture {
                    let fixture_dir = Path::new("ops/datasets/fixtures").join(name);
                    if !ctx.adapters.fs.exists(ctx.repo_root, &fixture_dir) {
                        violations.push(violation(
                            "OPS_E2E_FIXTURE_REFERENCE_MISSING",
                            format!(
                                "e2e suite `{id}` references missing fixture family `{}`",
                                fixture_dir.display()
                            ),
                            "create fixture family directory or rename e2e suite id",
                            Some(suites_rel),
                        ));
                    }
                }
            }
        }
    }

    let smoke_manifest_rel = Path::new("ops/e2e/manifests/smoke.manifest.json");
    if ctx.adapters.fs.exists(ctx.repo_root, smoke_manifest_rel) {
        let smoke_manifest_text = fs::read_to_string(ctx.repo_root.join(smoke_manifest_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let smoke_manifest_json: serde_json::Value = serde_json::from_str(&smoke_manifest_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let queries_lock_rel = Path::new("ops/e2e/smoke/queries.lock");
        if smoke_manifest_json
            .get("queries_lock")
            .and_then(|v| v.as_str())
            .is_none()
        {
            violations.push(violation(
                "OPS_E2E_SMOKE_MANIFEST_PINNED_QUERIES_MISSING",
                "smoke.manifest.json must define `queries_lock`".to_string(),
                "reference the pinned query lock file from smoke.manifest.json",
                Some(smoke_manifest_rel),
            ));
        }
        if let Some(queries_lock_ref) = smoke_manifest_json
            .get("queries_lock")
            .and_then(|v| v.as_str())
        {
            if queries_lock_ref != queries_lock_rel.display().to_string() {
                violations.push(violation(
                    "OPS_E2E_SMOKE_MANIFEST_PINNED_QUERIES_PATH_INVALID",
                    format!(
                        "smoke manifest queries_lock must be `{}`; found `{queries_lock_ref}`",
                        queries_lock_rel.display()
                    ),
                    "point smoke manifest queries_lock to ops/e2e/smoke/queries.lock",
                    Some(smoke_manifest_rel),
                ));
            }
            if !ctx.adapters.fs.exists(ctx.repo_root, queries_lock_rel) {
                violations.push(violation(
                    "OPS_E2E_SMOKE_QUERIES_LOCK_MISSING",
                    format!("missing pinned query lock file `{}`", queries_lock_rel.display()),
                    "restore ops/e2e/smoke/queries.lock",
                    Some(smoke_manifest_rel),
                ));
            }
        }
    }

    let load_suites_rel = Path::new("ops/load/suites/suites.json");
    let mut load_suite_names = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, load_suites_rel) {
        let load_suites_text = fs::read_to_string(ctx.repo_root.join(load_suites_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let load_suites_json: serde_json::Value = serde_json::from_str(&load_suites_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for suite in load_suites_json
            .get("suites")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let suite_name = suite
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            load_suite_names.insert(suite_name.to_string());
            if suite.get("kind").and_then(|v| v.as_str()) == Some("k6") {
                if let Some(scenario_name) = suite.get("scenario").and_then(|v| v.as_str()) {
                    let scenario_rel = Path::new("ops/load/scenarios").join(scenario_name);
                    if !ctx.adapters.fs.exists(ctx.repo_root, &scenario_rel) {
                        violations.push(violation(
                            "OPS_LOAD_SUITE_SCENARIO_MISSING",
                            format!(
                                "load suite `{suite_name}` references missing scenario `{}`",
                                scenario_rel.display()
                            ),
                            "add missing scenario file or fix suite scenario reference",
                            Some(load_suites_rel),
                        ));
                    }
                }
            }
            if let Some(threshold_file) = suite.get("threshold_file").and_then(|v| v.as_str()) {
                let threshold_rel = Path::new(threshold_file);
                if !ctx.adapters.fs.exists(ctx.repo_root, threshold_rel) {
                    violations.push(violation(
                        "OPS_LOAD_SUITE_THRESHOLD_FILE_MISSING",
                        format!(
                            "load suite `{suite_name}` references missing threshold file `{threshold_file}`"
                        ),
                        "add threshold file or remove stale threshold_file reference",
                        Some(load_suites_rel),
                    ));
                }
            }
        }
    }

    let slo_definitions_rel = Path::new("ops/observe/slo-definitions.json");
    let mut slo_ids = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, slo_definitions_rel) {
        let slo_text = fs::read_to_string(ctx.repo_root.join(slo_definitions_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let slo_json: serde_json::Value =
            serde_json::from_str(&slo_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        slo_ids = slo_json
            .get("slos")
            .and_then(|v| v.as_array())
            .map(|slos| {
                slos.iter()
                    .filter_map(|entry| entry.get("id").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
    }

    let drills_rel = Path::new("ops/inventory/drills.json");
    let mut drill_ids = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, drills_rel) {
        let drills_text = fs::read_to_string(ctx.repo_root.join(drills_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let drills_json: serde_json::Value = serde_json::from_str(&drills_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        drill_ids = drills_json
            .get("drills")
            .and_then(|v| v.as_array())
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
    }

    if ctx.adapters.fs.exists(ctx.repo_root, scenario_slo_map_rel) {
        let map_text = fs::read_to_string(ctx.repo_root.join(scenario_slo_map_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let map_json: serde_json::Value =
            serde_json::from_str(&map_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let map_entries = map_json
            .get("mappings")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut mapped_scenarios = BTreeSet::new();
        for entry in &map_entries {
            let scenario_id = entry
                .get("scenario_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if scenario_id.is_empty() {
                continue;
            }
            mapped_scenarios.insert(scenario_id.clone());

            for slo_id in entry
                .get("slo_ids")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                if !slo_ids.contains(slo_id) {
                    violations.push(violation(
                        "OPS_SCENARIO_SLO_MAP_UNKNOWN_SLO_ID",
                        format!(
                            "scenario-slo-map entry `{scenario_id}` references unknown slo id `{slo_id}`"
                        ),
                        "align scenario-slo-map slo_ids with ops/observe/slo-definitions.json",
                        Some(scenario_slo_map_rel),
                    ));
                }
            }

            for drill_id in entry
                .get("drill_ids")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                if !drill_ids.contains(drill_id) {
                    violations.push(violation(
                        "OPS_SCENARIO_SLO_MAP_UNKNOWN_DRILL_ID",
                        format!(
                            "scenario-slo-map entry `{scenario_id}` references unknown drill id `{drill_id}`"
                        ),
                        "align scenario-slo-map drill_ids with ops/inventory/drills.json",
                        Some(scenario_slo_map_rel),
                    ));
                }
            }

            for load_suite in entry
                .get("load_suites")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                if !load_suite_names.contains(load_suite) {
                    violations.push(violation(
                        "OPS_SCENARIO_SLO_MAP_UNKNOWN_LOAD_SUITE",
                        format!(
                            "scenario-slo-map entry `{scenario_id}` references unknown load suite `{load_suite}`"
                        ),
                        "align scenario-slo-map load_suites with ops/load/suites/suites.json",
                        Some(scenario_slo_map_rel),
                    ));
                }
            }
        }

        for scenario_id in &canonical_e2e_scenario_ids {
            if !mapped_scenarios.contains(scenario_id) {
                violations.push(violation(
                    "OPS_SCENARIO_SLO_MAP_MISSING_SCENARIO",
                    format!("e2e scenario `{scenario_id}` missing from scenario-slo-map"),
                    "add mapping entries for every e2e scenario in ops/inventory/scenario-slo-map.json",
                    Some(scenario_slo_map_rel),
                ));
            }
        }
    } else {
        violations.push(violation(
            "OPS_SCENARIO_SLO_MAP_MISSING",
            format!(
                "missing scenario to slo mapping contract `{}`",
                scenario_slo_map_rel.display()
            ),
            "restore ops/inventory/scenario-slo-map.json",
            Some(scenario_slo_map_rel),
        ));
    }

    let realdata_readme_rel = Path::new("ops/e2e/realdata/README.md");
    if ctx.adapters.fs.exists(ctx.repo_root, realdata_readme_rel) {
        let text = fs::read_to_string(ctx.repo_root.join(realdata_readme_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if !(text.to_lowercase().contains("example") && text.to_lowercase().contains("required")) {
            violations.push(violation(
                "OPS_E2E_REALDATA_SNAPSHOT_POLICY_MISSING",
                "realdata README must distinguish example snapshots from required fixtures"
                    .to_string(),
                "document example vs required snapshot policy in ops/e2e/realdata/README.md",
                Some(realdata_readme_rel),
            ));
        }
    }

    let fixture_inventory_rel = Path::new("ops/datasets/generated/fixture-inventory.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, fixture_inventory_rel) {
        violations.push(violation(
            "OPS_FIXTURE_INVENTORY_ARTIFACT_MISSING",
            format!(
                "missing fixture inventory generated artifact `{}`",
                fixture_inventory_rel.display()
            ),
            "generate and commit ops/datasets/generated/fixture-inventory.json",
            Some(fixture_inventory_rel),
        ));
    } else {
        let text = fs::read_to_string(ctx.repo_root.join(fixture_inventory_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let Some(fixtures) = json.get("fixtures").and_then(|v| v.as_array()) else {
            violations.push(violation(
                "OPS_FIXTURE_INVENTORY_SHAPE_INVALID",
                "fixture inventory must contain a fixtures array".to_string(),
                "populate fixtures array in fixture-inventory.json",
                Some(fixture_inventory_rel),
            ));
            return Ok(violations);
        };

        let mut indexed_versions = BTreeMap::new();
        for entry in fixtures {
            let Some(name) = entry.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(version) = entry.get("version").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(asset) = entry.get("asset").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(asset_sha) = entry.get("asset_sha256").and_then(|v| v.as_str()) else {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_HASH_MISSING",
                    format!("fixture inventory entry `{name}/{version}` is missing asset_sha256"),
                    "add asset_sha256 for each fixture inventory entry",
                    Some(fixture_inventory_rel),
                ));
                continue;
            };
            indexed_versions.insert(
                format!("{name}/{version}"),
                (asset.to_string(), asset_sha.to_string()),
            );
        }

        let mut discovered_versions = BTreeMap::new();
        for manifest in walk_files(&fixtures_root)
            .into_iter()
            .filter(|p| p.file_name().and_then(|v| v.to_str()) == Some("manifest.lock"))
        {
            let rel = manifest
                .strip_prefix(ctx.repo_root)
                .unwrap_or(manifest.as_path())
                .display()
                .to_string();
            let parts = rel.split('/').collect::<Vec<_>>();
            if parts.len() < 6 {
                continue;
            }
            let fixture_name = parts[3];
            let fixture_version = parts[4];
            let key = format!("{fixture_name}/{fixture_version}");
            let manifest_text =
                fs::read_to_string(&manifest).map_err(|err| CheckError::Failed(err.to_string()))?;
            let archive = manifest_text
                .lines()
                .find_map(|line| line.strip_prefix("archive="))
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            let manifest_sha = manifest_text
                .lines()
                .find_map(|line| line.strip_prefix("sha256="))
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            let asset =
                format!("ops/datasets/fixtures/{fixture_name}/{fixture_version}/assets/{archive}");
            let asset_path = ctx.repo_root.join(format!(
                "ops/datasets/fixtures/{fixture_name}/{fixture_version}/assets/{archive}"
            ));
            let asset_sha = if archive.is_empty() || !asset_path.exists() {
                String::new()
            } else {
                sha256_hex(&asset_path)?
            };
            if !manifest_sha.is_empty() && manifest_sha != asset_sha {
                violations.push(violation(
                    "OPS_FIXTURE_MANIFEST_SHA_STALE",
                    format!(
                        "manifest sha256 is stale for fixture `{key}`: manifest={} actual={}",
                        manifest_sha, asset_sha
                    ),
                    "refresh fixture manifest.lock sha256 after asset changes",
                    Some(Path::new(&rel)),
                ));
            }
            discovered_versions.insert(key, (asset, asset_sha));
        }

        for (key, (asset, sha)) in &discovered_versions {
            let Some((indexed_asset, indexed_sha)) = indexed_versions.get(key) else {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ENTRY_MISSING",
                    format!("fixture inventory missing entry for `{key}`"),
                    "add fixture version entry to ops/datasets/generated/fixture-inventory.json",
                    Some(fixture_inventory_rel),
                ));
                continue;
            };
            if indexed_asset != asset {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ASSET_PATH_DRIFT",
                    format!(
                        "fixture inventory asset path drift for `{key}`: expected `{asset}` got `{indexed_asset}`"
                    ),
                    "refresh fixture inventory asset paths from fixture manifests",
                    Some(fixture_inventory_rel),
                ));
            }
            if indexed_sha != sha {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ASSET_HASH_DRIFT",
                    format!(
                        "fixture inventory hash drift for `{key}`: expected `{sha}` got `{indexed_sha}`"
                    ),
                    "refresh fixture inventory hashes from fixture assets",
                    Some(fixture_inventory_rel),
                ));
            }
        }
        for key in indexed_versions.keys() {
            if !discovered_versions.contains_key(key) {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_STALE_ENTRY",
                    format!("fixture inventory has stale entry `{key}`"),
                    "remove stale fixture inventory entries not backed by fixture manifests",
                    Some(fixture_inventory_rel),
                ));
            }
        }
    }

    let fixture_drift_rel = Path::new("ops/_generated.example/fixture-drift-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, fixture_drift_rel) {
        violations.push(violation(
            "OPS_FIXTURE_DRIFT_REPORT_MISSING",
            format!(
                "missing fixture drift report artifact `{}`",
                fixture_drift_rel.display()
            ),
            "generate and commit fixture drift report under ops/_generated.example",
            Some(fixture_drift_rel),
        ));
    } else {
        let fixture_drift_text = fs::read_to_string(ctx.repo_root.join(fixture_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let fixture_drift_json: serde_json::Value = serde_json::from_str(&fixture_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in ["schema_version", "generated_by", "status", "summary", "drift"] {
            if fixture_drift_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_FIXTURE_DRIFT_REPORT_INVALID",
                    format!("fixture drift report is missing required key `{key}`"),
                    "populate fixture drift report with required governance keys",
                    Some(fixture_drift_rel),
                ));
            }
        }
        if !matches!(
            fixture_drift_json.get("status").and_then(|v| v.as_str()),
            Some("clean" | "pass")
        ) {
            violations.push(violation(
                "OPS_FIXTURE_DRIFT_REPORT_BLOCKING",
                "fixture drift report status must be `clean` or `pass`".to_string(),
                "resolve fixture drift and regenerate fixture-drift-report.json",
                Some(fixture_drift_rel),
            ));
        }
    }

    Ok(violations)
}

fn sha256_hex(path: &Path) -> Result<String, CheckError> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
}

fn is_binary_like_file(path: &Path) -> Result<bool, CheckError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let known_binary_ext = [
        "gz", "zip", "zst", "tar", "sqlite", "db", "bin", "png", "jpg", "jpeg",
    ];
    if known_binary_ext.contains(&ext.as_str()) {
        return Ok(true);
    }
    let bytes = fs::read(path).map_err(|err| CheckError::Failed(err.to_string()))?;
    if bytes.contains(&0) {
        return Ok(true);
    }
    Ok(std::str::from_utf8(&bytes).is_err())
}

struct RequiredFilesContract {
    required_files: Vec<PathBuf>,
    required_dirs: Vec<PathBuf>,
    forbidden_patterns: Vec<String>,
    notes: Vec<String>,
}

fn parse_required_files_markdown_yaml(
    content: &str,
    rel: &Path,
) -> Result<RequiredFilesContract, CheckError> {
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
        return Err(CheckError::Failed(format!(
            "{} must include a YAML contract block",
            rel.display()
        )));
    }
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&yaml_block).map_err(|err| CheckError::Failed(err.to_string()))?;
    let parsed_map = parsed.as_mapping().ok_or_else(|| {
        CheckError::Failed(format!(
            "{} YAML block must be a mapping with canonical keys",
            rel.display()
        ))
    })?;
    for key in [
        "required_files",
        "required_dirs",
        "forbidden_patterns",
        "notes",
    ] {
        if !parsed_map.contains_key(serde_yaml::Value::from(key)) {
            return Err(CheckError::Failed(format!(
                "{} must define `{key}` in REQUIRED_FILES contract YAML",
                rel.display()
            )));
        }
    }
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
    let required_dirs = parsed
        .get("required_dirs")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let forbidden_patterns = parsed
        .get("forbidden_patterns")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let notes = parsed
        .get("notes")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if required_files.is_empty() {
        return Err(CheckError::Failed(format!(
            "{} must define non-empty `required_files` YAML list",
            rel.display()
        )));
    }
    Ok(RequiredFilesContract {
        required_files,
        required_dirs,
        forbidden_patterns,
        notes,
    })
}

fn extract_ops_data_paths(text: &str) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for token in text.split_whitespace() {
        let trimmed = token
            .trim_matches(|c: char| {
                c == '`'
                    || c == '('
                    || c == ')'
                    || c == '['
                    || c == ']'
                    || c == ','
                    || c == ';'
                    || c == ':'
                    || c == '"'
                    || c == '\''
            })
            .to_string();
        if !trimmed.starts_with("ops/") {
            continue;
        }
        if trimmed.ends_with(".json")
            || trimmed.ends_with(".yaml")
            || trimmed.ends_with(".yml")
            || trimmed.ends_with(".toml")
        {
            refs.insert(trimmed);
        }
    }
    refs
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
    let deadline_line = text.lines().find(|line| {
        line.trim_start()
            .starts_with("- Legacy shell compatibility deadline: ")
    });
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
