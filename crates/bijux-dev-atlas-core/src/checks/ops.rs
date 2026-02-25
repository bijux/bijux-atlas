// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use bijux_dev_atlas_model::{ArtifactPath, CheckId, Severity, Violation, ViolationId};
use serde_yaml::Value as YamlValue;

use crate::{CheckContext, CheckError, CheckFn};

const OPS_TEXT_EXTENSIONS: [&str; 5] = ["md", "json", "toml", "yaml", "yml"];
mod documentation_and_config_checks;
mod governance_checks;
mod governance_repo_checks;
mod surface_contract_checks;
use documentation_and_config_checks::*;
use governance_checks::*;
use governance_repo_checks::*;
use surface_contract_checks::*;

pub fn builtin_ops_check_fn(check_id: &CheckId) -> Option<CheckFn> {
    match check_id.as_str() {
        "checks_ops_surface_manifest" => Some(check_ops_surface_manifest),
        "checks_ops_tree_contract" => Some(checks_ops_tree_contract),
        "checks_ops_generated_readonly_markers" => Some(checks_ops_generated_readonly_markers),
        "checks_ops_schema_presence" => Some(checks_ops_schema_presence),
        "checks_ops_manifest_integrity" => Some(checks_ops_manifest_integrity),
        "checks_ops_surface_inventory" => Some(checks_ops_surface_inventory),
        "checks_ops_artifacts_not_tracked" => Some(checks_ops_artifacts_not_tracked),
        "checks_ops_no_scripts_areas_or_xtask_refs" => {
            Some(checks_ops_no_scripts_areas_or_xtask_refs)
        }
        "checks_ops_artifacts_gitignore_policy" => Some(checks_ops_artifacts_gitignore_policy),
        "checks_ops_makefile_routes_dev_atlas" => Some(checks_ops_makefile_routes_dev_atlas),
        "checks_ops_workflow_routes_dev_atlas" => Some(checks_ops_workflow_routes_dev_atlas),
        "checks_make_ops_wrappers_delegate_dev_atlas" => {
            Some(check_make_ops_wrappers_delegate_dev_atlas)
        }
        "checks_workflows_ops_entrypoints_bijux_only" => {
            Some(check_workflows_ops_entrypoints_bijux_only)
        }
        "checks_ops_internal_registry_consistency" => Some(check_ops_internal_registry_consistency),
        "checks_root_python_toolchain_toml_absent" => Some(check_root_python_toolchain_toml_absent),
        "checks_root_uv_lock_absent" => Some(check_root_uv_lock_absent),
        "checks_workflows_no_direct_ops_script_execution" => {
            Some(check_workflows_no_direct_ops_script_execution)
        }
        "checks_make_no_direct_ops_script_execution" => {
            Some(check_make_no_direct_ops_script_execution)
        }
        "checks_makefiles_no_cd_invocations" => Some(check_makefiles_no_cd_invocations),
        "checks_makefiles_no_direct_tool_invocations" => {
            Some(check_makefiles_no_direct_tool_invocations)
        }
        "checks_makefiles_no_direct_fetch_commands" => {
            Some(check_makefiles_no_direct_fetch_commands)
        }
        "checks_makefiles_no_multiline_recipes" => Some(check_makefiles_no_multiline_recipes),
        "checks_root_dockerignore_context_contract" => {
            Some(check_root_dockerignore_context_contract)
        }
        "checks_root_dockerfile_pointer_only" => Some(check_root_dockerfile_pointer_only),
        "checks_dockerfiles_under_canonical_directory_only" => {
            Some(check_dockerfiles_under_canonical_directory_only)
        }
        "checks_workflows_no_direct_docker_build_execution" => {
            Some(check_workflows_no_direct_docker_build_execution)
        }
        "checks_ops_no_executable_bit_files" => Some(check_ops_no_executable_bit_files),
        "checks_ops_no_behavior_source_files" => Some(check_ops_no_behavior_source_files),
        "checks_root_no_scripts_areas_presence_or_references" => {
            Some(check_root_no_scripts_areas_presence_or_references)
        }
        "checks_root_forbidden_retired_directories_absent" => {
            Some(check_root_forbidden_retired_directories_absent)
        }
        "checks_root_makefile_single_include_entrypoint" => {
            Some(check_root_makefile_single_include_entrypoint)
        }
        "checks_makefiles_root_includes_sorted" => Some(check_makefiles_root_includes_sorted),
        "checks_root_top_level_directories_contract" => {
            Some(check_root_top_level_directories_contract)
        }
        "checks_root_cargo_config_contract" => Some(check_root_cargo_config_contract),
        "checks_root_rust_toolchain_toml_contract" => Some(check_root_rust_toolchain_toml_contract),
        "checks_root_rustfmt_toml_present" => Some(check_root_rustfmt_toml_present),
        "checks_root_clippy_toml_present" => Some(check_root_clippy_toml_present),
        "checks_configs_nextest_toml_present" => Some(check_configs_nextest_toml_present),
        "checks_configs_security_deny_toml_present" => {
            Some(check_configs_security_deny_toml_present)
        }
        "checks_workflows_rust_toolchain_matches_repo_pin" => {
            Some(check_workflows_rust_toolchain_matches_repo_pin)
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
        "checks_make_governance_wrappers_bijux_only" => {
            Some(check_make_governance_wrappers_bijux_only)
        }
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
        "checks_docs_mkdocs_yaml_parseable" => Some(check_docs_mkdocs_yaml_parseable),
        "checks_docs_mkdocs_nav_files_exist" => Some(check_docs_mkdocs_nav_files_exist),
        "checks_docs_no_orphan_markdown_pages" => Some(check_docs_no_orphan_markdown_pages),
        "checks_docs_no_duplicate_nav_titles" => Some(check_docs_no_duplicate_nav_titles),
        "checks_docs_readme_index_contract_presence" => {
            Some(check_docs_readme_index_contract_presence)
        }
        "checks_docs_file_naming_conventions" => Some(check_docs_file_naming_conventions),
        "checks_docs_command_surface_docs_exist" => Some(check_docs_command_surface_docs_exist),
        "checks_crate_docs_governance_contract" => Some(check_crate_docs_governance_contract),
        "checks_make_docs_wrappers_delegate_dev_atlas" => {
            Some(check_make_docs_wrappers_delegate_dev_atlas)
        }
        "checks_configs_required_surface_paths" => Some(check_configs_required_surface_paths),
        "checks_configs_schema_paths_present" => Some(check_configs_schema_paths_present),
        "checks_make_configs_wrappers_delegate_dev_atlas" => {
            Some(check_make_configs_wrappers_delegate_dev_atlas)
        }
        "checks_ops_control_plane_doc_contract" => Some(check_ops_control_plane_doc_contract),
        "checks_docs_ops_command_list_matches_snapshot" => {
            Some(check_docs_ops_command_list_matches_snapshot)
        }
        "checks_docs_configs_command_list_matches_snapshot" => {
            Some(check_docs_configs_command_list_matches_snapshot)
        }
        "checks_docs_control_plane_naming_contract" => {
            Some(check_control_plane_naming_contract_docs)
        }
        "checks_docs_removed_system_references_absent" => {
            Some(check_docs_removed_system_references_absent)
        }
        "checks_ops_ssot_manifests_schema_versions" => {
            Some(check_ops_ssot_manifests_schema_versions)
        }
        "checks_crates_dev_atlas_final_crate_set_contract" => {
            Some(check_final_dev_atlas_crate_set_contract)
        }
        "checks_docs_scripting_contract_rust_control_plane_lock" => {
            Some(check_scripting_contract_rust_control_plane_lock)
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
        "checks_crates_plugin_conformance_binaries" => {
            Some(check_crates_plugin_conformance_binaries)
        }
        "checks_root_artifacts_bin_binaries_executable_and_version_printable" => {
            Some(check_artifacts_bin_binaries_executable_and_version_printable)
        }
        _ => None,
    }
}

pub fn builtin_ops_check_ids() -> BTreeSet<String> {
    [
        "checks_ops_surface_manifest",
        "checks_ops_tree_contract",
        "checks_ops_generated_readonly_markers",
        "checks_ops_schema_presence",
        "checks_ops_manifest_integrity",
        "checks_ops_surface_inventory",
        "checks_ops_artifacts_not_tracked",
        "checks_ops_no_scripts_areas_or_xtask_refs",
        "checks_ops_artifacts_gitignore_policy",
        "checks_ops_makefile_routes_dev_atlas",
        "checks_ops_workflow_routes_dev_atlas",
        "checks_make_ops_wrappers_delegate_dev_atlas",
        "checks_workflows_ops_entrypoints_bijux_only",
        "checks_ops_internal_registry_consistency",
        "checks_root_python_toolchain_toml_absent",
        "checks_root_uv_lock_absent",
        "checks_workflows_no_direct_ops_script_execution",
        "checks_make_no_direct_ops_script_execution",
        "checks_makefiles_no_cd_invocations",
        "checks_makefiles_no_direct_tool_invocations",
        "checks_makefiles_no_direct_fetch_commands",
        "checks_makefiles_no_multiline_recipes",
        "checks_root_dockerignore_context_contract",
        "checks_root_dockerfile_pointer_only",
        "checks_dockerfiles_under_canonical_directory_only",
        "checks_workflows_no_direct_docker_build_execution",
        "checks_ops_no_executable_bit_files",
        "checks_ops_no_behavior_source_files",
        "checks_root_no_scripts_areas_presence_or_references",
        "checks_crates_bijux_atlas_cli_owns_umbrella_dispatch",
        "checks_crates_bijux_atlas_help_excludes_dev_commands",
        "checks_crates_bijux_dev_atlas_help_dispatch_present",
        "checks_ops_no_bash_lib_execution",
        "checks_make_governance_wrappers_bijux_only",
        "checks_workflows_governance_entrypoints_bijux_only",
        "checks_make_governance_wrappers_no_direct_cargo",
        "checks_docs_runtime_command_list_matches_contract",
        "checks_docs_dev_command_list_matches_contract",
        "checks_docs_mkdocs_yaml_parseable",
        "checks_docs_mkdocs_nav_files_exist",
        "checks_docs_no_orphan_markdown_pages",
        "checks_docs_no_duplicate_nav_titles",
        "checks_docs_readme_index_contract_presence",
        "checks_docs_file_naming_conventions",
        "checks_docs_command_surface_docs_exist",
        "checks_crate_docs_governance_contract",
        "checks_make_docs_wrappers_delegate_dev_atlas",
        "checks_configs_required_surface_paths",
        "checks_configs_schema_paths_present",
        "checks_make_configs_wrappers_delegate_dev_atlas",
        "checks_ops_control_plane_doc_contract",
        "checks_docs_ops_command_list_matches_snapshot",
        "checks_docs_configs_command_list_matches_snapshot",
        "checks_docs_control_plane_naming_contract",
        "checks_docs_removed_system_references_absent",
        "checks_ops_ssot_manifests_schema_versions",
        "checks_crates_dev_atlas_final_crate_set_contract",
        "checks_docs_scripting_contract_rust_control_plane_lock",
        "checks_crates_bijux_atlas_reserved_verbs_exclude_dev",
        "checks_crates_bijux_dev_atlas_not_umbrella_binary",
        "checks_crates_command_namespace_ownership_unique",
        "checks_crates_plugin_conformance_binaries",
        "checks_root_artifacts_bin_binaries_executable_and_version_printable",
        "checks_root_forbidden_retired_directories_absent",
        "checks_root_makefile_single_include_entrypoint",
        "checks_makefiles_root_includes_sorted",
        "checks_root_top_level_directories_contract",
        "checks_root_cargo_config_contract",
        "checks_root_rust_toolchain_toml_contract",
        "checks_root_rustfmt_toml_present",
        "checks_root_clippy_toml_present",
        "checks_configs_nextest_toml_present",
        "checks_configs_security_deny_toml_present",
        "checks_workflows_rust_toolchain_matches_repo_pin",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn violation(code: &str, message: String, hint: &str, path: Option<&Path>) -> Violation {
    Violation {
        schema_version: bijux_dev_atlas_model::schema_version(),
        code: ViolationId::parse(&code.to_ascii_lowercase()).expect("valid violation id"),
        message,
        hint: Some(hint.to_string()),
        path: path.map(|p| ArtifactPath::parse(&p.display().to_string()).expect("valid path")),
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
    let canonical_dirs = [
        "inventory",
        "schema",
        "env",
        "stack",
        "k8s",
        "observe",
        "load",
        "datasets",
        "e2e",
        "report",
        "_generated",
        "_generated.example",
    ];
    for dir in canonical_dirs {
        let rel = Path::new("ops").join(dir);
        if !ctx.adapters.fs.exists(ctx.repo_root, &rel) {
            violations.push(violation(
                "OPS_CANONICAL_DIRECTORY_MISSING",
                format!("missing canonical ops directory `{}`", rel.display()),
                "restore the canonical ops directory set",
                Some(&rel),
            ));
            continue;
        }
        for required_file in ["README.md", "OWNER.md", "REQUIRED_FILES.md"] {
            let target = rel.join(required_file);
            if !ctx.adapters.fs.exists(ctx.repo_root, &target) {
                violations.push(violation(
                    "OPS_CANONICAL_DIRECTORY_REQUIRED_FILE_MISSING",
                    format!(
                        "missing required file `{}` in canonical ops directory",
                        target.display()
                    ),
                    "add required README/OWNER/REQUIRED_FILES marker files for canonical ops directories",
                    Some(&target),
                ));
            }
        }
        let full = ctx.repo_root.join(&rel);
        let has_any_entry = fs::read_dir(&full)
            .ok()
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false);
        if !has_any_entry {
            violations.push(violation(
                "OPS_CANONICAL_DIRECTORY_EMPTY",
                format!("canonical ops directory is empty: `{}`", rel.display()),
                "add required marker files and committed contract content",
                Some(&rel),
            ));
        }
    }
    let allowed_top_level_dirs = BTreeSet::from([
        "_evidence",
        "_examples",
        "_generated",
        "_generated.example",
        "_meta",
        "atlas-dev",
        "datasets",
        "docs",
        "e2e",
        "env",
        "fixtures",
        "helm",
        "inventory",
        "k8s",
        "kind",
        "load",
        "manifests",
        "obs",
        "observe",
        "quarantine",
        "registry",
        "report",
        "schema",
        "schemas",
        "stack",
        "tools",
    ]);
    for entry in read_dir_entries(&ctx.repo_root.join("ops")) {
        if !entry.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !allowed_top_level_dirs.contains(name) {
            let rel = Path::new("ops").join(name);
            violations.push(violation(
                "OPS_TOP_LEVEL_DIRECTORY_FORBIDDEN",
                format!("non-canonical top-level ops directory found: `{}`", rel.display()),
                "remove stray directories or update contract and checks if the directory is intentional",
                Some(&rel),
            ));
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
        "inventory",
        "schema",
        "env",
        "stack",
        "k8s",
        "observe",
        "load",
        "datasets",
        "e2e",
        "report",
        "_generated",
        "_generated.example",
    ];
    let listed_dirs: BTreeSet<String> = index_text
        .lines()
        .filter(|line| line.trim_start().starts_with("- `ops/"))
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
    let listed_order = index_text
        .lines()
        .filter(|line| line.trim_start().starts_with("- `ops/"))
        .filter_map(|line| line.split("`ops/").nth(1))
        .filter_map(|tail| tail.split('/').next())
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let expected_order = required_entries
        .iter()
        .map(|v| (*v).to_string())
        .collect::<Vec<_>>();
    if listed_order != expected_order {
        violations.push(violation(
            "OPS_INDEX_DIRECTORY_ORDER_INVALID",
            format!(
                "ops/INDEX.md canonical directory order mismatch: listed={listed_order:?} expected={expected_order:?}"
            ),
            "list canonical ops directories in stable contract order",
            Some(index_rel),
        ));
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
