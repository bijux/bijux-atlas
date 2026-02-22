# Checks Index

Generated from `packages/atlasctl/src/atlasctl/registry/checks_catalog.json`.

## checks

- `checks_checks_registry_change_docs_gate`: require docs update when checks registry changes
- `checks_checks_registry_change_goldens_gate`: require checks list/tree/owners goldens update when registry changes
- `checks_checks_registry_change_owners_gate`: require ownership metadata update when checks registry changes
- `checks_checks_registry_integrity`: validate checks registry TOML/JSON integrity and drift

## configs

- `checks_configs_pyproject_no_duplicate_tool_config`: forbid duplicate pyproject tool blocks
- `checks_configs_pyproject_required_blocks`: validate required pyproject blocks

## contracts

- `checks_contracts_atlasctl_cli`: validate atlasctl CLI contract surface
- `checks_contracts_layout_contract`: validate repository layout contract
- `checks_contracts_schema_catalog`: validate schema catalog for duplicate names and missing files
- `checks_contracts_schema_catalog_files_exist`: ensure schemas listed in catalog exist on disk
- `checks_contracts_schema_catalog_referenced`: ensure schema catalog contains only referenced schemas
- `checks_contracts_schema_catalog_sorted`: ensure schema catalog order is deterministic
- `checks_contracts_schema_catalog_ssot`: enforce contracts catalog.json as schema SSOT
- `checks_contracts_schema_change_release_notes`: enforce schema bump + release notes policy for schema changes
- `checks_contracts_schema_disk_files_listed`: ensure no schema file exists outside catalog
- `checks_contracts_schema_goldens`: validate JSON goldens against declared schemas
- `checks_contracts_schema_id_naming`: enforce schema id naming/version suffix policy
- `checks_contracts_schema_readme_updated`: ensure schemas README stays in sync with schema catalog/files
- `checks_contracts_schema_samples`: validate sample payloads against declared schemas

## docker

- `checks_docker_image_size`: enforce docker image size budget
- `checks_docker_layout_contract`: validate docker layout contracts
- `checks_docker_no_latest_tags`: forbid floating latest image tags
- `checks_docker_policy_contract`: validate docker policy contracts

## docs

- `checks_docs_check_id_drift`: forbid docs references to unknown check ids
- `checks_docs_command_group_pages`: require command-group docs pages with examples section
- `checks_docs_index_complete`: validate docs/index.md covers all docs files
- `checks_docs_links_exist`: validate docs markdown link targets exist
- `checks_docs_lint_style`: enforce docs style lint policy
- `checks_docs_migration_not_stale`: forbid stale migration wording once removals are active
- `checks_docs_nav_references_exist`: validate mkdocs nav references existing docs pages
- `checks_docs_new_command_workflow`: require docs/tests updates when command registry changes
- `checks_docs_no_ops_generated_run_path_refs`: forbid runtime ops generated path references in docs
- `checks_docs_no_orphans`: forbid orphan docs files outside allowed generated/meta paths
- `checks_docs_no_placeholder_release_docs`: forbid placeholder docs paths in release docs tree
- `checks_docs_no_scripts_path_refs`: forbid scripts/ path references in docs
- `checks_docs_ownership_metadata`: require docs ownership metadata for major docs areas
- `checks_docs_package_root_markdown`: forbid markdown files at package root except README
- `checks_docs_registry_command_drift`: forbid docs references to unknown atlasctl commands
- `checks_docs_registry_indexes`: require registry-generated command/check/suite index pages to be in sync
- `checks_docs_stable_command_examples`: require stable commands to have examples in group docs

## license

- `checks_license_file_mit`: ensure package license file exists and is MIT
- `checks_license_spdx_policy`: enforce SPDX policy for python source headers
- `checks_license_statements_consistent`: ensure package docs contain only MIT-compatible license statements

## make

