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

pub(super) fn check_ops_domain_contract_structure(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let contract_files = [
        "ops/datasets/CONTRACT.md",
        "ops/e2e/CONTRACT.md",
        "ops/k8s/CONTRACT.md",
        "ops/load/CONTRACT.md",
        "ops/observe/CONTRACT.md",
        "ops/report/CONTRACT.md",
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
                format!("domain contract `{}` must include `## Invariants`", rel.display()),
                "add an invariants section with explicit, enforceable rules",
                Some(rel),
            ));
            continue;
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
        if invariant_count < 6 {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_INVARIANT_COUNT_TOO_LOW",
                format!(
                    "domain contract `{}` must define at least 6 invariants; found {}",
                    rel.display(),
                    invariant_count
                ),
                "add concrete invariants until the minimum count is satisfied",
                Some(rel),
            ));
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
    let contracts_meta_rel = Path::new("ops/inventory/meta/contracts.json");
    let namespaces_rel = Path::new("ops/inventory/namespaces.json");
    let layers_rel = Path::new("ops/inventory/layers.json");
    let gates_rel = Path::new("ops/inventory/gates.json");
    let surfaces_rel = Path::new("ops/inventory/surfaces.json");
    let policy_rel = Path::new("ops/inventory/policies/dev-atlas-policy.json");
    let policy_schema_rel = Path::new("ops/inventory/policies/dev-atlas-policy.schema.json");
    let pins_rel = Path::new("ops/inventory/pins.yaml");
    let stack_manifest_rel = Path::new("ops/stack/generated/version-manifest.json");
    let registry_rel = Path::new("ops/inventory/registry.toml");
    let tools_rel = Path::new("ops/inventory/tools.toml");
    let inventory_index_rel = Path::new("ops/_generated.example/inventory-index.json");

    let contracts_map_text = fs::read_to_string(ctx.repo_root.join(contracts_map_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_map: serde_json::Value = serde_json::from_str(&contracts_map_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if contracts_map
        .get("authoritative")
        .and_then(|v| v.as_bool())
        != Some(true)
    {
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
        let schema = item.get("schema").and_then(|v| v.as_str()).unwrap_or("none");
        if schema != "none" && !ctx.adapters.fs.exists(ctx.repo_root, Path::new(schema)) {
            violations.push(violation(
                "OPS_INVENTORY_SCHEMA_REFERENCE_MISSING",
                format!(
                    "contracts-map references missing schema `{schema}` for `{path}`"
                ),
                "restore schema path or fix schema pointer in contracts-map",
                Some(contracts_map_rel),
            ));
        }
    }

    let contracts_text = fs::read_to_string(ctx.repo_root.join(contracts_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_json: serde_json::Value = serde_json::from_str(&contracts_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
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
    let contracts_meta_text = fs::read_to_string(ctx.repo_root.join(contracts_meta_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_meta_json: serde_json::Value = serde_json::from_str(&contracts_meta_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if contracts_meta_json
        .get("generated_from")
        .and_then(|v| v.as_str())
        != Some("ops/inventory/contracts-map.json")
    {
        violations.push(violation(
            "OPS_INVENTORY_META_CONTRACTS_GENERATION_METADATA_MISSING",
            "ops/inventory/meta/contracts.json must declare `generated_from: ops/inventory/contracts-map.json`"
                .to_string(),
            "mark meta/contracts.json as generated mirror metadata",
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
                format!("orphan inventory file not tracked by contracts-map: `{}`", rel.display()),
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
    let layers_json: serde_json::Value = serde_json::from_str(&layers_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
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
    let gates_json: serde_json::Value = serde_json::from_str(&gates_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_text = fs::read_to_string(ctx.repo_root.join(surfaces_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_json: serde_json::Value = serde_json::from_str(&surfaces_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
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
    if let Some(gates) = gates_json.get("gates").and_then(|v| v.as_array()) {
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
            "ops/inventory/tools.toml must not define [[checks]] or [[actions]] entries".to_string(),
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
    let policy_json: serde_json::Value = serde_json::from_str(&policy_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
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
    let pins_yaml: serde_yaml::Value = serde_yaml::from_str(&pins_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
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
                    format!(
                        "stack manifest image `{key}` differs from inventory pin value"
                    ),
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
    }

    Ok(violations)
}

pub(super) fn check_ops_docs_governance(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();

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

            for required_doc in ["README.md", "CONTRACT.md"] {
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

    let reference_index_rel = Path::new("ops/report/docs/REFERENCE_INDEX.md");
    let reference_index_text = fs::read_to_string(ctx.repo_root.join(reference_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let docs_root = ctx.repo_root.join("ops/report/docs");
    for doc in walk_files(&docs_root) {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        if rel.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = rel.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name == "REFERENCE_INDEX.md" {
            continue;
        }
        if !reference_index_text.contains(&format!("({name})")) {
            violations.push(violation(
                "OPS_REPORT_DOC_ORPHAN",
                format!(
                    "report doc `{}` is not linked from ops/report/docs/REFERENCE_INDEX.md",
                    rel.display()
                ),
                "add doc link to REFERENCE_INDEX.md or remove orphan report doc",
                Some(reference_index_rel),
            ));
        }
    }

    let control_plane_rel = Path::new("ops/CONTROL_PLANE.md");
    let control_plane_snapshot_rel = Path::new("ops/_generated.example/control-plane.snapshot.md");
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
    }

    let docs_drift_rel = Path::new("ops/_generated.example/docs-drift-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, docs_drift_rel) {
        violations.push(violation(
            "OPS_DOCS_DRIFT_ARTIFACT_MISSING",
            format!("missing docs drift artifact `{}`", docs_drift_rel.display()),
            "generate and commit docs drift report artifact",
            Some(docs_drift_rel),
        ));
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
        if text.contains("TODO") {
            violations.push(violation(
                "OPS_DOC_TODO_MARKER_FORBIDDEN",
                format!("doc `{}` contains TODO marker", rel.display()),
                "remove TODO markers from ops docs for release-ready contracts",
                Some(rel),
            ));
        }
    }

    Ok(violations)
}

pub(super) fn check_ops_evidence_bundle_discipline(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let mirror_policy_rel = Path::new("ops/_generated.example/MIRROR_POLICY.md");
    let ops_index_rel = Path::new("ops/_generated.example/ops-index.json");
    let scorecard_rel = Path::new("ops/_generated.example/scorecard.json");
    let bundle_rel = Path::new("ops/_generated.example/ops-evidence-bundle.json");
    let gates_rel = Path::new("ops/inventory/gates.json");

    for rel in [mirror_policy_rel, ops_index_rel, scorecard_rel, bundle_rel] {
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
    for required in [
        "ops-index.json",
        "ops-evidence-bundle.json",
        "scorecard.json",
        "inventory-index.json",
        "control-plane.snapshot.md",
        "docs-drift-report.json",
    ] {
        if !mirror_policy_text.contains(required) {
            violations.push(violation(
                "OPS_EVIDENCE_MIRROR_POLICY_INCOMPLETE",
                format!(
                    "mirror policy must declare mirrored artifact `{required}`"
                ),
                "update MIRROR_POLICY.md mirrored artifact list",
                Some(mirror_policy_rel),
            ));
        }
    }

    let bundle_text = fs::read_to_string(ctx.repo_root.join(bundle_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let bundle_json: serde_json::Value = serde_json::from_str(&bundle_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in ["schema_version", "release", "status", "hashes", "gates", "pin_freeze_status"] {
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

    let gates_text = fs::read_to_string(ctx.repo_root.join(gates_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let gates_json: serde_json::Value = serde_json::from_str(&gates_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
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

fn sha256_hex(path: &Path) -> Result<String, CheckError> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
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
