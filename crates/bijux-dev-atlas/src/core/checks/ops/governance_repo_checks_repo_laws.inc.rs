pub(super) fn check_repo_no_executable_script_sources(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let policy_rel = Path::new("configs/repo/script-extension-allowlist.json");
    let policy = read_json_value(&ctx.repo_root.join(policy_rel))?;
    let allowed_paths = policy["allowed_paths"]
        .as_array()
        .ok_or_else(|| {
            CheckError::Failed("script-extension-allowlist missing allowed_paths".to_string())
        })?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let extensions = policy["extensions"]
        .as_array()
        .ok_or_else(|| {
            CheckError::Failed("script-extension-allowlist missing extensions".to_string())
        })?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();

    let mut violations = Vec::new();
    for file in walk_files(ctx.repo_root) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        if rel.starts_with(".git") || rel.starts_with("artifacts") {
            continue;
        }
        let ext = file
            .extension()
            .and_then(|v| v.to_str())
            .map(|v| format!(".{v}"));
        let Some(ext) = ext else { continue };
        if !extensions.contains(&ext) {
            continue;
        }
        let rel_str = rel.display().to_string();
        if allowed_paths
            .iter()
            .any(|prefix| rel_str.starts_with(prefix))
        {
            continue;
        }
        violations.push(violation(
            "REPO_EXECUTABLE_SCRIPT_SOURCE_FORBIDDEN",
            format!("script source is forbidden outside allowlisted fixture paths: `{rel_str}`"),
            "move script to an allowlisted fixture path or replace with Rust/data contract",
            Some(rel),
        ));
    }
    Ok(violations)
}

