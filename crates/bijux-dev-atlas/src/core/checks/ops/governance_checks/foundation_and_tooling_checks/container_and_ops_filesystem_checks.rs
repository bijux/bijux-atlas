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
            "remove Makefile from ops/ and delegate through make/*.mk wrappers",
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
