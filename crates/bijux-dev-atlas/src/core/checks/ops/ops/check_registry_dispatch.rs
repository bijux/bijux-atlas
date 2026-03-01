pub fn builtin_ops_check_fn(check_id: &CheckId) -> Option<CheckFn> {
    match check_id.as_str() {
        "checks_ops_surface_manifest" => Some(check_ops_surface_manifest),
        "checks_ops_tree_contract" => Some(checks_ops_tree_contract),
        "checks_ops_generated_readonly_markers" => Some(checks_ops_generated_readonly_markers),
        "checks_ops_generated_lifecycle_metadata" => Some(checks_ops_generated_lifecycle_metadata),
        "checks_ops_schema_presence" => Some(checks_ops_schema_presence),
        "checks_ops_manifest_integrity" => Some(checks_ops_manifest_integrity),
        "checks_ops_surface_inventory" => Some(checks_ops_surface_inventory),
        "checks_ops_artifacts_not_tracked" => Some(checks_ops_artifacts_not_tracked),
        "checks_ops_retired_artifact_path_references_absent" => {
            Some(checks_ops_retired_artifact_path_references_absent)
        }
        "checks_ops_runtime_output_roots_under_ops_absent" => {
            Some(checks_ops_runtime_output_roots_under_ops_absent)
        }
        "checks_ops_no_scripts_areas_or_xtask_refs" => {
            Some(checks_ops_no_scripts_areas_or_xtask_refs)
        }
        "checks_ops_artifacts_gitignore_policy" => Some(checks_ops_artifacts_gitignore_policy),
        "checks_ops_makefile_routes_dev_atlas" => Some(checks_ops_makefile_routes_dev_atlas),
        "checks_ops_workflow_routes_dev_atlas" => Some(checks_ops_workflow_routes_dev_atlas),
        "checks_ops_workflows_github_actions_pinned" => {
            Some(checks_ops_workflows_github_actions_pinned)
        }
        "checks_ops_image_references_digest_pinned" => {
            Some(checks_ops_image_references_digest_pinned)
        }
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
        "checks_ops_no_makefiles" => Some(check_ops_no_makefiles),
        "checks_ops_no_direct_tool_invocations" => Some(check_ops_no_direct_tool_invocations),
        "checks_ops_quarantine_shim_expiration_contract" => {
            Some(check_ops_quarantine_shim_expiration_contract)
        }
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
        "checks_docs_command_list_matches_contract" => {
            Some(check_docs_command_list_matches_contract)
        }
        "checks_docs_dev_command_list_matches_contract" => {
            Some(check_docs_dev_command_list_matches_contract)
        }
        "checks_docs_mkdocs_yaml_parseable" => Some(check_docs_mkdocs_yaml_parseable),
        "checks_docs_mkdocs_nav_files_exist" => Some(check_docs_mkdocs_nav_files_exist),
        "checks_docs_no_orphan_markdown_pages" => Some(check_docs_no_orphan_markdown_pages),
        "checks_docs_no_duplicate_nav_titles" => Some(check_docs_no_duplicate_nav_titles),
        "checks_docs_markdown_link_targets_exist" => Some(check_docs_markdown_link_targets_exist),
        "checks_docs_markdown_directory_budgets" => Some(check_docs_markdown_directory_budgets),
        "checks_docs_index_reachability_ledger" => Some(check_docs_index_reachability_ledger),
        "checks_docs_ops_operations_duplicate_titles" => {
            Some(check_docs_ops_operations_duplicate_titles)
        }
        "checks_docs_near_duplicate_filenames" => Some(check_docs_near_duplicate_filenames),
        "checks_docs_operations_directory_index_contract" => {
            Some(check_docs_operations_directory_index_contract)
        }
        "checks_docs_operations_canonical_concept_paths" => {
            Some(check_docs_operations_canonical_concept_paths)
        }
        "checks_docs_operations_verify_command_quality" => {
            Some(check_docs_operations_verify_command_quality)
        }
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
        "checks_ops_required_files_contracts" => Some(check_ops_required_files_contracts),
        "checks_ops_domain_contract_structure" => Some(check_ops_domain_contract_structure),
        "checks_ops_inventory_contract_integrity" => Some(check_ops_inventory_contract_integrity),
        "checks_ops_file_usage_and_orphan_contract" => {
            Some(check_ops_file_usage_and_orphan_contract)
        }
        "checks_ops_docs_governance" => Some(check_ops_docs_governance),
        "checks_ops_evidence_bundle_discipline" => Some(check_ops_evidence_bundle_discipline),
        "checks_ops_fixture_governance" => Some(check_ops_fixture_governance),
        "checks_ops_portability_environment_contract" => {
            Some(checks_ops_portability_environment_contract)
        }
        "checks_ops_minimalism_and_deletion_safety" => {
            Some(checks_ops_minimalism_and_deletion_safety)
        }
        "checks_ops_human_workflow_maturity" => Some(checks_ops_human_workflow_maturity),
        "checks_ops_final_polish_contracts" => Some(checks_ops_final_polish_contracts),
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
        "checks_repo_artifacts_not_tracked" => Some(check_repo_artifacts_not_tracked),
        "checks_repo_defaults_work_surface_contract" => {
            Some(check_repo_defaults_work_surface_contract)
        }
        "checks_repo_duplicate_ssot_registries_absent" => {
            Some(check_repo_duplicate_ssot_registries_absent)
        }
        "checks_repo_generated_content_stays_in_allowed_paths" => {
            Some(check_repo_generated_content_stays_in_allowed_paths)
        }
        "checks_repo_law_metadata_complete_and_unique" => {
            Some(check_repo_law_metadata_complete_and_unique)
        }
        "checks_repo_no_executable_script_sources" => Some(check_repo_no_executable_script_sources),
        "checks_repo_pr_required_suite_not_skippable" => {
            Some(check_repo_pr_required_suite_not_skippable)
        }
        "checks_repo_root_directory_allowlist_contract" => {
            Some(check_repo_root_directory_allowlist_contract)
        }
        "checks_repo_root_markdown_allowlist_contract" => {
            Some(check_repo_root_markdown_allowlist_contract)
        }
        "checks_repo_registry_order_deterministic" => Some(check_repo_registry_order_deterministic),
        "checks_repo_suite_includes_p0_checks" => Some(check_repo_suite_includes_p0_checks),
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
        "checks_ops_generated_lifecycle_metadata",
        "checks_ops_schema_presence",
        "checks_ops_manifest_integrity",
        "checks_ops_surface_inventory",
        "checks_ops_artifacts_not_tracked",
        "checks_ops_retired_artifact_path_references_absent",
        "checks_ops_runtime_output_roots_under_ops_absent",
        "checks_ops_no_scripts_areas_or_xtask_refs",
        "checks_ops_artifacts_gitignore_policy",
        "checks_ops_makefile_routes_dev_atlas",
        "checks_ops_workflow_routes_dev_atlas",
        "checks_ops_workflows_github_actions_pinned",
        "checks_ops_image_references_digest_pinned",
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
        "checks_ops_no_makefiles",
        "checks_ops_no_direct_tool_invocations",
        "checks_ops_quarantine_shim_expiration_contract",
        "checks_root_no_scripts_areas_presence_or_references",
        "checks_crates_bijux_atlas_cli_owns_umbrella_dispatch",
        "checks_crates_bijux_atlas_help_excludes_dev_commands",
        "checks_crates_bijux_dev_atlas_help_dispatch_present",
        "checks_ops_no_bash_lib_execution",
        "checks_make_governance_wrappers_bijux_only",
        "checks_workflows_governance_entrypoints_bijux_only",
        "checks_make_governance_wrappers_no_direct_cargo",
        "checks_docs_command_list_matches_contract",
        "checks_docs_dev_command_list_matches_contract",
        "checks_docs_mkdocs_yaml_parseable",
        "checks_docs_mkdocs_nav_files_exist",
        "checks_docs_no_orphan_markdown_pages",
        "checks_docs_no_duplicate_nav_titles",
        "checks_docs_markdown_link_targets_exist",
        "checks_docs_markdown_directory_budgets",
        "checks_docs_index_reachability_ledger",
        "checks_docs_ops_operations_duplicate_titles",
        "checks_docs_near_duplicate_filenames",
        "checks_docs_operations_directory_index_contract",
        "checks_docs_operations_canonical_concept_paths",
        "checks_docs_operations_verify_command_quality",
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
        "checks_ops_required_files_contracts",
        "checks_ops_domain_contract_structure",
        "checks_ops_inventory_contract_integrity",
        "checks_ops_file_usage_and_orphan_contract",
        "checks_ops_docs_governance",
        "checks_ops_evidence_bundle_discipline",
        "checks_ops_fixture_governance",
        "checks_ops_portability_environment_contract",
        "checks_ops_minimalism_and_deletion_safety",
        "checks_ops_human_workflow_maturity",
        "checks_ops_final_polish_contracts",
        "checks_crates_dev_atlas_final_crate_set_contract",
        "checks_docs_scripting_contract_rust_control_plane_lock",
        "checks_crates_bijux_atlas_reserved_verbs_exclude_dev",
        "checks_crates_bijux_dev_atlas_not_umbrella_binary",
        "checks_crates_command_namespace_ownership_unique",
        "checks_crates_plugin_conformance_binaries",
        "checks_repo_artifacts_not_tracked",
        "checks_repo_defaults_work_surface_contract",
        "checks_repo_duplicate_ssot_registries_absent",
        "checks_repo_generated_content_stays_in_allowed_paths",
        "checks_repo_law_metadata_complete_and_unique",
        "checks_repo_no_executable_script_sources",
        "checks_repo_pr_required_suite_not_skippable",
        "checks_repo_root_directory_allowlist_contract",
        "checks_repo_root_markdown_allowlist_contract",
        "checks_repo_registry_order_deterministic",
        "checks_repo_suite_includes_p0_checks",
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