pub(super) fn check_repo_root_markdown_allowlist_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let allowlist_rel = Path::new("configs/repo/root-file-allowlist.json");
    let allowlist = read_json_value(&ctx.repo_root.join(allowlist_rel))?;
    let allowed = allowlist["allowed_root_markdown"]
        .as_array()
        .ok_or_else(|| {
            CheckError::Failed("root-file-allowlist missing allowed_root_markdown".to_string())
        })?
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for entry in fs::read_dir(ctx.repo_root).map_err(|err| CheckError::Failed(err.to_string()))? {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name.ends_with(".md") && !allowed.contains(name) {
            violations.push(violation(
                "REPO_ROOT_MARKDOWN_NOT_ALLOWLISTED",
                format!("unexpected root markdown file: `{name}`"),
                "add file to root-file-allowlist.json only after governance approval",
                Some(Path::new(name)),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_repo_root_directory_allowlist_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let allowlist_rel = Path::new("configs/repo/root-file-allowlist.json");
    let allowlist = read_json_value(&ctx.repo_root.join(allowlist_rel))?;
    let allowed = allowlist["allowed_root_directories"]
        .as_array()
        .ok_or_else(|| {
            CheckError::Failed("root-file-allowlist missing allowed_root_directories".to_string())
        })?
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for entry in fs::read_dir(ctx.repo_root).map_err(|err| CheckError::Failed(err.to_string()))? {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name == ".git" || name == ".idea" {
            continue;
        }
        if !allowed.contains(name) {
            violations.push(violation(
                "REPO_ROOT_DIRECTORY_NOT_ALLOWLISTED",
                format!("unexpected root directory: `{name}`"),
                "add directory to root-file-allowlist.json only after governance approval",
                Some(Path::new(name)),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_repo_artifacts_not_tracked(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let output = std::process::Command::new("git")
        .current_dir(ctx.repo_root)
        .args(["ls-files", "artifacts"])
        .output()
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if !output.status.success() {
        return Ok(vec![violation(
            "REPO_ARTIFACTS_TRACKING_CHECK_FAILED",
            "git ls-files artifacts failed".to_string(),
            "ensure git is available and artifacts is ignored",
            Some(Path::new("artifacts")),
        )]);
    }
    let tracked_text = String::from_utf8_lossy(&output.stdout).into_owned();
    let tracked = tracked_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    Ok(tracked
        .into_iter()
        .map(|path| {
            violation(
                "REPO_ARTIFACTS_TRACKED_PATH_FORBIDDEN",
                format!("tracked path under artifacts is forbidden: `{path}`"),
                "remove tracked artifacts and keep artifacts/ ignored",
                Some(Path::new(path)),
            )
        })
        .collect())
}

pub(super) fn check_repo_generated_content_stays_in_allowed_paths(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let allowlist_rel = Path::new("configs/repo/generated-path-allowlist.json");
    let allowlist = read_json_value(&ctx.repo_root.join(allowlist_rel))?;
    let allowed_prefixes = allowlist["allowed_prefixes"]
        .as_array()
        .ok_or_else(|| {
            CheckError::Failed("generated-path-allowlist missing allowed_prefixes".to_string())
        })?
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    let output = std::process::Command::new("git")
        .current_dir(ctx.repo_root)
        .args(["ls-files"])
        .output()
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if !output.status.success() {
        return Ok(vec![violation(
            "REPO_GENERATED_PATH_SCAN_FAILED",
            "git ls-files failed".to_string(),
            "ensure git is available for generated-path governance checks",
            None,
        )]);
    }
    let mut violations = Vec::new();
    for rel in String::from_utf8_lossy(&output.stdout).lines() {
        let is_generated = rel.contains("/generated/")
            || rel.starts_with("ops/_generated")
            || rel.starts_with("configs/_generated");
        if !is_generated {
            continue;
        }
        if allowed_prefixes
            .iter()
            .any(|prefix| rel.starts_with(prefix))
        {
            continue;
        }
        violations.push(violation(
            "REPO_GENERATED_PATH_OUTSIDE_ALLOWLIST",
            format!("generated content path outside allowlist: `{rel}`"),
            "move generated content under approved generated output roots",
            Some(Path::new(rel)),
        ));
    }
    Ok(violations)
}

pub(super) fn check_repo_duplicate_ssot_registries_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for rel in ["metadata", "owners", "registry", "sections"] {
        let path = Path::new(rel);
        if ctx.adapters.fs.exists(ctx.repo_root, path) {
            violations.push(violation(
                "REPO_DUPLICATE_SSOT_REGISTRY_ROOT_PRESENT",
                format!("duplicate ssot registry root is forbidden at repo root: `{rel}`"),
                "keep ssot registries under canonical docs/configs/ops locations",
                Some(path),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_repo_law_metadata_complete_and_unique(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("docs/_internal/contracts/repo-laws.json");
    let payload = read_json_value(&ctx.repo_root.join(rel))?;
    let laws = payload["laws"]
        .as_array()
        .ok_or_else(|| CheckError::Failed("repo-laws.json missing laws array".to_string()))?;
    let mut ids = BTreeSet::new();
    let mut ordered = Vec::new();
    let mut violations = Vec::new();
    for law in laws {
        let id = law.get("id").and_then(Value::as_str).unwrap_or_default();
        let severity = law
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let owner = law.get("owner").and_then(Value::as_str).unwrap_or_default();
        if id.is_empty() || severity.is_empty() || owner.is_empty() {
            violations.push(violation(
                "REPO_LAW_METADATA_MISSING_FIELDS",
                "repo law is missing id/severity/owner".to_string(),
                "set id, severity, and owner for every repo law row",
                Some(rel),
            ));
            continue;
        }
        ordered.push(id.to_string());
        if !ids.insert(id.to_string()) {
            violations.push(violation(
                "REPO_LAW_ID_DUPLICATE",
                format!("duplicate repo law id: `{id}`"),
                "keep repo law ids unique and stable",
                Some(rel),
            ));
        }
    }
    let mut sorted = ordered.clone();
    sorted.sort();
    if ordered != sorted {
        violations.push(violation(
            "REPO_LAW_ORDER_NONDETERMINISTIC",
            "repo-laws.json laws must be sorted by id".to_string(),
            "sort laws by id for deterministic ordering",
            Some(rel),
        ));
    }
    Ok(violations)
}

pub(super) fn check_repo_pr_required_suite_not_skippable(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".github/workflows/ci-pr.yml");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    if !text.contains("--suite repo_required") {
        violations.push(violation(
            "REPO_REQUIRED_SUITE_NOT_IN_CI_PR",
            "ci-pr workflow must execute repo_required suite".to_string(),
            "add check run --suite repo_required to ci-pr workflow",
            Some(rel),
        ));
    }
    if !text.contains("--suite repo_required --include-internal --include-slow --allow-git") {
        violations.push(violation(
            "REPO_REQUIRED_SUITE_MISSING_REQUIRED_GIT_CAPABILITY",
            "ci-pr workflow must execute repo_required with --allow-git".to_string(),
            "run repo_required with --allow-git so tracked artifacts checks are not skipped",
            Some(rel),
        ));
    }
    if text.contains("continue-on-error: true") {
        violations.push(violation(
            "REPO_CI_PR_CONTINUE_ON_ERROR_FORBIDDEN",
            "ci-pr workflow must not use continue-on-error for required validation".to_string(),
            "remove continue-on-error from required workflow jobs",
            Some(rel),
        ));
    }
    Ok(violations)
}

pub(super) fn check_repo_suite_includes_p0_checks(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let registry_rel = Path::new("ops/inventory/registry.toml");
    let registry_text = fs::read_to_string(ctx.repo_root.join(registry_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let summary_rel = Path::new("ops/_generated.example/repo-integrity-summary.json");
    let summary = read_json_value(&ctx.repo_root.join(summary_rel))?;
    let p0 = summary["p0_checks"]
        .as_array()
        .ok_or_else(|| CheckError::Failed("repo-integrity-summary missing p0_checks".to_string()))?
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let mut violations = Vec::new();
    for check_id in p0 {
        if !registry_text.contains(&format!("\"{check_id}\"")) {
            violations.push(violation(
                "REPO_P0_CHECK_MISSING_FROM_REGISTRY",
                format!("p0 check is not declared in registry suites: `{check_id}`"),
                "add p0 check id to repo_required and ci_pr suite coverage",
                Some(registry_rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_repo_registry_order_deterministic(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("ops/inventory/registry.toml");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut check_ids = Vec::new();
    let mut suite_ids = Vec::new();
    let mut mode = "";
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[[checks]]" {
            mode = "checks";
            continue;
        }
        if trimmed == "[[suites]]" {
            mode = "suites";
            continue;
        }
        if let Some(raw_id) = trimmed.strip_prefix("id = ") {
            let id = raw_id.trim().trim_matches('"').to_string();
            if mode == "checks" {
                check_ids.push(id);
            } else if mode == "suites" {
                suite_ids.push(id);
            }
        }
    }
    let mut violations = Vec::new();
    let mut sorted_checks = check_ids.clone();
    sorted_checks.sort();
    if check_ids != sorted_checks {
        violations.push(violation(
            "REPO_CHECK_REGISTRY_ORDER_NOT_DETERMINISTIC",
            "ops/inventory/registry.toml checks must be sorted by id".to_string(),
            "sort [[checks]] blocks lexicographically by id",
            Some(rel),
        ));
    }
    let mut sorted_suites = suite_ids.clone();
    sorted_suites.sort();
    if suite_ids != sorted_suites {
        violations.push(violation(
            "REPO_SUITE_REGISTRY_ORDER_NOT_DETERMINISTIC",
            "ops/inventory/registry.toml suites must be sorted by id".to_string(),
            "sort [[suites]] blocks lexicographically by id",
            Some(rel),
        ));
    }
    Ok(violations)
}

pub(super) fn check_repo_defaults_work_surface_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let build_mk_rel = Path::new("make/build.mk");
    let docs_mk_rel = Path::new("make/docs.mk");
    let helm_templates_rel = Path::new("ops/k8s/charts/bijux-atlas/templates");
    let build_text = fs::read_to_string(ctx.repo_root.join(build_mk_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let docs_text = fs::read_to_string(ctx.repo_root.join(docs_mk_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if !build_text.contains("build:") {
        violations.push(violation(
            "REPO_DEFAULT_BUILD_TARGET_MISSING",
            "make/build.mk must define `build:` target".to_string(),
            "restore default build target",
            Some(build_mk_rel),
        ));
    }
    if !docs_text.contains("docs-build:") {
        violations.push(violation(
            "REPO_DEFAULT_DOCS_TARGET_MISSING",
            "make/docs.mk must define `docs-build:` target".to_string(),
            "restore default docs build target",
            Some(docs_mk_rel),
        ));
    }
    if !ctx.adapters.fs.exists(ctx.repo_root, helm_templates_rel) {
        violations.push(violation(
            "REPO_DEFAULT_HELM_TEMPLATE_SURFACE_MISSING",
            "helm template surface directory is missing".to_string(),
            "restore ops/k8s/charts/bijux-atlas/templates",
            Some(helm_templates_rel),
        ));
    }
    Ok(violations)
}