- `checks_make_ci_entrypoints_contract`: validate CI workflow entrypoints contract
- `checks_make_ci_workflows_make_only`: require CI workflows to call make entrypoints and keep atlasctl delegation in makefiles
- `checks_make_command_allowlist`: enforce allowed direct recipe commands
- `checks_make_forbidden_paths`: forbid direct forbidden paths in make recipes
- `checks_make_help_determinism`: ensure deterministic make help output
- `checks_make_index_drift_contract`: enforce makefiles index drift contract
- `checks_make_lane_reports_via_atlasctl_reporting`: require lane reports to be emitted through atlasctl reporting command
- `checks_make_no_bypass_atlasctl`: forbid make recipes bypassing atlasctl without explicit allowlist
- `checks_make_no_direct_artifact_writes`: forbid direct artifact writes in make recipes
- `checks_make_no_direct_bash_ops`: forbid direct `bash ops/...` invocations in make recipes
- `checks_make_no_direct_python`: forbid direct python script execution in make recipes
- `checks_make_no_direct_python_only_atlasctl`: forbid direct python invocations in make recipes
- `checks_make_no_direct_script_exec_drift`: drift check for direct script execution in make recipes
- `checks_make_no_direct_scripts_only_atlasctl`: forbid direct script path invocations in make recipes
- `checks_make_no_python_module_invocation`: forbid `python -m atlasctl.cli` in make recipes
- `checks_make_public_target_atlasctl_mapping`: require every public make target to delegate to atlasctl
- `checks_make_public_targets_documented`: require documented public make targets
- `checks_make_root_budget`: enforce root.mk LOC/target-count budget
- `checks_make_scripts_refs`: forbid scripts/ references in make recipes
- `checks_make_target_boundaries_enforced`: enforce make target boundary contracts
- `checks_make_target_ownership_complete`: require make target ownership coverage
- `checks_make_wrapper_purity`: enforce make wrapper purity for canonical makefiles

## ops

- `checks_ops_committed_generated_hygiene`: validate deterministic committed generated assets
- `checks_ops_manifests_schema`: validate ops manifests against atlas.ops.manifest.v1 schema
- `checks_ops_no_tracked_generated`: forbid tracked files in generated ops dirs
- `checks_ops_no_tracked_timestamps`: forbid tracked timestamped paths

## python

- `checks_python_lock_lockfile`: validate python lock format
- `checks_python_migration_exceptions_expiry`: fail on expired python migration exceptions

## repo

