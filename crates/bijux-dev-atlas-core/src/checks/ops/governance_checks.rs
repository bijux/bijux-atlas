use super::*;

pub(super) fn checks_ops_no_atlasctl_invocations(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let targets = [
        Path::new("makefiles/ops.mk"),
        Path::new("makefiles/CONTRACT.md"),
        Path::new(".github/workflows/atlas-dev-rust.yml"),
        Path::new("ops/CONTRACT.md"),
        Path::new("ops/README.md"),
        Path::new("ops/INDEX.md"),
    ];
    let mut violations = Vec::new();
    for rel in targets {
        let path = ctx.repo_root.join(rel);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            let is_command_like = trimmed.contains("./bin/atlasctl")
                || trimmed.contains(" atlasctl ")
                || trimmed.starts_with("atlasctl ");
            if is_command_like {
                violations.push(violation(
                    "OPS_ATLASCTL_REFERENCE_FOUND",
                    format!(
                        "forbidden atlasctl invocation found in {}: `{trimmed}`",
                        rel.display()
                    ),
                    "route ops control-plane invocations through bijux dev atlas",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

pub(super) fn checks_ops_no_scripts_areas_or_xtask_refs(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let targets = [
        Path::new("makefiles/ops.mk"),
        Path::new(".github/workflows/atlas-dev-rust.yml"),
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
                            "forbidden legacy reference `{needle}` found in {}: `{trimmed}`",
                            rel.display()
                        ),
                        "remove scripts/areas and xtask references from ops-owned surfaces",
                        Some(rel),
                    ));
                }
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
    let rel = Path::new(".github/workflows/atlas-dev-rust.yml");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let has_legacy_ops_route =
        content.contains("./bin/atlasctl ops") || content.contains(" atlasctl ops ");
    if has_legacy_ops_route {
        Ok(vec![violation(
            "OPS_WORKFLOW_ROUTE_INVALID",
            "atlas-dev-rust workflow must not call atlasctl ops commands".to_string(),
            "route ops checks through bijux-dev-atlas commands",
            Some(rel),
        )])
    } else {
        Ok(Vec::new())
    }
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

pub(super) fn check_root_packages_atlasctl_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("packages").join("atlasctl");
    if ctx.adapters.fs.exists(ctx.repo_root, &rel) {
        Ok(vec![Violation {
            code: "ROOT_PACKAGES_ATLASCTL_STILL_PRESENT".to_string(),
            message: "legacy package-tree atlasctl directory still exists".to_string(),
            hint: Some("remove the legacy atlasctl package tree after migration closure".to_string()),
            path: Some(rel.display().to_string()),
            line: None,
            severity: Severity::Warn,
        }])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_root_bin_atlasctl_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("bin/atlasctl");
    if ctx.adapters.fs.exists(ctx.repo_root, &rel) {
        Ok(vec![violation(
            "ROOT_BIN_ATLASCTL_SHIM_PRESENT",
            "legacy root atlasctl shim still exists".to_string(),
            "delete bin/atlasctl; use cargo run -p bijux-dev-atlas or bijux dev atlas",
            Some(&rel),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_root_artifacts_reports_atlasctl_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("artifacts/reports/atlasctl");
    if ctx.adapters.fs.exists(ctx.repo_root, &rel) {
        Ok(vec![violation(
            "ROOT_ARTIFACTS_REPORTS_ATLASCTL_PRESENT",
            "legacy atlasctl report artifact directory exists".to_string(),
            "remove artifacts/reports/atlasctl and migrate report writers to artifacts/atlas-dev",
            Some(&rel),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_root_python_toolchain_toml_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("packages").join("python-toolchain.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, &rel) {
        Ok(vec![violation(
            "ROOT_PYTHON_TOOLCHAIN_TOML_PRESENT",
            "legacy python toolchain SSOT file still exists".to_string(),
            "delete the legacy python toolchain file after control-plane migration",
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
            "legacy root uv.lock exists".to_string(),
            "remove uv.lock if it is only used for atlasctl tooling",
            Some(rel),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_docs_no_atlasctl_string_references(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let root = ctx.repo_root.join("docs");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let allowed_prefix = Path::new("docs/development/tooling/atlasctl-deletion");
    let mut violations = Vec::new();
    for file in walk_files(&root) {
        if let Ok(content) = fs::read_to_string(&file) {
            if !content.contains("atlasctl") {
                continue;
            }
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
            if rel.starts_with(allowed_prefix) {
                continue;
            }
            violations.push(violation(
                "DOCS_ATLASCTL_REFERENCE_FOUND",
                format!("docs file contains forbidden `atlasctl` reference: {}", rel.display()),
                "move legacy mention under docs/development/tooling/atlasctl-deletion/ or remove it",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}
pub(super) fn check_workflows_no_atlasctl_string_references(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_string_references_under(
        ctx,
        ".github/workflows",
        "atlasctl",
        "WORKFLOW_ATLASCTL_REFERENCE_FOUND",
    )
}
pub(super) fn check_make_no_atlasctl_string_references(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_string_references_under(
        ctx,
        "makefiles",
        "atlasctl",
        "MAKE_ATLASCTL_REFERENCE_FOUND",
    )
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
        && non_comment_lines.iter().any(|line| line.contains("docker/") || line.contains("see "));
    if looks_like_pointer {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "ROOT_DOCKERFILE_FORBIDDEN",
            "root Dockerfile must be absent or a tiny pointer to canonical docker/ definitions".to_string(),
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
                format!("Dockerfile outside canonical docker/ directory: {}", rel.display()),
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
    for file in walk_files(&ops_root) {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or_default();
        if ext != "sh" && ext != "py" {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        let allowlisted_fixture = rel.starts_with("ops/stack/tests/");
        if allowlisted_fixture {
            continue;
        }
        violations.push(violation(
            "OPS_BEHAVIOR_SOURCE_FILE_PRESENT",
            format!("ops contains behavior source file outside fixture allowlist: {}", rel.display()),
            "move behavior into crates/bijux-dev-atlas; keep ops/ for manifests, schemas, fixtures, and docs",
            Some(rel),
        ));
    }
    Ok(violations)
}

pub(super) fn check_root_no_scripts_areas_presence_or_references(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let root = Path::new("scripts/areas");
    if ctx.adapters.fs.exists(ctx.repo_root, root) {
        violations.push(violation(
            "ROOT_SCRIPTS_AREAS_DIRECTORY_PRESENT",
            "scripts/areas directory exists".to_string(),
            "remove scripts/areas; route control-plane through bijux dev atlas",
            Some(root),
        ));
    }
    for rel in ["makefiles", ".github/workflows", "docs", "ops"] {
        violations.extend(check_no_string_references_under(
            ctx,
            rel,
            "scripts/areas",
            "ROOT_SCRIPTS_AREAS_REFERENCE_FOUND",
        )?);
    }
    Ok(violations)
}

pub(super) fn check_root_forbidden_legacy_directories_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for rel in ["scripts", "tools", "xtask"] {
        let path = Path::new(rel);
        if ctx.adapters.fs.exists(ctx.repo_root, path) {
            violations.push(violation(
                "ROOT_FORBIDDEN_LEGACY_DIRECTORY_PRESENT",
                format!("forbidden legacy top-level directory exists: {}", path.display()),
                "delete the directory and move behavior into crates/bijux-dev-atlas command surfaces",
                Some(path),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_root_makefile_single_include_entrypoint(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("Makefile");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>();
    if lines.len() == 1 && lines[0] == "include makefiles/root.mk" {
        return Ok(Vec::new());
    }
    if !lines.contains(&"include makefiles/root.mk") {
        return Ok(vec![violation(
            "ROOT_MAKEFILE_MISSING_ROOT_INCLUDE",
            "root Makefile must include makefiles/root.mk".to_string(),
            "use root Makefile as a single include entrypoint",
            Some(rel),
        )]);
    }
    lines.retain(|line| *line != "include makefiles/root.mk");
    Ok(vec![violation(
        "ROOT_MAKEFILE_NOT_SINGLE_INCLUDE_ENTRYPOINT",
        "root Makefile contains logic beyond the single root include".to_string(),
        "keep root Makefile to one line: `include makefiles/root.mk`",
        Some(rel),
    )])
}

pub(super) fn check_makefiles_root_includes_sorted(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("makefiles/root.mk");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let includes = text
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("include "))
        .map(|line| line.trim_start_matches("include ").to_string())
        .collect::<Vec<_>>();
    let mut sorted = includes.clone();
    sorted.sort();
    if includes == sorted {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "MAKEFILES_ROOT_INCLUDES_NOT_SORTED",
            "makefiles/root.mk include statements must be sorted".to_string(),
            "sort include lines lexicographically for deterministic diffs",
            Some(rel),
        )])
    }
}

pub(super) fn check_root_top_level_directories_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let expected = ["artifacts", "configs", "crates", "docker", "docs", "makefiles", "ops"];
    let mut actual = fs::read_dir(ctx.repo_root)
        .map_err(|err| CheckError::Failed(err.to_string()))?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect::<Vec<_>>();
    actual.sort();
    let expected_vec = expected.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    if actual == expected_vec {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "ROOT_TOP_LEVEL_DIRECTORIES_CONTRACT_MISMATCH",
            format!("top-level visible directories mismatch: actual={actual:?} expected={expected_vec:?}"),
            "keep only the canonical top-level directory set and move legacy roots into crates/ or ops/",
            None,
        )])
    }
}

pub(super) fn check_root_cargo_config_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".cargo/config.toml");
    let path = ctx.repo_root.join(rel);
    let text = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for required in ["[build]", "target-dir = \"artifacts/target\"", "[term]"] {
        if !text.contains(required) {
            violations.push(violation(
                "ROOT_CARGO_CONFIG_CONTRACT_MISSING",
                format!(".cargo/config.toml is missing required contract snippet: `{required}`"),
                "restore deterministic cargo target-dir and terminal config defaults",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_root_rust_toolchain_toml_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("rust-toolchain.toml");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for required in [
        "[toolchain]",
        "channel = \"",
        "profile = \"minimal\"",
        "components = [\"rustfmt\", \"clippy\"]",
    ] {
        if !text.contains(required) {
            violations.push(violation(
                "ROOT_RUST_TOOLCHAIN_CONTRACT_MISSING",
                format!("rust-toolchain.toml is missing required contract snippet: `{required}`"),
                "pin stable Rust channel with minimal profile and rustfmt/clippy components",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_root_rustfmt_toml_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/rust/rustfmt.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "ROOT_RUSTFMT_TOML_MISSING",
            "configs/rust/rustfmt.toml must exist".to_string(),
            "define rustfmt policy under configs/rust and use explicit cargo fmt --config-path",
            Some(rel),
        )])
    }
}

pub(super) fn check_root_clippy_toml_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/rust/clippy.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "ROOT_CLIPPY_TOML_MISSING",
            "configs/rust/clippy.toml must exist".to_string(),
            "define clippy policy under configs/rust and use explicit CLIPPY_CONF_DIR",
            Some(rel),
        )])
    }
}

pub(super) fn check_configs_nextest_toml_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/nextest/nextest.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CONFIGS_NEXTEST_TOML_MISSING",
            "configs/nextest/nextest.toml must exist".to_string(),
            "define nextest execution profiles and isolated store path under configs/nextest",
            Some(rel),
        )])
    }
}

pub(super) fn check_configs_security_deny_toml_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("configs/security/deny.toml");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CONFIGS_SECURITY_DENY_TOML_MISSING",
            "configs/security/deny.toml must exist".to_string(),
            "keep cargo-deny policy under configs/security/deny.toml",
            Some(rel),
        )])
    }
}

pub(super) fn check_workflows_rust_toolchain_matches_repo_pin(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let toolchain_rel = Path::new("rust-toolchain.toml");
    let workflow_rel = Path::new(".github/workflows/atlas-dev-rust.yml");
    let toolchain_text = fs::read_to_string(ctx.repo_root.join(toolchain_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let workflow_text = fs::read_to_string(ctx.repo_root.join(workflow_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let pinned = toolchain_text
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("channel = ") {
                trimmed.split('"').nth(1).map(str::to_string)
            } else {
                None
            }
        })
        .ok_or_else(|| CheckError::Failed("rust-toolchain.toml channel missing".to_string()))?;
    let expected = format!("toolchain: {pinned}");
    if workflow_text.contains(&expected) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "WORKFLOWS_RUST_TOOLCHAIN_PIN_MISMATCH",
            format!(
                "atlas-dev-rust workflow must pin the same Rust toolchain as rust-toolchain.toml (`{pinned}`)"
            ),
            "update .github/workflows/atlas-dev-rust.yml dtolnay/rust-toolchain step `with.toolchain` to match rust-toolchain.toml",
            Some(workflow_rel),
        )])
    }
}

pub(super) fn check_crates_bijux_atlas_cli_owns_umbrella_dispatch(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let crates_root = ctx.repo_root.join("crates");
    if !crates_root.exists() {
        return Ok(Vec::new());
    }
    let mut owners = BTreeSet::new();
    for file in walk_files(&crates_root) {
        if file.extension().and_then(|v| v.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        if rel == Path::new("crates/bijux-dev-atlas-core/src/checks/ops.rs")
            || rel.starts_with("crates/bijux-dev-atlas-core/src/checks/ops/")
        {
            continue;
        }
        if content.contains("--bijux-plugin-metadata") || content.contains("--umbrella-version") {
            let owner = rel
                .components()
                .nth(1)
                .and_then(|v| v.as_os_str().to_str())
                .unwrap_or_default()
                .to_string();
            if !owner.is_empty() {
                owners.insert(owner);
            }
        }
    }
    if owners == BTreeSet::from(["bijux-atlas-cli".to_string()]) {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_UMBRELLA_DISPATCH_OWNER_INVALID",
            format!("umbrella dispatch ownership must be bijux-atlas-cli only; found {owners:?}"),
            "keep bijux-atlas-cli as the only owner of umbrella dispatch metadata flags",
            Some(Path::new("crates")),
        )])
    }
}

pub(super) fn check_crates_bijux_atlas_help_excludes_dev_commands(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let src = ctx.repo_root.join("crates/bijux-atlas-cli/src/lib.rs");
    let text = fs::read_to_string(&src).map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("Subcommand::Dev") {
        Ok(vec![violation(
            "CRATES_ATLAS_HELP_EXPOSES_DEV_COMMANDS",
            "bijux atlas help surface must not include dev commands".to_string(),
            "move dev command routing under bijux-dev-atlas only",
            Some(Path::new("crates/bijux-atlas-cli/src/lib.rs")),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_crates_bijux_dev_atlas_help_dispatch_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let src = ctx.repo_root.join("crates/bijux-atlas-cli/src/lib.rs");
    let text = fs::read_to_string(&src).map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("bijux dev atlas <command>") {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_DEV_ATLAS_DISPATCH_HINT_MISSING",
            "bijux atlas command routing must advertise `bijux dev atlas --help`".to_string(),
            "restore dev atlas dispatch hint in bijux-atlas-cli help routing",
            Some(Path::new("crates/bijux-atlas-cli/src/lib.rs")),
        )])
    }
}

pub(super) fn check_no_string_references_under(
    ctx: &CheckContext<'_>,
    rel_root: &str,
    needle: &str,
    code: &str,
) -> Result<Vec<Violation>, CheckError> {
    let base = ctx.repo_root.join(rel_root);
    if !base.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&base) {
        let ext = file
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if !OPS_TEXT_EXTENSIONS.contains(&ext) && ext != "mk" && ext != "rs" {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        for line in content.lines() {
            if line.contains(needle) {
                let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
                violations.push(violation(
                    code,
                    format!(
                        "forbidden `{needle}` reference in {}: `{}`",
                        rel.display(),
                        line.trim()
                    ),
                    "remove legacy references and route through bijux dev atlas",
                    Some(rel),
                ));
                break;
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_no_any_string_references_under(
    ctx: &CheckContext<'_>,
    rel_root: &str,
    needles: &[&str],
    code: &str,
) -> Result<Vec<Violation>, CheckError> {
    let base = ctx.repo_root.join(rel_root);
    if !base.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&base) {
        let ext = file
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if !OPS_TEXT_EXTENSIONS.contains(&ext) && ext != "mk" && ext != "rs" {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        for line in content.lines() {
            for needle in needles {
                if line.contains(needle) {
                    let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
                    violations.push(violation(
                        code,
                        format!(
                            "forbidden `{needle}` reference in {}: `{}`",
                            rel.display(),
                            line.trim()
                        ),
                        "remove direct ops script execution and route through bijux dev atlas",
                        Some(rel),
                    ));
                    break;
                }
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_ops_no_bash_lib_execution(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for rel in ["makefiles", ".github/workflows"] {
        let root = ctx.repo_root.join(rel);
        if !root.exists() {
            continue;
        }
        for file in walk_files(&root) {
            let ext = file
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
            if !OPS_TEXT_EXTENSIONS.contains(&ext) && ext != "mk" && ext != "yml" && ext != "yaml" {
                continue;
            }
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('!') && trimmed.contains("rg -n") {
                    continue;
                }
                let invokes_ops_shell = trimmed.contains("bash ops/")
                    || trimmed.contains("source ops/")
                    || trimmed.starts_with(". ops/");
                let sources_legacy_lib = trimmed.contains("source ops/_lib")
                    || trimmed.contains(". ops/_lib")
                    || trimmed.contains("bash ops/_lib");
                if invokes_ops_shell || sources_legacy_lib {
                    let path = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
                    violations.push(violation(
                        "OPS_BASH_LIB_EXECUTION_REFERENCE_FOUND",
                        format!(
                            "forbidden bash lib execution reference in {}: `{}`",
                            path.display(),
                            trimmed
                        ),
                        "route ops behavior through bijux dev atlas rust commands",
                        Some(path),
                    ));
                    break;
                }
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_ops_legacy_shell_quarantine_empty(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("ops/quarantine/legacy-ops-shell");
    let dir = ctx.repo_root.join(rel);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut non_marker = Vec::new();
    for file in walk_files(&dir) {
        let name = file
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if name != ".gitkeep" && name != "README.md" {
            non_marker.push(file);
        }
    }
    if non_marker.is_empty() {
        Ok(Vec::new())
    } else {
        let first = non_marker[0]
            .strip_prefix(ctx.repo_root)
            .unwrap_or(&non_marker[0]);
        Ok(vec![violation(
            "OPS_LEGACY_SHELL_QUARANTINE_NOT_EMPTY",
            format!(
                "legacy ops shell quarantine must be empty; found `{}`",
                first.display()
            ),
            "delete legacy shell helpers and keep quarantine empty",
            Some(rel),
        )])
    }
}
