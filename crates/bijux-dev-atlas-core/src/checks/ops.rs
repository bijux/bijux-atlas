#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use bijux_dev_atlas_model::{CheckId, Severity, Violation};

use crate::{CheckContext, CheckError, CheckFn};

const OPS_TEXT_EXTENSIONS: [&str; 5] = ["md", "json", "toml", "yaml", "yml"];

pub fn builtin_ops_check_fn(check_id: &CheckId) -> Option<CheckFn> {
    match check_id.as_str() {
        "checks_ops_surface_manifest" => Some(check_ops_surface_manifest),
        "checks_ops_tree_contract" => Some(checks_ops_tree_contract),
        "checks_ops_no_legacy_tooling_refs" => Some(checks_ops_no_legacy_tooling_refs),
        "checks_ops_generated_readonly_markers" => Some(checks_ops_generated_readonly_markers),
        "checks_ops_schema_presence" => Some(checks_ops_schema_presence),
        "checks_ops_manifest_integrity" => Some(checks_ops_manifest_integrity),
        "checks_ops_surface_inventory" => Some(checks_ops_surface_inventory),
        "checks_ops_artifacts_not_tracked" => Some(checks_ops_artifacts_not_tracked),
        "checks_ops_no_python_legacy_runtime_refs" => Some(checks_ops_no_python_legacy_runtime_refs),
        "checks_ops_no_legacy_runner_paths" => Some(checks_ops_no_legacy_runner_paths),
        "checks_ops_no_atlasctl_invocations" => Some(checks_ops_no_atlasctl_invocations),
        "checks_ops_no_scripts_areas_or_xtask_refs" => Some(checks_ops_no_scripts_areas_or_xtask_refs),
        "checks_ops_artifacts_gitignore_policy" => Some(checks_ops_artifacts_gitignore_policy),
        "checks_ops_makefile_routes_dev_atlas" => Some(checks_ops_makefile_routes_dev_atlas),
        "checks_ops_workflow_routes_dev_atlas" => Some(checks_ops_workflow_routes_dev_atlas),
        "checks_ops_internal_registry_consistency" => Some(check_ops_internal_registry_consistency),
        "checks_root_packages_atlasctl_absent" => Some(check_root_packages_atlasctl_absent),
        "checks_docs_no_atlasctl_string_references" => Some(check_docs_no_atlasctl_string_references),
        "checks_workflows_no_atlasctl_string_references" => {
            Some(check_workflows_no_atlasctl_string_references)
        }
        "checks_make_no_atlasctl_string_references" => {
            Some(check_make_no_atlasctl_string_references)
        }
        "checks_workflows_no_direct_ops_script_execution" => {
            Some(check_workflows_no_direct_ops_script_execution)
        }
        "checks_make_no_direct_ops_script_execution" => Some(check_make_no_direct_ops_script_execution),
        "checks_root_no_scripts_areas_presence_or_references" => {
            Some(check_root_no_scripts_areas_presence_or_references)
        }
        "checks_crates_bijux_atlas_cli_owns_umbrella_dispatch" => {
            Some(check_crates_bijux_atlas_cli_owns_umbrella_dispatch)
        }
        "checks_crates_bijux_atlas_help_excludes_dev_commands" => {
            Some(check_crates_bijux_atlas_help_excludes_dev_commands)
        }
        "checks_crates_bijux_dev_atlas_help_dispatch_present" => {
            Some(check_crates_bijux_dev_atlas_help_dispatch_present)
        }
        "checks_ops_no_bash_lib_execution" => Some(check_ops_no_bash_lib_execution),
        "checks_ops_legacy_shell_quarantine_empty" => Some(check_ops_legacy_shell_quarantine_empty),
        "checks_make_governance_wrappers_bijux_only" => Some(check_make_governance_wrappers_bijux_only),
        "checks_workflows_governance_entrypoints_bijux_only" => {
            Some(check_workflows_governance_entrypoints_bijux_only)
        }
        "checks_make_governance_wrappers_no_direct_cargo" => {
            Some(check_make_governance_wrappers_no_direct_cargo)
        }
        "checks_docs_runtime_command_list_matches_contract" => {
            Some(check_docs_runtime_command_list_matches_contract)
        }
        "checks_docs_dev_command_list_matches_contract" => {
            Some(check_docs_dev_command_list_matches_contract)
        }
        "checks_crates_bijux_atlas_reserved_verbs_exclude_dev" => {
            Some(check_crates_bijux_atlas_reserved_verbs_exclude_dev)
        }
        "checks_crates_bijux_dev_atlas_not_umbrella_binary" => {
            Some(check_crates_bijux_dev_atlas_not_umbrella_binary)
        }
        "checks_crates_command_namespace_ownership_unique" => {
            Some(check_crates_command_namespace_ownership_unique)
        }
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
            if rel.components().any(|c| c.as_os_str() == "tests")
                || rel.to_string_lossy().ends_with("_tests.rs")
            {
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
    let has_dev_route = content.contains("BIJUX_DEV_ATLAS") || content.contains("bijux dev atlas");
    let has_legacy_route = content.contains("./bin/atlasctl")
        || content.contains(" atlasctl ")
        || content.contains("atlasctl ops");
    if !has_dev_route || has_legacy_route {
        violations.push(violation(
            "OPS_MAKEFILE_ROUTE_INVALID",
            "makefiles/ops.mk must delegate to BIJUX_DEV_ATLAS and must not call atlasctl"
                .to_string(),
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
        if line.trim_end().ends_with('\\') {
            violations.push(violation(
                "OPS_MAKEFILE_SINGLE_LINE_RECIPE_REQUIRED",
                "makefiles/ops.mk wrapper recipes must be single-line delegations".to_string(),
                "replace multi-line shell blocks with one-line bijux delegations",
                Some(rel),
            ));
        }
        if line.contains("$(BIJUX_DEV_ATLAS)") {
            let words = line.split_whitespace().collect::<Vec<_>>();
            if words.iter().any(|word| {
                *word == "python"
                    || *word == "python3"
                    || *word == "bash"
                    || *word == "sh"
                    || *word == "helm"
                    || *word == "kubectl"
                    || *word == "k6"
            }) {
                violations.push(violation(
                    "OPS_MAKEFILE_DELEGATION_ONLY_VIOLATION",
                    format!("makefiles/ops.mk wrapper must not invoke tools directly: `{line}`"),
                    "delegate to bijux dev atlas ops commands only",
                    Some(rel),
                ));
            }
            continue;
        }
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words
            .iter()
            .any(|word| *word == "kubectl" || *word == "helm" || *word == "kind" || *word == "k6")
        {
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

fn check_make_governance_wrappers_bijux_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("makefiles/ci.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();

    if !content.contains("BIJUX ?= bijux") || !content.contains("BIJUX_DEV_ATLAS ?=") {
        violations.push(violation(
            "MAKE_GOVERNANCE_BIJUX_VARIABLES_MISSING",
            "makefiles/ci.mk must declare BIJUX and BIJUX_DEV_ATLAS variables".to_string(),
            "declare BIJUX ?= bijux and BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas",
            Some(rel),
        ));
    }

    for line in content.lines().filter(|line| line.starts_with('\t')) {
        if line.trim_end().ends_with('\\') {
            violations.push(violation(
                "MAKE_GOVERNANCE_SINGLE_LINE_RECIPE_REQUIRED",
                "makefiles/ci.mk wrapper recipes must be single-line delegations".to_string(),
                "replace multi-line shell blocks with one-line make/bijux delegations",
                Some(rel),
            ));
        }
        if line.contains("atlasctl") {
            violations.push(violation(
                "MAKE_GOVERNANCE_ATLASCTL_REFERENCE_FOUND",
                format!("makefiles/ci.mk must not call atlasctl: `{line}`"),
                "route governance wrappers through make dev-doctor/dev-check-ci or bijux dev atlas",
                Some(rel),
            ));
        }
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.iter().any(|word| {
            *word == "python"
                || *word == "python3"
                || *word == "bash"
                || *word == "sh"
                || *word == "helm"
                || *word == "kubectl"
                || *word == "k6"
        }) {
            violations.push(violation(
                "MAKE_GOVERNANCE_DELEGATION_ONLY_VIOLATION",
                format!("makefiles/ci.mk must remain delegation-only: `{line}`"),
                "wrapper recipes may call make or bijux only",
                Some(rel),
            ));
        }
        if !(line.contains("$(BIJUX") || line.contains("$(MAKE)")) {
            violations.push(violation(
                "MAKE_GOVERNANCE_ENTRYPOINT_INVALID",
                format!("makefiles/ci.mk wrapper must call make/bijux only: `{line}`"),
                "use $(MAKE) or $(BIJUX_DEV_ATLAS) in ci wrapper recipes",
                Some(rel),
            ));
        }
    }

    Ok(violations)
}

fn check_workflows_governance_entrypoints_bijux_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".github/workflows/atlas-dev-rust.yml");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();

    for token in ["atlasctl", "scripts/areas"] {
        if content.contains(token) {
            violations.push(violation(
                "WORKFLOW_GOVERNANCE_LEGACY_REFERENCE_FOUND",
                format!("atlas-dev-rust workflow contains forbidden token `{token}`"),
                "route governance workflow through make/bijux dev atlas only",
                Some(rel),
            ));
        }
    }

    for required in [
        "make dev-doctor",
        "make dev-check-ci",
        "make ops-doctor",
        "make ops-validate",
        "bijux-dev-atlas -- doctor --format json",
    ] {
        if !content.contains(required) {
            violations.push(violation(
                "WORKFLOW_GOVERNANCE_ENTRYPOINT_MISSING",
                format!("atlas-dev-rust workflow is missing `{required}`"),
                "keep governance workflow checks routed through make and bijux dev atlas",
                Some(rel),
            ));
        }
    }

    Ok(violations)
}

fn check_make_governance_wrappers_no_direct_cargo(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for rel_text in ["makefiles/dev.mk", "makefiles/ci.mk", "makefiles/ops.mk"] {
        let rel = Path::new(rel_text);
        let path = ctx.repo_root.join(rel);
        let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
        for line in content.lines().filter(|line| line.starts_with('\t')) {
            let trimmed = line.trim();
            if trimmed.contains(" cargo ") || trimmed.starts_with("@cargo ") || trimmed.starts_with("cargo ")
            {
                violations.push(violation(
                    "MAKE_GOVERNANCE_DIRECT_CARGO_FORBIDDEN",
                    format!("{rel_text} must not call cargo directly in governance wrappers: `{trimmed}`"),
                    "route governance wrappers through make or bijux dev atlas entrypoints",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

fn check_docs_runtime_command_list_matches_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-atlas-cli/docs/CLI_COMMAND_LIST.md");
    let contract = Path::new("docs/contracts/CLI_COMMANDS.json");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let raw = fs::read_to_string(ctx.repo_root.join(contract))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let payload: serde_json::Value =
        serde_json::from_str(&raw).map_err(|err| CheckError::Failed(err.to_string()))?;
    let commands = payload
        .get("commands")
        .and_then(|v| v.as_array())
        .ok_or_else(|| CheckError::Failed("docs/contracts/CLI_COMMANDS.json missing commands".to_string()))?;
    let expected = commands
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if current.trim() == expected.trim() {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "DOCS_RUNTIME_COMMAND_LIST_MISMATCH",
            "runtime command list doc does not match canonical command contract".to_string(),
            "regenerate or edit crates/bijux-atlas-cli/docs/CLI_COMMAND_LIST.md to match docs/contracts/CLI_COMMANDS.json",
            Some(rel),
        )])
    }
}

fn check_docs_dev_command_list_matches_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/docs/CLI_COMMAND_LIST.md");
    let help_golden = Path::new("crates/bijux-dev-atlas/tests/goldens/help.txt");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let help = fs::read_to_string(ctx.repo_root.join(help_golden))
        .map_err(|err| CheckError::Failed(err.to_string()))?;

    let mut commands = Vec::new();
    let mut in_commands = false;
    for line in help.lines() {
        if line.trim() == "Commands:" {
            in_commands = true;
            continue;
        }
        if in_commands {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("Options:") {
                break;
            }
            let cmd = trimmed.split_whitespace().next().unwrap_or_default();
            if !cmd.is_empty() && cmd != "help" {
                commands.push(cmd.to_string());
            }
        }
    }
    let expected = commands.join("\n");
    if current.trim() == expected.trim() {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "DOCS_DEV_COMMAND_LIST_MISMATCH",
            "dev command list doc does not match dev help golden command list".to_string(),
            "update crates/bijux-dev-atlas/docs/CLI_COMMAND_LIST.md to match help snapshot commands",
            Some(rel),
        )])
    }
}

fn check_crates_bijux_atlas_reserved_verbs_exclude_dev(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-atlas-cli/tests/lib_contracts.rs");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let required = ["\" plugin\"", "\" plugins\"", "\" dev\""];
    let mut violations = Vec::new();
    for needle in required {
        if !text.contains(needle) {
            violations.push(violation(
                "CRATES_RESERVED_VERB_TEST_COVERAGE_MISSING",
                format!("reserved verb coverage in lib_contracts.rs is missing `{needle}`"),
                "extend top-level reserved verb test to include dev-reserved umbrella verbs",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

fn check_crates_bijux_dev_atlas_not_umbrella_binary(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/Cargo.toml");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    if !text.contains("name = \"bijux-dev-atlas\"") {
        violations.push(violation(
            "CRATES_DEV_ATLAS_BINARY_NAME_MISSING",
            "bijux-dev-atlas Cargo.toml must define bin name `bijux-dev-atlas`".to_string(),
            "keep dev control-plane binary separate from umbrella binary name",
            Some(rel),
        ));
    }
    if text.contains("name = \"bijux\"") {
        violations.push(violation(
            "CRATES_DEV_ATLAS_UMBRELLA_BINARY_FORBIDDEN",
            "bijux-dev-atlas crate must not define umbrella binary `bijux`".to_string(),
            "reserve `bijux` binary name for umbrella owner only",
            Some(rel),
        ));
    }
    Ok(violations)
}

fn check_crates_command_namespace_ownership_unique(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let runtime_rel = Path::new("crates/bijux-atlas-cli/docs/CLI_COMMAND_LIST.md");
    let dev_rel = Path::new("crates/bijux-dev-atlas/docs/CLI_COMMAND_LIST.md");
    let runtime = fs::read_to_string(ctx.repo_root.join(runtime_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let dev = fs::read_to_string(ctx.repo_root.join(dev_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;

    let runtime_first = runtime
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let dev_first = dev
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();

    let overlap = runtime_first
        .intersection(&dev_first)
        .filter(|v| **v != "version")
        .cloned()
        .collect::<Vec<_>>();
    if overlap.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_COMMAND_NAMESPACE_OWNERSHIP_DUPLICATE",
            format!(
                "runtime and dev command surfaces have duplicate namespace ownership: {}",
                overlap.join(", ")
            ),
            "keep runtime and dev command surface ownership disjoint (except shared global version semantics)",
            Some(dev_rel),
        )])
    }
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

fn check_root_packages_atlasctl_absent(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("packages/atlasctl");
    if ctx.adapters.fs.exists(ctx.repo_root, rel) {
        Ok(vec![Violation {
            code: "ROOT_PACKAGES_ATLASCTL_STILL_PRESENT".to_string(),
            message: "legacy packages/atlasctl directory still exists".to_string(),
            hint: Some("remove packages/atlasctl after migration closure".to_string()),
            path: Some(rel.display().to_string()),
            line: None,
            severity: Severity::Warn,
        }])
    } else {
        Ok(Vec::new())
    }
}

fn check_docs_no_atlasctl_string_references(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_string_references_under(ctx, "docs", "atlasctl", "DOCS_ATLASCTL_REFERENCE_FOUND")
}

fn check_workflows_no_atlasctl_string_references(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_string_references_under(
        ctx,
        ".github/workflows",
        "atlasctl",
        "WORKFLOW_ATLASCTL_REFERENCE_FOUND",
    )
}

fn check_make_no_atlasctl_string_references(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_string_references_under(ctx, "makefiles", "atlasctl", "MAKE_ATLASCTL_REFERENCE_FOUND")
}

fn check_workflows_no_direct_ops_script_execution(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_any_string_references_under(
        ctx,
        ".github/workflows",
        &["bash ops/", "sh ops/", "./ops/"],
        "WORKFLOW_DIRECT_OPS_SCRIPT_EXECUTION_FOUND",
    )
}

fn check_make_no_direct_ops_script_execution(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_no_any_string_references_under(
        ctx,
        "makefiles",
        &["bash ops/", "sh ops/", "./ops/"],
        "MAKE_DIRECT_OPS_SCRIPT_EXECUTION_FOUND",
    )
}

fn check_root_no_scripts_areas_presence_or_references(
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
        let nested =
            check_no_string_references_under(ctx, rel, "scripts/areas", "ROOT_SCRIPTS_AREAS_REFERENCE_FOUND")?;
        violations.extend(nested);
    }
    Ok(violations)
}

fn check_crates_bijux_atlas_cli_owns_umbrella_dispatch(
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
        if rel == Path::new("crates/bijux-dev-atlas-core/src/checks/ops.rs") {
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

fn check_crates_bijux_atlas_help_excludes_dev_commands(
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

fn check_crates_bijux_dev_atlas_help_dispatch_present(
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

fn check_no_string_references_under(
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
        let ext = file.extension().and_then(|v| v.to_str()).unwrap_or_default();
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
                    format!("forbidden `{needle}` reference in {}: `{}`", rel.display(), line.trim()),
                    "remove legacy references and route through bijux dev atlas",
                    Some(rel),
                ));
                break;
            }
        }
    }
    Ok(violations)
}

fn check_no_any_string_references_under(
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
        let ext = file.extension().and_then(|v| v.to_str()).unwrap_or_default();
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

fn check_ops_no_bash_lib_execution(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for rel in ["makefiles", ".github/workflows"] {
        let root = ctx.repo_root.join(rel);
        if !root.exists() {
            continue;
        }
        for file in walk_files(&root) {
            let ext = file.extension().and_then(|v| v.to_str()).unwrap_or_default();
            if !OPS_TEXT_EXTENSIONS.contains(&ext) && ext != "mk" && ext != "yml" && ext != "yaml" {
                continue;
            }
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            for line in content.lines() {
                let trimmed = line.trim();
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

fn check_ops_legacy_shell_quarantine_empty(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("ops/quarantine/legacy-ops-shell");
    let dir = ctx.repo_root.join(rel);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut non_marker = Vec::new();
    for file in walk_files(&dir) {
        let name = file.file_name().and_then(|v| v.to_str()).unwrap_or_default();
        if name != ".gitkeep" && name != "README.md" {
            non_marker.push(file);
        }
    }
    if non_marker.is_empty() {
        Ok(Vec::new())
    } else {
        let first = non_marker[0].strip_prefix(ctx.repo_root).unwrap_or(&non_marker[0]);
        Ok(vec![violation(
            "OPS_LEGACY_SHELL_QUARANTINE_NOT_EMPTY",
            format!("legacy ops shell quarantine must be empty; found `{}`", first.display()),
            "delete legacy shell helpers and keep quarantine empty",
            Some(rel),
        )])
    }
}
