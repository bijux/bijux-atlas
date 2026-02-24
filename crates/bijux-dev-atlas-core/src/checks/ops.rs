#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use bijux_dev_atlas_model::{CheckId, Severity, Violation};

use crate::{CheckContext, CheckError, CheckFn};

const OPS_TEXT_EXTENSIONS: [&str; 5] = ["md", "json", "toml", "yaml", "yml"];

pub fn builtin_ops_check_fn(check_id: &CheckId) -> Option<CheckFn> {
    match check_id.as_str() {
        "ops_surface_manifest" => Some(check_ops_surface_manifest),
        "ops_tree_contract" => Some(checks_ops_tree_contract),
        "ops_no_legacy_tooling_refs" => Some(checks_ops_no_legacy_tooling_refs),
        "ops_generated_readonly_markers" => Some(checks_ops_generated_readonly_markers),
        "ops_schema_presence" => Some(checks_ops_schema_presence),
        "ops_manifest_integrity" => Some(checks_ops_manifest_integrity),
        "ops_surface_inventory" => Some(checks_ops_surface_inventory),
        "ops_artifacts_not_tracked" => Some(checks_ops_artifacts_not_tracked),
        "ops_no_python_legacy_runtime_refs" => Some(checks_ops_no_python_legacy_runtime_refs),
        "ops_no_legacy_runner_paths" => Some(checks_ops_no_legacy_runner_paths),
        "ops_no_atlasctl_invocations" => Some(checks_ops_no_atlasctl_invocations),
        "ops_no_scripts_areas_or_xtask_refs" => Some(checks_ops_no_scripts_areas_or_xtask_refs),
        "ops_artifacts_gitignore_policy" => Some(checks_ops_artifacts_gitignore_policy),
        "ops_makefile_routes_dev_atlas" => Some(checks_ops_makefile_routes_dev_atlas),
        "ops_workflow_routes_dev_atlas" => Some(checks_ops_workflow_routes_dev_atlas),
        "ops_internal_registry_consistency" => Some(check_ops_internal_registry_consistency),
        _ => None,
    }
}

fn violation(code: &str, message: String, hint: &str, path: Option<&Path>) -> Violation {
    Violation {
        code: code.to_string(),
        message,
        hint: Some(hint.to_string()),
        path: path.map(|p| p.display().to_string()),
        line: None,
        severity: Severity::Error,
    }
}

fn read_dir_entries(path: &Path) -> Vec<PathBuf> {
    match fs::read_dir(path) {
        Ok(entries) => entries.filter_map(Result::ok).map(|e| e.path()).collect(),
        Err(_) => Vec::new(),
    }
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in read_dir_entries(&dir) {
            if entry.is_dir() {
                stack.push(entry);
            } else if entry.is_file() {
                out.push(entry);
            }
        }
    }
    out.sort();
    out
}

fn check_ops_surface_manifest(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let manifest = Path::new("configs/ops/ops-surface-manifest.json");
    let surface = Path::new("ops/inventory/surfaces.json");
    let mut violations = Vec::new();
    if !ctx.adapters.fs.exists(ctx.repo_root, manifest) {
        violations.push(violation(
            "OPS_SURFACE_MANIFEST_MISSING",
            "missing configs/ops/ops-surface-manifest.json".to_string(),
            "restore ops surface manifest",
            Some(manifest),
        ));
    }
    if !ctx.adapters.fs.exists(ctx.repo_root, surface) {
        violations.push(violation(
            "OPS_SURFACE_INVENTORY_MISSING",
            "missing ops/inventory/surfaces.json".to_string(),
            "regenerate inventory surfaces",
            Some(surface),
        ));
    }
    Ok(violations)
}

