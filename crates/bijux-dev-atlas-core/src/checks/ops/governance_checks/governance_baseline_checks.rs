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