- `checks_repo_budget_drift_approval`: forbid budget loosening without explicit approval marker
- `checks_repo_canonical_concept_homes`: forbid duplicate top-level concept packages for registry/runner/contracts/output
- `checks_repo_check_impl_no_cli_imports`: forbid direct CLI imports from check implementation files
- `checks_repo_check_test_coverage`: ensure each registered check has test or golden coverage marker
- `checks_repo_checks_canonical_location`: require check implementations under atlasctl/checks canonical tree
- `checks_repo_checks_domain_split`: require canonical check domain tree (repo_shape/makefiles/ops/docs/observability/artifacts)
- `checks_repo_checks_import_lint`: enforce checks module import boundaries
- `checks_repo_checks_no_cli_imports`: forbid checks modules from importing cli layer
- `checks_repo_checks_root_contract`: enforce checks root contract (single python root file and module cap)
- `checks_repo_cli_argparse_policy`: restrict direct argparse parser construction to canonical parser modules
- `checks_repo_cli_canonical_paths`: enforce canonical cli parser/dispatch/output path
- `checks_repo_cli_import_scope`: restrict cli imports to commands/core and approved runtime shims
- `checks_repo_cold_import_budget`: enforce cold import time budget for atlasctl package
- `checks_repo_command_alias_budget`: enforce command alias/name budget
- `checks_repo_command_group_owners`: ensure every command group maps to a valid owner id
- `checks_repo_command_metadata_contract`: ensure command metadata includes touches/tools/effect declarations
- `checks_repo_command_module_cli_intent`: enforce command.py modules as CLI entry shims only
- `checks_repo_command_ownership_docs`: ensure command owners are documented
- `checks_repo_command_scripts_registry`: enforce command-invoked shell scripts are registered with owner and valid naming
- `checks_repo_command_test_coverage`: ensure each command has explicit test coverage marker
- `checks_repo_commands_help_docs_drift`: check command help/docs drift
- `checks_repo_commands_import_lint`: enforce command module import boundaries
- `checks_repo_commands_surface_stability`: enforce strict command surface compatibility against commands golden
- `checks_repo_compileall_gate`: ensure atlasctl source compiles with compileall
- `checks_repo_console_script_entry`: ensure atlasctl console script entry exists and points to callable target
- `checks_repo_contract_import_boundaries`: enforce contracts/core-contracts import boundaries
- `checks_repo_contracts_namespace_purpose`: enforce contracts namespace for schema and validation modules only
- `checks_repo_core_no_bash_subprocess`: forbid subprocess.run(['bash'|'sh', ...]) in core logic
- `checks_repo_core_no_command_imports`: forbid core modules from importing command/cli layers
- `checks_repo_dead_module_reachability`: enforce dead module candidates are explicitly allowlisted
- `checks_repo_dead_modules`: ensure dead modules analyzer runs and returns canonical payload
- `checks_repo_dependency_declarations`: ensure pyproject dependency declarations match imports
- `checks_repo_dependency_gate_targets`: ensure dependency gate make targets exist
- `checks_repo_dependency_owner_justification`: ensure each dependency has owner and justification
- `checks_repo_deps_command_surface`: ensure atlasctl deps command surface is runnable
- `checks_repo_deps_workflow_doc`: ensure docs/deps.md matches chosen dependency workflow
- `checks_repo_dir_budget_entries`: enforce per-directory entries budget in atlasctl src/tests
- `checks_repo_dir_budget_exceptions_documented`: ensure budget exceptions are documented
- `checks_repo_dir_budget_exceptions_sorted`: ensure budget exceptions are sorted deterministically
- `checks_repo_dir_budget_loc`: enforce per-directory total LOC budget
- `checks_repo_dir_budget_modules`: enforce per-directory python module count budget
- `checks_repo_dir_budget_py_files`: enforce per-directory python file count budget
- `checks_repo_dir_budget_py_scope`: enforce per-directory .py file budget in atlasctl src/tests (excluding __init__.py)
- `checks_repo_dir_budget_shell_files`: enforce per-directory shell file count budget
- `checks_repo_dir_count_trend_gate`: forbid critical directory file/module count drift above baseline
- `checks_repo_docs_no_ops_generated_refs`: disallow docs refs to ops generated runtime paths
- `checks_repo_duplicate_contract_assertions`: forbid duplicate contract assertions inside one test module
- `checks_repo_duplicate_script_names`: forbid duplicate script stem names
- `checks_repo_effect_boundaries`: forbid direct subprocess/fs/env/network effects outside core boundaries
- `checks_repo_effect_boundary_exceptions_policy`: enforce explicit sorted effect boundary exceptions with reasons
- `checks_repo_env_docs_present`: ensure docs/env.md exists and lists canonical env vars
- `checks_repo_file_complexity_budget`: enforce complexity heuristic budget in core/cli
- `checks_repo_file_import_budget`: enforce python import count budget per file
- `checks_repo_file_public_symbol_budget`: enforce public symbol budget per module
- `checks_repo_folder_intent_contract`: require intent marker for checks directories
- `checks_repo_forbidden_adjectives`: forbid banned wording across tracked repository files
- `checks_repo_forbidden_root_files`: forbid junk files at repository root
- `checks_repo_forbidden_root_names`: forbid legacy top-level root names
- `checks_repo_forbidden_top_dirs`: forbid top-level forbidden directories
- `checks_repo_import_smoke`: ensure atlasctl package imports in minimal environment
- `checks_repo_internal_commands_not_public`: ensure unstable/internal commands are not exposed in public docs
- `checks_repo_internal_import_boundaries`: forbid atlasctl.internal imports outside internal namespace
- `checks_repo_internal_utils_stdlib_only`: prefer stdlib-only internal utility modules
- `checks_repo_json_golden_conflicts`: forbid conflicting JSON goldens for the same schema surface
- `checks_repo_json_goldens_validate_schema`: require schema validation in JSON golden tests
- `checks_repo_layout_domain_alias_cleanup`: forbid deprecated layout registry aliases
- `checks_repo_layout_domain_readmes`: ensure each layout check domain includes a README
- `checks_repo_layout_no_legacy_imports`: forbid legacy imports in checks/layout modules
- `checks_repo_layout_no_shadow`: enforce no-shadow config policy
- `checks_repo_legacy_package_absent`: require atlasctl legacy package to be absent
- `checks_repo_legacy_zero_importers`: require zero importers of removed atlasctl legacy namespace
- `checks_repo_managed_artifact_write_roots`: enforce managed write roots under artifacts and reject out-of-root writes
- `checks_repo_modern_no_legacy_imports`: forbid modern modules from importing atlasctl.legacy
- `checks_repo_modern_no_legacy_obs_imports`: forbid modern modules from importing atlasctl.legacy.obs
- `checks_repo_module_budget_domains`: enforce module count budget per top-level atlasctl domain
- `checks_repo_module_size`: enforce module size budget
- `checks_repo_modules_reachability`: ensure repo check modules are imported and reachable via registry
- `checks_repo_network_default_deny`: enforce network-forbidden-by-default command policy
- `checks_repo_no_adhoc_python`: forbid ad-hoc python files outside package boundaries
- `checks_repo_no_core_integration_dir`: forbid deprecated core/integration namespace
- `checks_repo_no_deprecated_commands`: forbid deprecated command surfaces before 0.1 release
- `checks_repo_no_deprecated_namespace_dirs`: forbid deprecated atlasctl/check report obs directories
- `checks_repo_no_deprecated_namespaces`: forbid imports from atlasctl.check/report/obs namespaces
- `checks_repo_no_direct_bash_invocations`: forbid direct bash script calls in docs/makefiles
- `checks_repo_no_direct_python_invocations`: forbid direct python script calls in docs/makefiles
- `checks_repo_no_direct_script_runs`: forbid direct scripts/ or ops/ invocations in GitHub workflows
- `checks_repo_no_duplicate_command_impl_patterns`: forbid duplicate command implementation patterns
- `checks_repo_no_duplicate_command_names`: ensure command names are unique
- `checks_repo_no_empty_dirs_or_pointless_nests`: forbid empty directories and pointless single-child nesting under src/atlasctl
- `checks_repo_no_empty_packages`: forbid empty non-legacy packages without README
- `checks_repo_no_exec_python_outside_packages`: forbid executable python outside package boundaries
- `checks_repo_no_forbidden_paths`: forbid legacy root path references in tracked text surfaces
- `checks_repo_no_legacy_command_names`: forbid command names containing legacy
- `checks_repo_no_legacy_module_paths`: forbid legacy module paths in atlasctl src tree
- `checks_repo_no_nested_same_name_packages`: forbid nested package segments with same name
- `checks_repo_no_ops_generated_placeholder`: forbid placeholder generated dirs
- `checks_repo_no_path_cwd_usage`: forbid Path.cwd usage outside core.runtime.repo_root.py
- `checks_repo_no_placeholder_module_names`: forbid placeholder-like module filenames in modern code
- `checks_repo_no_scripts_dir`: forbid legacy root scripts dir
- `checks_repo_no_tracked_ops_generated`: ensure ops/_generated has no tracked files
- `checks_repo_no_tracked_timestamp_paths`: forbid timestamp-like tracked paths
- `checks_repo_no_undocumented_help_commands`: forbid undocumented commands in public help surface
- `checks_repo_no_wildcard_exports`: forbid wildcard imports/exports outside public surface
- `checks_repo_no_xtask_refs`: forbid xtask references
- `checks_repo_ops_examples_immutable`: enforce immutability of ops examples
- `checks_repo_optional_dependency_groups`: ensure required pyproject optional-dependency groups exist and are non-empty
- `checks_repo_optional_dependency_usage_gates`: forbid optional dependency usage without explicit allowlist gate
- `checks_repo_output_format_stability`: enforce output formatting stability across OS
- `checks_repo_package_has_module_or_readme`: require each non-legacy package to contain a module or README
- `checks_repo_package_max_depth`: enforce maximum atlasctl package nesting depth
- `checks_repo_packages_atlasctl_root_shape`: require atlasctl package root to contain only allowed files and directories
- `checks_repo_packaging_metadata_completeness`: ensure pyproject packaging metadata is complete (classifiers and project URLs)
- `checks_repo_public_api_doc_exists`: require canonical PUBLIC_API.md documentation file
- `checks_repo_public_api_exports`: enforce docs/PUBLIC_API.md coverage for __all__ exports
- `checks_repo_public_commands_docs_index`: ensure public commands are listed in docs/commands/index.md
- `checks_repo_pyproject_minimalism`: forbid dead/unknown pyproject tool keys
- `checks_repo_pyproject_no_duplicate_tool_config`: forbid duplicate tool config files beside pyproject
- `checks_repo_pyproject_required_blocks`: ensure pyproject contains required project and tool config blocks
- `checks_repo_python_module_help`: ensure python -m atlasctl --help works
- `checks_repo_python_requires_version_and_ci`: enforce pyproject requires-python and CI python version alignment
- `checks_repo_registry_definition_boundary`: forbid registry modules from importing command/cli/suite/check runtime modules
- `checks_repo_registry_single_source`: enforce registry-only command/check registration points
- `checks_repo_required_target_shape`: enforce package root minimalism and src no-symlink target shape
- `checks_repo_requirements_artifact_policy`: ensure only route-B requirements artifacts exist
- `checks_repo_requirements_sync`: ensure requirements files match pyproject dev dependency declarations
- `checks_repo_root_determinism`: verify deterministic root output across two make root runs
- `checks_repo_root_shape`: enforce repository root shape contract from root_whitelist.json
- `checks_repo_runcontext_single_builder`: ensure RunContext is built only in core/context.py
- `checks_repo_script_help_coverage`: validate script help contract coverage
- `checks_repo_script_ownership_coverage`: validate script ownership coverage
- `checks_repo_shell_docs_present`: require shell directory README and policy docs
- `checks_repo_shell_invocation_boundary`: forbid direct shell subprocess invocations outside core.exec
- `checks_repo_shell_location_policy`: forbid shell scripts under atlasctl python package tree
- `checks_repo_shell_no_direct_python`: forbid direct python invocation in shell scripts
- `checks_repo_shell_no_network_fetch`: forbid direct curl/wget usage in shell scripts
- `checks_repo_shell_readonly_checks`: forbid direct file writes in layout shell checks
- `checks_repo_shell_script_budget`: cap total shell script count in repository
- `checks_repo_shell_strict_mode`: require shell header and strict mode
- `checks_repo_single_registry_module`: enforce a single canonical registry.py module
- `checks_repo_single_runner_module`: enforce a single canonical runner.py module
- `checks_repo_stable_command_no_breaking_changes`: forbid breaking changes in stable command contracts
- `checks_repo_subprocess_boundary`: restrict subprocess imports to core execution boundary
- `checks_repo_suite_inventory_policy`: validate suite inventory coverage and required tag policy
- `checks_repo_suite_marker_rules`: enforce check-suite-coverage marker file policy
- `checks_repo_test_determinism_patterns`: forbid nondeterministic test patterns without explicit marker
- `checks_repo_test_no_duplicated_coverage`: forbid duplicated golden coverage across multiple tests
- `checks_repo_test_no_unmarked_network`: forbid test network usage unless marked
- `checks_repo_test_ownership_tags`: ensure tests declare ownership tags or live in domain directories
- `checks_repo_test_skip_justification`: forbid skipped tests without justification and expiry markers
- `checks_repo_test_taxonomy_layout`: enforce test taxonomy and placement invariants
- `checks_repo_test_write_sandbox`: forbid writes outside temp/artifacts isolation sandbox in tests
- `checks_repo_tests_no_duplicate_expectations`: forbid duplicate test function names across test modules
- `checks_repo_top_level_package_group_mapping`: require top-level atlasctl packages to map to control-plane groups
- `checks_repo_top_level_structure`: enforce atlasctl top-level package intent and budget
- `checks_repo_type_coverage`: enforce minimum type coverage in core/contracts
- `checks_repo_version_matches_pyproject`: ensure atlasctl package version and --version output match pyproject
- `checks_repo_workflows_targets_exist`: ensure make targets used by workflows exist