fn checks_ops_tree_contract(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let required = [
        "ops/CONTRACT.md",
        "ops/INDEX.md",
        "ops/ERRORS.md",
        "ops/README.md",
    ];
    let mut violations = Vec::new();
    for path in required {
        let rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_TREE_REQUIRED_PATH_MISSING",
                format!("missing required ops path `{path}`"),
                "restore the required ops contract file",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

fn checks_ops_no_legacy_tooling_refs(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let forbidden = [
        ("scripts/areas", "OPS_FORBIDDEN_SCRIPTS_AREAS_REF"),
        ("xtask", "OPS_FORBIDDEN_XTASK_REF"),
        ("/tools/", "OPS_FORBIDDEN_TOOLS_REF"),
    ];
    let mut violations = Vec::new();
    for rel in ["ops/CONTRACT.md", "ops/INDEX.md", "ops/README.md"] {
        let file = ctx.repo_root.join(rel);
        if !file.exists() {
            continue;
        }
        let ext = file
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if !OPS_TEXT_EXTENSIONS.contains(&ext) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let rel = file
            .strip_prefix(ctx.repo_root)
            .unwrap_or(&file)
            .to_path_buf();
        for (needle, code) in forbidden {
            if content.contains(needle) {
                violations.push(violation(
                    code,
                    format!(
                        "forbidden legacy reference `{needle}` found in {}",
                        rel.display()
                    ),
                    "remove legacy tooling references from ops contracts",
                    Some(rel.as_path()),
                ));
            }
        }
    }
    Ok(violations)
}

fn checks_ops_generated_readonly_markers(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let policy_rel = Path::new("ops/inventory/generated-committed-mirror.json");
    let policy_path = ctx.repo_root.join(policy_rel);
    let policy_text =
        fs::read_to_string(&policy_path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let policy_json: serde_json::Value =
        serde_json::from_str(&policy_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut allowlisted = BTreeSet::new();
    if let Some(entries) = policy_json
        .get("allow_runtime_compat")
        .and_then(|v| v.as_array())
    {
        for entry in entries {
            if let Some(path) = entry.as_str() {
                allowlisted.insert(path.to_string());
            }
        }
    }
    if let Some(entries) = policy_json.get("mirrors").and_then(|v| v.as_array()) {
        for entry in entries {
            if let Some(path) = entry.get("committed").and_then(|v| v.as_str()) {
                allowlisted.insert(path.to_string());
            }
        }
    }

    let roots = ["ops/_generated.example"];
    let mut violations = Vec::new();
    for root in roots {
        let dir = ctx.repo_root.join(root);
        if !dir.exists() {
            continue;
        }
        for file in walk_files(&dir) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
            let rel_str = rel.display().to_string();
            if !allowlisted.contains(&rel_str) {
                violations.push(violation(
                    "OPS_GENERATED_FILE_ALLOWLIST_MISSING",
                    format!("generated mirror file `{}` is not declared in mirror policy", rel_str),
                    "declare generated mirror files in ops/inventory/generated-committed-mirror.json",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

fn checks_ops_schema_presence(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let required = [
        "ops/schema/README.md",
        "ops/schema/meta/ownership.schema.json",
        "ops/schema/report/unified.schema.json",
        "ops/schema/stack/profile-manifest.schema.json",
    ];
    let mut violations = Vec::new();
    for path in required {
        let rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_SCHEMA_REQUIRED_FILE_MISSING",
                format!("missing required schema file `{path}`"),
                "restore required schema file under ops/schema",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

fn checks_ops_manifest_integrity(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let manifests: [(&str, &[&str]); 3] = [
        (
            "ops/inventory/surfaces.json",
            &["schema_version", "entrypoints"],
        ),
        ("ops/inventory/contracts.json", &["schema_version"]),
        ("ops/inventory/drills.json", &["schema_version"]),
    ];
    let mut violations = Vec::new();
    for (path, required_keys) in manifests {
        let rel = Path::new(path);
        let target = ctx.repo_root.join(rel);
        let Ok(text) = fs::read_to_string(&target) else {
            violations.push(violation(
                "OPS_MANIFEST_MISSING",
                format!("missing required manifest `{path}`"),
                "restore required inventory manifest",
                Some(rel),
            ));
            continue;
        };
        let parsed = serde_json::from_str::<serde_json::Value>(&text);
        let Ok(value) = parsed else {
            violations.push(violation(
                "OPS_MANIFEST_INVALID_JSON",
                format!("manifest `{path}` is not valid JSON"),
                "fix JSON syntax in inventory manifest",
                Some(rel),
            ));
            continue;
        };
        for key in required_keys {
            if value.get(*key).is_none() {
                violations.push(violation(
                    "OPS_MANIFEST_REQUIRED_KEY_MISSING",
                    format!("manifest `{path}` is missing key `{key}`"),
                    "add the required key to the manifest payload",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

fn checks_ops_surface_inventory(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let index_rel = Path::new("ops/INDEX.md");
    let index = ctx.repo_root.join(index_rel);
    let index_text =
        fs::read_to_string(&index).map_err(|err| CheckError::Failed(err.to_string()))?;
    let required_entries = [
        "stack", "k8s", "observe", "load", "e2e", "datasets", "report",
    ];
    let listed_dirs: BTreeSet<String> = index_text
        .lines()
        .filter_map(|line| line.split("`ops/").nth(1))
        .filter_map(|tail| tail.split('/').next())
        .map(|name| name.to_string())
        .collect();

    let mut violations = Vec::new();
    for dir in required_entries {
        if !listed_dirs.contains(dir) {
            violations.push(violation(
                "OPS_INDEX_DIRECTORY_MISSING",
                format!("ops/INDEX.md does not list ops directory `{dir}`"),
                "regenerate ops index so directories are listed",
                Some(index_rel),
            ));
        }
    }
    Ok(violations)
}

fn checks_ops_artifacts_not_tracked(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let evidence_root = ctx.repo_root.join("ops/_evidence");
    if !evidence_root.exists() {
        return Ok(Vec::new());
    }
    let files = walk_files(&evidence_root);
    let tracked_like = files
        .into_iter()
        .filter(|path| path.file_name().and_then(|v| v.to_str()) != Some(".gitkeep"))
        .collect::<Vec<_>>();
    if tracked_like.is_empty() {
        Ok(Vec::new())
    } else {
        let first = tracked_like[0]
            .strip_prefix(ctx.repo_root)
            .unwrap_or(&tracked_like[0]);
        Ok(vec![violation(
            "OPS_ARTIFACTS_POLICY_VIOLATION",
            format!(
                "ops evidence directory contains committed file `{}`",
                first.display()
            ),
            "remove files under ops/_evidence and keep runtime output under artifacts/",
            Some(Path::new("ops/_evidence")),
        )])
    }
}

fn checks_ops_no_python_legacy_runtime_refs(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let forbidden = [
        ["packages/", "atlasctl"].concat(),
        ["python -m ", "atlasctl"].concat(),
        ["./bin/", "atlasctl"].concat(),
    ];
    let roots = [
        ctx.repo_root.join("crates/bijux-dev-atlas"),
        ctx.repo_root.join("crates/bijux-dev-atlas-core"),
        ctx.repo_root.join("crates/bijux-dev-atlas-adapters"),
        ctx.repo_root.join("crates/bijux-dev-atlas-model"),
    ];

    for root in roots {
        if !root.exists() {
            continue;
        }
        for file in walk_files(&root) {
            if file.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
            if rel.components().any(|c| c.as_os_str() == "tests") || rel.to_string_lossy().ends_with("_tests.rs") {
                continue;
            }
            if rel == Path::new("crates/bijux-dev-atlas-core/src/checks/ops.rs") {
                continue;
            }
            for needle in &forbidden {
                if content.contains(needle) {
                    violations.push(violation(
                        "OPS_PYTHON_LEGACY_REFERENCE_FOUND",
                        format!(
                            "forbidden legacy runtime reference `{needle}` found in {}",
                            rel.display()
                        ),
                        "remove python atlasctl coupling from bijux-dev-atlas crates",
                        Some(rel),
                    ));
                }
            }
        }
    }

    Ok(violations)
}

fn checks_ops_no_legacy_runner_paths(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let forbidden = [
        ["scripts/", "areas"].concat(),
        ["x", "task"].concat(),
        ["/to", "ols/"].concat(),
    ];
    let roots = [
        ctx.repo_root.join("crates/bijux-dev-atlas"),
        ctx.repo_root.join("crates/bijux-dev-atlas-core"),
        ctx.repo_root.join("crates/bijux-dev-atlas-adapters"),
        ctx.repo_root.join("crates/bijux-dev-atlas-model"),
    ];

    for root in roots {
        if !root.exists() {
            continue;
        }
        for file in walk_files(&root) {
            if file.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
            if rel == Path::new("crates/bijux-dev-atlas-core/src/lib_tests.rs") {
                continue;
            }
            if rel == Path::new("crates/bijux-dev-atlas-core/src/checks/ops.rs") {
                continue;
            }
            for needle in &forbidden {
                if content.contains(needle) {
                    violations.push(violation(
                        "OPS_LEGACY_RUNNER_PATH_REFERENCE_FOUND",
                        format!(
                            "forbidden legacy runner path reference `{needle}` found in {}",
                            rel.display()
                        ),
                        "remove legacy runner path references from dev-atlas crates",
                        Some(rel),
                    ));
                }
            }
        }
    }

    Ok(violations)
}

fn checks_ops_makefile_routes_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("makefiles/ops.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;

    let mut violations = Vec::new();
    let has_dev_route = content.contains("BIJUX_DEV_ATLAS")
        || content.contains("bijux dev atlas");
    let has_legacy_route =
        content.contains("./bin/atlasctl") || content.contains(" atlasctl ") || content.contains("atlasctl ops");
    if !has_dev_route || has_legacy_route {
        violations.push(violation(
            "OPS_MAKEFILE_ROUTE_INVALID",
            "makefiles/ops.mk must delegate to BIJUX_DEV_ATLAS and must not call atlasctl".to_string(),
            "replace atlasctl calls with bijux dev atlas routing in makefiles/ops.mk",
            Some(rel),
        ));
    }

    let forbidden_tokens = ["bash ops/", "scripts/areas"];
    for token in forbidden_tokens {
        if content.contains(token) {
            violations.push(violation(
                "OPS_MAKEFILE_DELEGATION_ONLY_VIOLATION",
                format!("makefiles/ops.mk contains forbidden token `{token}`"),
                "keep ops.mk as thin delegation-only wrappers",
                Some(rel),
            ));
        }
    }
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        if line.contains("$(BIJUX_DEV_ATLAS)") {
            continue;
        }
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.iter().any(|word| *word == "kubectl" || *word == "helm" || *word == "kind" || *word == "k6") {
            violations.push(violation(
                "OPS_MAKEFILE_DELEGATION_ONLY_VIOLATION",
                format!("makefiles/ops.mk must not execute tool binary directly: `{line}`"),
                "delegate to bijux dev atlas ops commands instead of direct tool execution",
                Some(rel),
            ));
        }
    }

    let target_lines = content
        .lines()
        .filter(|line| line.ends_with(':') || line.contains(": ##"))
        .filter(|line| !line.starts_with('.') && !line.starts_with('#'))
        .count();
    if target_lines > 20 {
        violations.push(violation(
            "OPS_MAKEFILE_TARGET_BUDGET_EXCEEDED",
            format!("makefiles/ops.mk defines {target_lines} targets; budget is 20"),
            "reduce ops.mk target count or move behavior into rust commands",
            Some(rel),
        ));
    }

    if !content.contains("PROFILE ?= kind") {
        violations.push(violation(
            "OPS_MAKEFILE_PROFILE_DEFAULT_MISSING",
            "makefiles/ops.mk must declare `PROFILE ?= kind`".to_string(),
            "add deterministic profile default in makefiles/ops.mk",
            Some(rel),
        ));
    }

    if !content.contains("ops-help:") {
        violations.push(violation(
            "OPS_MAKEFILE_HELP_TARGET_MISSING",
            "makefiles/ops.mk must expose `ops-help` target".to_string(),
            "add ops-help target delegating to bijux dev atlas ops --help",
            Some(rel),
        ));
    }

    if !content.contains("ops-doctor:")
        || !content.contains("ops-validate:")
        || !content.contains("ops-render:")
        || !content.contains("ops-install-plan:")
        || !content.contains("ops-status:")
        || !content.contains("ops-tools-verify:")
    {
        violations.push(violation(
            "OPS_MAKEFILE_REQUIRED_TARGETS_MISSING",
            "makefiles/ops.mk is missing required delegation targets".to_string(),
            "add ops-doctor, ops-validate, ops-render, ops-install-plan, ops-status, ops-tools-verify",
            Some(rel),
        ));
    }

    Ok(violations)
}

fn checks_ops_no_atlasctl_invocations(
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

fn checks_ops_no_scripts_areas_or_xtask_refs(
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
                if !trimmed.contains(needle) {
                    continue;
                }
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
    Ok(violations)
}

fn checks_ops_artifacts_gitignore_policy(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".gitignore");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    if content.lines().any(|line| line.trim() == "artifacts/" || line.trim() == "/artifacts/") {
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

fn checks_ops_workflow_routes_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".github/workflows/atlas-dev-rust.yml");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;

    let has_legacy_ops_route =
        content.contains("./bin/atlasctl ops") || content.contains(" atlasctl ops ");
    if has_legacy_ops_route {
        return Ok(vec![violation(
            "OPS_WORKFLOW_ROUTE_INVALID",
            "atlas-dev-rust workflow must not call atlasctl ops commands".to_string(),
            "route ops checks through bijux-dev-atlas commands",
            Some(rel),
        )]);
    }

    Ok(Vec::new())
}

fn check_ops_internal_registry_consistency(
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
