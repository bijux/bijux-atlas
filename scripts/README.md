# Scripts

Categories:
- `scripts/areas/`: internal script areas by concern
- `scripts/lib/`: shared script libraries
- `scripts/bin/`: thin entrypoints only

Policy: scripts are internal unless listed in `configs/ops/public-surface.json` or the `public` section in `scripts/areas/docs/ENTRYPOINTS.md`.

## Full Inventory

# Scripts Index

Generated file. Do not edit manually.

| Script | Owner | Stability | Called By |
|---|---|---|---|
| `scripts/areas/_internal/check-root-layout.sh` | `internal` | `internal` | - |
| `scripts/areas/_internal/run_suite_wrapper_legacy.sh` | `internal` | `internal` | - |
| `scripts/areas/_meta/ownership.json` | `platform` | `internal` | - |
| `scripts/areas/bootstrap/install_tools.sh` | `developer-experience` | `public` | `bootstrap-tools` |
| `scripts/areas/check/check-atlas-scripts-cli-contract.py` | `platform` | `public` | - |
| `scripts/areas/check/check-bijux-atlas-scripts-boundaries.py` | `platform` | `public` | - |
| `scripts/areas/check/check-bin-entrypoints.py` | `platform` | `public` | - |
| `scripts/areas/check/check-docker-image-size.py` | `platform` | `public` | `docker-contracts` |
| `scripts/areas/check/check-docker-layout.py` | `platform` | `public` | `docker-contracts` |
| `scripts/areas/check/check-docker-policy.py` | `platform` | `public` | `docker-contracts` |
| `scripts/areas/check/check-no-adhoc-python.py` | `platform` | `public` | - |
| `scripts/areas/check/check-no-direct-python-invocations.py` | `platform` | `public` | - |
| `scripts/areas/check/check-no-latest-tags.py` | `platform` | `public` | `docker-contracts` |
| `scripts/areas/check/check-no-make-scripts-references.py` | `platform` | `public` | - |
| `scripts/areas/check/check-no-python-executable-outside-tools.py` | `platform` | `public` | - |
| `scripts/areas/check/check-python-lock.py` | `platform` | `public` | - |
| `scripts/areas/check/check-python-migration-exceptions-expiry.py` | `platform` | `public` | - |
| `scripts/areas/check/check-repo-script-boundaries.py` | `platform` | `public` | - |
| `scripts/areas/check/check-script-errors.py` | `platform` | `public` | - |
| `scripts/areas/check/check-script-help.py` | `platform` | `public` | - |
| `scripts/areas/check/check-script-ownership.py` | `platform` | `public` | - |
| `scripts/areas/check/check-script-shim-expiry.py` | `platform` | `public` | - |
| `scripts/areas/check/check-script-shims-minimal.py` | `platform` | `public` | - |
| `scripts/areas/check/check-script-tool-guards.py` | `platform` | `public` | - |
| `scripts/areas/check/check-script-write-roots.py` | `platform` | `public` | - |
| `scripts/areas/check/check-scripts-lock-sync.py` | `platform` | `public` | - |
| `scripts/areas/check/check-scripts-ssot-final.py` | `platform` | `public` | - |
| `scripts/areas/check/check-scripts-surface-docs-drift.py` | `platform` | `public` | - |
| `scripts/areas/check/check_duplicate_script_names.py` | `platform` | `public` | - |
| `scripts/areas/check/docker-runtime-smoke.sh` | `platform` | `public` | - |
| `scripts/areas/check/docker-scan.sh` | `platform` | `public` | - |
| `scripts/areas/check/generate-scripts-sbom.py` | `platform` | `public` | - |
| `scripts/areas/check/no-direct-path-usage.sh` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/areas/check/no-duplicate-script-names.sh` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/areas/check/python_migration_exceptions.py` | `platform` | `public` | - |
| `scripts/areas/ci/scripts-ci.sh` | `platform` | `internal` | - |
| `scripts/areas/configs/check_config_files_well_formed.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_config_keys_docs_coverage.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_config_ownership.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_configs_readmes.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_docs_links_for_configs.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_duplicate_threshold_sources.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_generated_configs_drift.sh` | `platform` | `public` | `configs-gen-check` |
| `scripts/areas/configs/check_no_adhoc_versions.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_openapi_snapshot_generated.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_ops_env_usage_declared.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_perf_thresholds_drift.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_root_config_shims.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_slo_sync.py` | `platform` | `public` | - |
| `scripts/areas/configs/check_tool_versions_doc_drift.py` | `platform` | `public` | - |
| `scripts/areas/configs/generate_configs_index.py` | `platform` | `public` | - |
| `scripts/areas/configs/generate_configs_surface.py` | `platform` | `public` | - |
| `scripts/areas/configs/generate_env_contract.py` | `platform` | `public` | - |
| `scripts/areas/configs/generate_tooling_versions_doc.py` | `platform` | `public` | - |
| `scripts/areas/configs/sync_slo_config.py` | `platform` | `public` | - |
| `scripts/areas/configs/validate_configs_schemas.py` | `platform` | `public` | - |
| `scripts/areas/demo/demo.sh` | `platform` | `private` | - |
| `scripts/areas/docs/ENTRYPOINTS.md` | `docs-governance` | `public` | - |
| `scripts/areas/docs/ban_legacy_terms.sh` | `docs-governance` | `public` | `docs-build`, `docs-lint-names` |
| `scripts/areas/docs/check-durable-naming.py` | `docs-governance` | `public` | `rename-lint` |
| `scripts/areas/docs/check_adr_headers.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_broken_examples.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_concept_ids.sh` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_concept_registry.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_configmap_env_docs.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_contract_doc_pairs.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/areas/docs/check_contracts_index_nav.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_crate_docs_contract.sh` | `docs-governance` | `public` | `crate-docs-contract`, `crate-structure`, `docs-build` |
| `scripts/areas/docs/check_critical_make_targets_referenced.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_doc_filename_style.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_doc_naming.sh` | `docs-governance` | `public` | `docs-build` |
| `scripts/areas/docs/check_docker_entrypoints.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_docs_deterministic.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_docs_freeze_drift.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_docs_make_only.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_docs_make_only_ops.py` | `docs-governance` | `public` | `ci-docs-make-only-ops` |
| `scripts/areas/docs/check_docs_make_targets_exist.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_duplicate_topics.sh` | `docs-governance` | `public` | `docs-build`, `rename-lint` |
| `scripts/areas/docs/check_example_configs.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_full_stack_page.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_generated_contract_docs.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_index_pages.sh` | `docs-governance` | `public` | `docs-build`, `docs-lint-names` |
| `scripts/areas/docs/check_k8s_docs_contract.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_load_docs_contract.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_make_help_drift.py` | `docs-governance` | `public` | `ci-make-help-drift` |
| `scripts/areas/docs/check_make_targets_documented.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_make_targets_drift.py` | `docs-governance` | `public` | `ops-make-targets-doc` |
| `scripts/areas/docs/check_mkdocs_site_links.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_nav_order.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_no_legacy_root_paths.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_no_orphan_docs.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/areas/docs/check_no_placeholders.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_no_removed_make_targets.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_observability_acceptance_checklist.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_observability_docs_checklist.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/areas/docs/check_observability_surface_drift.py` | `docs-governance` | `public` | `ops-observability-validate` |
| `scripts/areas/docs/check_openapi_examples.py` | `docs-governance` | `public` | `ops-api-smoke`, `ops-openapi-validate` |
| `scripts/areas/docs/check_ops_doc_duplication.py` | `docs-governance` | `public` | `ci-ops-doc-duplication` |
| `scripts/areas/docs/check_ops_docs_make_targets.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_ops_observability_links.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_ops_readme_canonical_links.py` | `docs-governance` | `public` | `ci-ops-readme-canonical-links` |
| `scripts/areas/docs/check_ops_readmes_make_only.py` | `docs-governance` | `public` | `ci-ops-readme-make-only` |
| `scripts/areas/docs/check_public_surface_docs.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_public_targets_docs_sections.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_reference_templates.sh` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_runbook_map_registration.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/areas/docs/check_runbooks_contract.py` | `docs-governance` | `public` | `ops-drill-runner` |
| `scripts/areas/docs/check_script_headers.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_script_locations.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/areas/docs/check_suite_id_docs.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_terminology_units_ssot.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/check_title_case.sh` | `docs-governance` | `public` | `docs-build` |
| `scripts/areas/docs/extract_code_blocks.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_architecture_map.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_chart_contract_index.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_concept_graph.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_config_keys_doc.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_contracts_index_doc.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_crates_map.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_env_vars_doc.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_k8s_install_matrix.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_k8s_values_doc.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_layer_contract_doc.py` | `docs-governance` | `public` | `ops-contracts-check`, `ops-gen` |
| `scripts/areas/docs/generate_make_targets_catalog.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_make_targets_inventory.py` | `docs-governance` | `public` | `ops-make-targets-doc` |
| `scripts/areas/docs/generate_makefiles_surface.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_observability_surface.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_openapi_docs.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_ops_badge.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_ops_contracts_doc.py` | `docs-governance` | `public` | `ops-contracts-check`, `ops-gen` |
| `scripts/areas/docs/generate_ops_schema_docs.py` | `docs-governance` | `public` | `ops-contracts-check`, `ops-gen` |
| `scripts/areas/docs/generate_ops_surface.py` | `docs-governance` | `public` | `ops-contracts-check`, `ops-gen` |
| `scripts/areas/docs/generate_repo_surface.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_runbook_map_index.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_scripts_graph.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/generate_sli_doc.py` | `docs-governance` | `public` | `ci-sli-docs-drift` |
| `scripts/areas/docs/generate_slos_doc.py` | `docs-governance` | `public` | `ci-slo-docs-drift` |
| `scripts/areas/docs/generate_upgrade_guide.py` | `docs-governance` | `public` | `upgrade-guide` |
| `scripts/areas/docs/legacy-terms-allowlist.txt` | `docs-governance` | `public` | - |
| `scripts/areas/docs/lint_depth.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/lint_doc_contracts.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/lint_doc_status.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/lint_glossary_links.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/naming_inventory.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/areas/docs/render_diagrams.sh` | `docs-governance` | `public` | `docs-build` |
| `scripts/areas/docs/rewrite_legacy_terms.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/run_blessed_snippets.py` | `docs-governance` | `public` | - |
| `scripts/areas/docs/spellcheck_docs.py` | `docs-governance` | `public` | - |
| `scripts/areas/fixtures/derive-release-111.sh` | `dataset-ops` | `internal` | - |
| `scripts/areas/fixtures/fetch-medium.sh` | `dataset-ops` | `public` | `fetch-fixtures`, `ops-publish-medium` |
| `scripts/areas/fixtures/fetch-real-datasets.sh` | `dataset-ops` | `public` | `fetch-real-datasets` |
| `scripts/areas/fixtures/run-medium-ingest.sh` | `dataset-ops` | `public` | `ingest-sharded-medium`, `run-medium-ingest` |
| `scripts/areas/fixtures/run-medium-serve.sh` | `dataset-ops` | `public` | `run-medium-serve` |
| `scripts/areas/gen/generate_scripts_readme.py` | `platform` | `public` | - |
| `scripts/areas/gen/generate_scripts_surface.py` | `platform` | `public` | - |
| `scripts/areas/internal/__init__.py` | `platform` | `internal` | - |
| `scripts/areas/internal/effects-lint.sh` | `platform` | `internal` | - |
| `scripts/areas/internal/env_dump.sh` | `platform` | `internal` | - |
| `scripts/areas/internal/exec.sh` | `platform` | `internal` | - |
| `scripts/areas/internal/migrate_paths.sh` | `platform` | `internal` | - |
| `scripts/areas/internal/naming-intent-lint.sh` | `platform` | `internal` | - |
| `scripts/areas/internal/openapi-generate.sh` | `platform` | `internal` | - |
| `scripts/areas/internal/paths.py` | `platform` | `internal` | - |
| `scripts/areas/internal/repo_root.sh` | `platform` | `internal` | - |
| `scripts/areas/layout/allowed_root.json` | `repo-surface` | `public` | - |
| `scripts/areas/layout/build_artifacts_index.py` | `repo-surface` | `public` | `artifacts-index` |
| `scripts/areas/layout/build_run_artifact_index.py` | `repo-surface` | `public` | `ops-artifacts-index-run` |
| `scripts/areas/layout/check_artifacts_allowlist.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_artifacts_policy.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_cargo_dev_metadata.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_cargo_invocations_scoped.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_chart_canonical_path.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_ci_entrypoints.py` | `repo-surface` | `public` | `ci-workflow-contract` |
| `scripts/areas/layout/check_dataset_manifest_lock.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_dir_budgets.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_duplicate_script_intent.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_e2e_scenarios.py` | `repo-surface` | `public` | `ops-e2e-validate` |
| `scripts/areas/layout/check_e2e_suites.py` | `repo-surface` | `public` | `ops-e2e-validate` |
| `scripts/areas/layout/check_evidence_not_tracked.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_forbidden_root_files.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_forbidden_root_names.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_generated_committed_no_timestamp_dirs.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_generated_dirs_policy.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_generated_policy.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_help_excludes_internal.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_help_output_determinism.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_help_snapshot.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_internal_targets_not_in_docs.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_kind_cluster_contract_drift.sh` | `repo-surface` | `public` | `ops-kind-cluster-drift-check` |
| `scripts/areas/layout/check_layer_drift.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_legacy_deprecation.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_make_command_allowlist.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_make_lane_reports.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_make_public_scripts.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_make_safety.py` | `repo-surface` | `public` | `ci-make-safety`, `path-contract-check` |
| `scripts/areas/layout/check_make_target_ownership.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_make_targets_catalog_drift.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_makefile_headers.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_makefile_target_boundaries.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_makefiles_contract.py` | `repo-surface` | `public` | `makefiles-contract`, `release` |
| `scripts/areas/layout/check_makefiles_index_drift.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_no_dead_entrypoints.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_no_direct_script_runs.sh` | `repo-surface` | `public` | `_lint-configs`, `no-direct-scripts` |
| `scripts/areas/layout/check_no_empty_dirs.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_no_forbidden_paths.sh` | `repo-surface` | `public` | `ci-forbid-raw-paths`, `layout-check`, `path-contract-check` |
| `scripts/areas/layout/check_no_hidden_defaults.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_no_legacy_target_names.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_no_legacy_targets_in_docs.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_no_mixed_script_name_variants.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_no_ops_evidence_writes.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_no_orphan_configs.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_no_orphan_docs_refs.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_no_orphan_owners.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_no_root_dumping.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_obs_pack_ssot.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_obs_script_name_collisions.py` | `repo-surface` | `public` | `ops-observability-validate` |
| `scripts/areas/layout/check_obs_suites.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_ops_artifacts_writes.py` | `repo-surface` | `public` | `layout-check`, `ops-layout-lint` |
| `scripts/areas/layout/check_ops_budgets.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_ops_canonical_entrypoints.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_ops_canonical_shims.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_ops_concept_ownership.py` | `repo-surface` | `public` | `layout-check`, `ops-layout-lint` |
| `scripts/areas/layout/check_ops_cross_area_script_refs.py` | `repo-surface` | `public` | `ops-lint-all` |
| `scripts/areas/layout/check_ops_external_entrypoints.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_ops_index_surface.py` | `repo-surface` | `public` | `ci-ops-index-surface`, `layout-check` |
| `scripts/areas/layout/check_ops_layout_contract.py` | `repo-surface` | `public` | `layout-check`, `ops-layout-lint` |
| `scripts/areas/layout/check_ops_lib_canonical.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_ops_pins.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_ops_run_entrypoints.py` | `repo-surface` | `public` | `ci-ops-run-entrypoints` |
| `scripts/areas/layout/check_ops_script_names.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_ops_script_targets.sh` | `repo-surface` | `public` | `ops-script-coverage` |
| `scripts/areas/layout/check_ops_shell_policy.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_ops_single_owner_contracts.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_ops_single_validators.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_ops_stack_order.sh` | `repo-surface` | `public` | `ops-stack-order-check`, `ops-stack-validate` |
| `scripts/areas/layout/check_ops_surface_drift.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/areas/layout/check_ops_workspace.sh` | `repo-surface` | `public` | `layout-check`, `ops-layout-lint` |
| `scripts/areas/layout/check_public_entrypoint_cap.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_public_surface.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_public_target_aliases.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_public_target_budget.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_public_target_descriptions.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_public_targets_documented.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_realdata_scenarios.py` | `repo-surface` | `public` | `ops-e2e-validate` |
| `scripts/areas/layout/check_repo_hygiene.sh` | `repo-surface` | `public` | `_check`, `_coverage`, `_fmt`, `_lint-configs`, `_test`, `_test-all`, `_test-contracts`, `layout-check` |
| `scripts/areas/layout/check_root_determinism.sh` | `repo-surface` | `public` | `root-determinism` |
| `scripts/areas/layout/check_root_diff_alarm.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_root_local_lane_isolation.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_root_makefile_hygiene.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_root_mk_size_budget.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_root_no_cargo_dev_deps.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_root_shape.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_script_entrypoints.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_script_naming_convention.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_script_relative_calls.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_scripts_buckets.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_scripts_readme_drift.sh` | `repo-surface` | `public` | `_lint-configs` |
| `scripts/areas/layout/check_scripts_submodules.py` | `repo-surface` | `public` | `ops-lint-all` |
| `scripts/areas/layout/check_scripts_top_level.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/check_slo_contracts.py` | `repo-surface` | `public` | `ci-sli-contract`, `ci-slo-config-validate`, `ci-slo-metrics-contract` |
| `scripts/areas/layout/check_slo_no_loosen_without_approval.py` | `repo-surface` | `public` | `ci-slo-no-loosen` |
| `scripts/areas/layout/check_stack_manifest_consolidation.sh` | `repo-surface` | `public` | `ops-stack-validate` |
| `scripts/areas/layout/check_symlink_index.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_symlink_policy.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/areas/layout/check_tool_versions.py` | `repo-surface` | `public` | `ops-helm-version-check`, `ops-jq-version-check`, `ops-k6-version-check`, `ops-kind-version-check`, `ops-kubectl-version-check`, `ops-yq-version-check` |
| `scripts/areas/layout/check_workflows_make_only.py` | `repo-surface` | `public` | `ci-workflows-make-only`, `layout-check` |
| `scripts/areas/layout/clean_artifacts.py` | `repo-surface` | `public` | `artifacts-clean` |
| `scripts/areas/layout/clean_make_artifacts.py` | `repo-surface` | `public` | `clean-all`, `clean-safe` |
| `scripts/areas/layout/clean_ops_generated.py` | `repo-surface` | `public` | `ops-gen-clean` |
| `scripts/areas/layout/dataset_id_lint.py` | `repo-surface` | `public` | `dataset-id-lint` |
| `scripts/areas/layout/evidence_check.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/evidence_clean.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/evidence_pr_summary.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/explain_public_target.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/generate_ops_pins.py` | `repo-surface` | `public` | `ops-gen` |
| `scripts/areas/layout/generate_ops_stack_versions.py` | `repo-surface` | `public` | `ops-stack-versions-sync` |
| `scripts/areas/layout/generate_ops_surface_meta.py` | `repo-surface` | `public` | `ops-gen` |
| `scripts/areas/layout/graph_public_target.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/legacy_audit.sh` | `repo-surface` | `public` | - |
| `scripts/areas/layout/legacy_inventory.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/list_internal_targets.py` | `repo-surface` | `public` | `internal-list` |
| `scripts/areas/layout/make_doctor.py` | `repo-surface` | `public` | `doctor` |
| `scripts/areas/layout/make_prereqs.py` | `repo-surface` | `public` | `prereqs` |
| `scripts/areas/layout/make_report.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/make_target_graph.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/migrate.sh` | `repo-surface` | `public` | `layout-migrate` |
| `scripts/areas/layout/public_make_targets.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/public_surface.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/render_public_help.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/replace_paths.sh` | `repo-surface` | `public` | `layout-migrate` |
| `scripts/areas/layout/root_whitelist.json` | `repo-surface` | `public` | - |
| `scripts/areas/layout/run_gate.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/update_ops_pins.py` | `repo-surface` | `public` | - |
| `scripts/areas/layout/validate_ops_contracts.py` | `repo-surface` | `public` | `ops-contracts-check`, `ops-gen`, `ops-k8s-contracts` |
| `scripts/areas/layout/validate_ops_env.py` | `repo-surface` | `public` | `ops-env-print`, `ops-env-validate` |
| `scripts/areas/layout/write_make_area_report.py` | `repo-surface` | `public` | - |
| `scripts/areas/ops/check_k8s_checks_layout.py` | `platform` | `public` | - |
| `scripts/areas/ops/check_k8s_flakes.py` | `platform` | `public` | `ops-k8s-tests` |
| `scripts/areas/ops/check_k8s_test_contract.py` | `platform` | `public` | `ops-k8s-tests` |
| `scripts/areas/ops/check_k8s_test_lib.py` | `platform` | `public` | - |
| `scripts/areas/ops/generate_k8s_test_surface.py` | `platform` | `public` | - |
| `scripts/areas/policy/find_relaxations.sh` | `platform` | `internal` | - |
| `scripts/areas/public/check-allow-env-schema.py` | `platform` | `public` | `policy-allow-env-lint` |
| `scripts/areas/public/check-cli-commands.sh` | `platform` | `public` | `cli-command-surface` |
| `scripts/areas/public/check-markdown-links.sh` | `platform` | `public` | `_lint-docs`, `docs-build` |
| `scripts/areas/public/config-drift-check.py` | `platform` | `public` | - |
| `scripts/areas/public/config-print.py` | `platform` | `public` | - |
| `scripts/areas/public/config-validate.py` | `platform` | `public` | - |
| `scripts/areas/public/contracts/check_breaking_contract_change.py` | `platform` | `public` | `api-contract-check`, `ci-openapi-drift` |
| `scripts/areas/public/contracts/check_endpoints_contract.py` | `platform` | `public` | `api-contract-check` |
| `scripts/areas/public/contracts/check_error_codes_contract.py` | `platform` | `public` | `api-contract-check` |
| `scripts/areas/public/contracts/check_sqlite_indexes_contract.py` | `platform` | `public` | `critical-query-check` |
| `scripts/areas/public/contracts/check_v1_surface.py` | `platform` | `public` | `api-contract-check` |
| `scripts/areas/public/contracts/gen_openapi.py` | `platform` | `public` | `api-contract-check` |
| `scripts/areas/public/generate-config-key-registry.py` | `platform` | `public` | - |
| `scripts/areas/public/no-network-unit-tests.sh` | `platform` | `public` | - |
| `scripts/areas/public/observability/check_alerts_contract.py` | `platform` | `public` | `ops-alerts-validate`, `ops-metrics-check` |
| `scripts/areas/public/observability/check_dashboard_contract.py` | `platform` | `public` | `ops-dashboards-validate`, `ops-metrics-check` |
| `scripts/areas/public/observability/check_metrics_contract.py` | `platform` | `public` | `ops-metrics-check`, `ops-observability-validate` |
| `scripts/areas/public/observability/check_runtime_metrics.py` | `platform` | `public` | `ops-metrics-check` |
| `scripts/areas/public/observability/check_tracing_contract.py` | `platform` | `public` | `ops-observability-validate`, `ops-traces-check` |
| `scripts/areas/public/observability/lint_runbooks.py` | `platform` | `public` | `ops-metrics-check` |
| `scripts/areas/public/openapi-diff-check.sh` | `platform` | `public` | `api-contract-check`, `openapi-drift`, `ops-openapi-validate` |
| `scripts/areas/public/ops-policy-audit.py` | `platform` | `public` | - |
| `scripts/areas/public/perf/check_baseline_update_policy.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/check_percent_regression.py` | `platform` | `public` | `stack-full` |
| `scripts/areas/public/perf/check_pinned_queries_lock.py` | `platform` | `public` | - |
| `scripts/areas/public/perf/check_prereqs.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/check_regression.py` | `platform` | `public` | - |
| `scripts/areas/public/perf/check_runbook_suite_names.py` | `platform` | `public` | - |
| `scripts/areas/public/perf/check_spike_assertions.py` | `platform` | `public` | `ops-load-spike-proof` |
| `scripts/areas/public/perf/cold_start_benchmark.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/cold_start_prefetch_5pods.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/compare_redis.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/generate_report.py` | `platform` | `public` | - |
| `scripts/areas/public/perf/load_under_rollback.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/load_under_rollout.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/prepare_perf_store.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/run_critical_queries.py` | `platform` | `public` | `critical-query-check` |
| `scripts/areas/public/perf/run_e2e_perf.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/run_nightly_perf.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/run_suite.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/run_suites_from_manifest.py` | `platform` | `public` | - |
| `scripts/areas/public/perf/score_k6.py` | `platform` | `public` | - |
| `scripts/areas/public/perf/update_baseline.sh` | `platform` | `public` | - |
| `scripts/areas/public/perf/validate_results.py` | `platform` | `public` | - |
| `scripts/areas/public/perf/validate_suite_manifest.py` | `platform` | `public` | - |
| `scripts/areas/public/policy-audit.py` | `platform` | `public` | `policy-audit` |
| `scripts/areas/public/policy-drift-diff.sh` | `platform` | `public` | `policy-drift-diff` |
| `scripts/areas/public/policy-enforcement-status.py` | `platform` | `public` | `policy-enforcement-status` |
| `scripts/areas/public/policy-lint.sh` | `platform` | `public` | `_lint-configs`, `policy-lint` |
| `scripts/areas/public/policy-schema-drift.py` | `platform` | `public` | `policy-schema-drift` |
| `scripts/areas/public/qc-fixtures-gate.sh` | `platform` | `public` | `ci-qc-fixtures` |
| `scripts/areas/public/query-plan-gate.sh` | `platform` | `public` | `query-plan-gate` |
| `scripts/areas/public/report-bundle.sh` | `platform` | `public` | `ops-report` |
| `scripts/areas/public/require-crate-docs.sh` | `platform` | `public` | `crate-structure` |
| `scripts/areas/public/stack/build_stack_report.py` | `platform` | `public` | `stack-full` |
| `scripts/areas/public/stack/validate_stack_report.py` | `platform` | `public` | `stack-full` |
| `scripts/areas/python/__init__.py` | `platform` | `internal` | - |
| `scripts/areas/python/bijux_scripts/__init__.py` | `platform` | `internal` | - |
| `scripts/areas/python/bijux_scripts/json_helpers.py` | `platform` | `internal` | - |
| `scripts/areas/python/bijux_scripts/paths.py` | `platform` | `internal` | - |
| `scripts/areas/python/bijux_scripts/reporting.py` | `platform` | `internal` | - |
| `scripts/areas/python/bijux_scripts/runner.py` | `platform` | `internal` | - |
| `scripts/areas/python/requirements.lock.txt` | `platform` | `internal` | - |
| `scripts/areas/release/update-compat-matrix.sh` | `release-engineering` | `public` | `release-update-compat-matrix` |
| `scripts/areas/release/validate-compat-matrix.sh` | `release-engineering` | `public` | `compat-matrix-validate` |
| `scripts/areas/tests/test_paths.py` | `platform` | `internal` | - |
| `scripts/areas/tools/__init__.py` | `platform` | `internal` | - |
| `scripts/areas/tools/json_helpers.py` | `platform` | `internal` | - |
| `scripts/areas/tools/path_utils.py` | `platform` | `internal` | - |
| `scripts/areas/tools/reporting.py` | `platform` | `internal` | - |
| `scripts/bin/bijux-atlas-dev` | `platform` | `public` | - |
| `scripts/bin/bijux-atlas-ops` | `platform` | `internal` | - |
| `scripts/bin/bijux-atlas-scripts` | `platform` | `public` | `ci-log-fields-contract`, `docs-build`, `docs-check`, `docs-freeze`, `docs-lint-names`, `docs-req-lock-refresh`, `layout-check`, `observability-pack-drills`, `observability-pack-test`, `ops-catalog-validate`, `ops-check`, `ops-dataset-qc-diff`, `ops-lint`, `ops-lint-all`, `ops-load-ci`, `ops-load-full`, `ops-load-manifest-validate`, `ops-load-nightly`, `ops-load-shedding`, `ops-load-smoke`, `ops-load-soak`, `ops-load-spike-proof`, `ops-local-full`, `ops-metrics-check`, `ops-observability-pack-conformance-report`, `ops-observability-validate`, `ops-perf-report`, `ops-slo-burn` |
| `scripts/bin/isolate` | `platform` | `public` | `bench-db-size-growth`, `bench-ingest-throughput-medium`, `bench-smoke`, `bench-sqlite-query-latency`, `check`, `coverage`, `test-all`, `test-contracts` |
| `scripts/bin/make_explain` | `platform` | `internal` | - |
| `scripts/bin/make_graph` | `platform` | `internal` | - |
| `scripts/bin/render_public_help` | `platform` | `internal` | - |
| `scripts/bin/require-isolate` | `platform` | `public` | `_audit`, `_bench-db-size-growth`, `_bench-ingest-throughput-medium`, `_bench-sqlite-query-latency`, `_check`, `_coverage`, `_fmt`, `_lint-clippy`, `_lint-configs`, `_lint-docs`, `_lint-rustfmt`, `_test`, `_test-all`, `_test-contracts`, `bench-db-size-growth`, `bench-ingest-throughput-medium`, `bench-smoke`, `bench-sqlite-query-latency`, `check`, `cli-command-surface`, `coverage`, `crate-docs-contract`, `crate-structure`, `test-all`, `test-contracts` |
| `scripts/bin/run_drill.sh` | `platform` | `internal` | - |
| `scripts/lib/errors.sh` | `platform` | `internal` | - |
