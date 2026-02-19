# Scripts

Categories:
- `scripts/check/`: validators and lint gates
- `scripts/gen/`: generators for docs and inventories
- `scripts/ci/`: CI-only glue
- `scripts/dev/`: local helpers
- `scripts/lib/`: shared script libraries
- `scripts/python/`: reusable Python modules
- `scripts/bin/`: thin entrypoints only

Policy: scripts are internal unless listed in `configs/ops/public-surface.json` or the `public` section in `scripts/ENTRYPOINTS.md`.

## Full Inventory

# Scripts Index

Generated file. Do not edit manually.

| Script | Owner | Stability | Called By |
|---|---|---|---|
| `scripts/ENTRYPOINTS.md` | `platform` | `internal` | - |
| `scripts/_internal/check-root-layout.sh` | `internal` | `internal` | - |
| `scripts/_internal/run_suite_wrapper_legacy.sh` | `internal` | `internal` | - |
| `scripts/_meta/ownership.json` | `platform` | `internal` | - |
| `scripts/bin/bijux-atlas-dev` | `platform` | `public` | - |
| `scripts/bin/bijux-atlas-ops` | `platform` | `internal` | - |
| `scripts/bin/isolate` | `platform` | `public` | `audit`, `bench-db-size-growth`, `bench-ingest-throughput-medium`, `bench-smoke`, `bench-sqlite-query-latency`, `check`, `coverage`, `fmt`, `lint`, `test`, `test-all`, `test-contracts` |
| `scripts/bin/require-isolate` | `platform` | `public` | `_audit`, `_bench-db-size-growth`, `_bench-ingest-throughput-medium`, `_bench-sqlite-query-latency`, `_check`, `_coverage`, `_fmt`, `_lint-clippy`, `_lint-configs`, `_lint-docs`, `_lint-rustfmt`, `_test`, `_test-all`, `_test-contracts`, `audit`, `bench-db-size-growth`, `bench-ingest-throughput-medium`, `bench-smoke`, `bench-sqlite-query-latency`, `check`, `cli-command-surface`, `coverage`, `crate-docs-contract`, `crate-structure`, `fmt`, `lint`, `test`, `test-all`, `test-contracts` |
| `scripts/bootstrap/install_tools.sh` | `developer-experience` | `public` | `bootstrap-tools` |
| `scripts/check/check-bin-entrypoints.py` | `platform` | `public` | `scripts-lint` |
| `scripts/check/check-docker-layout.py` | `platform` | `public` | - |
| `scripts/check/check-docker-policy.py` | `platform` | `public` | - |
| `scripts/check/check-no-latest-tags.py` | `platform` | `public` | - |
| `scripts/check/check-python-lock.py` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/check/check-script-errors.py` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/check/check-script-help.py` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/check/check-script-ownership.py` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/check/check-script-tool-guards.py` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/check/check-script-write-roots.py` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/check/docker-runtime-smoke.sh` | `platform` | `public` | `docker-smoke` |
| `scripts/check/docker-scan.sh` | `platform` | `public` | `docker-scan` |
| `scripts/check/no-direct-path-usage.sh` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/check/no-duplicate-script-names.sh` | `platform` | `public` | `scripts-check`, `scripts-lint` |
| `scripts/check_no_root_dumping.sh` | `platform` | `public` | `layout-check` |
| `scripts/ci/scripts-ci.sh` | `platform` | `internal` | - |
| `scripts/configs/check_config_ownership.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_configs_readmes.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_docs_links_for_configs.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_duplicate_threshold_sources.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_no_adhoc_versions.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_openapi_snapshot_generated.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_ops_env_usage_declared.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_perf_thresholds_drift.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_root_config_shims.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_slo_sync.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/check_tool_versions_doc_drift.py` | `platform` | `public` | `configs-check` |
| `scripts/configs/generate_configs_surface.py` | `platform` | `public` | - |
| `scripts/configs/generate_tooling_versions_doc.py` | `platform` | `public` | - |
| `scripts/configs/sync_slo_config.py` | `platform` | `public` | - |
| `scripts/configs/validate_configs_schemas.py` | `platform` | `public` | `configs-check` |
| `scripts/contracts/check_all.sh` | `contracts` | `public` | `ssot-check` |
| `scripts/contracts/check_breaking_contract_change.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_chart_values_contract.py` | `contracts` | `public` | `chart-validate`, `ops-values-validate` |
| `scripts/contracts/check_cli_ssot.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_config_keys_contract.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_contract_drift.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_endpoints_contract.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_error_codes_contract.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_sqlite_indexes_contract.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_v1_surface.py` | `contracts` | `internal` | - |
| `scripts/contracts/format_contracts.py` | `contracts` | `internal` | - |
| `scripts/contracts/gen_openapi.py` | `contracts` | `internal` | - |
| `scripts/contracts/generate_chart_values_schema.py` | `contracts` | `public` | `chart-validate`, `ops-gen`, `ops-values-validate` |
| `scripts/contracts/generate_contract_artifacts.py` | `contracts` | `public` | `telemetry-contracts` |
| `scripts/demo/demo.sh` | `platform` | `private` | - |
| `scripts/docs/ban_legacy_terms.sh` | `docs-governance` | `public` | `docs-build`, `docs-lint-names` |
| `scripts/docs/check-durable-naming.py` | `docs-governance` | `public` | `rename-lint` |
| `scripts/docs/check_adr_headers.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_broken_examples.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_concept_ids.sh` | `docs-governance` | `public` | - |
| `scripts/docs/check_concept_registry.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_contract_doc_pairs.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/docs/check_crate_docs_contract.sh` | `docs-governance` | `public` | `crate-docs-contract`, `crate-structure`, `docs-build` |
| `scripts/docs/check_critical_make_targets_referenced.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_doc_filename_style.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_doc_naming.sh` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_docker_entrypoints.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_docs_deterministic.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_docs_freeze_drift.py` | `docs-governance` | `public` | `docs-freeze` |
| `scripts/docs/check_docs_make_only.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_docs_make_only_ops.py` | `docs-governance` | `public` | `ci-docs-make-only-ops` |
| `scripts/docs/check_docs_make_targets_exist.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_duplicate_topics.sh` | `docs-governance` | `public` | `docs-build`, `rename-lint` |
| `scripts/docs/check_example_configs.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_full_stack_page.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_generated_contract_docs.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_index_pages.sh` | `docs-governance` | `public` | `docs-build`, `docs-lint-names` |
| `scripts/docs/check_k8s_docs_contract.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_load_docs_contract.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_make_help_drift.py` | `docs-governance` | `public` | `ci-make-help-drift`, `docs-build` |
| `scripts/docs/check_make_targets_documented.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_make_targets_drift.py` | `docs-governance` | `public` | `docs-build`, `ops-make-targets-doc` |
| `scripts/docs/check_mkdocs_site_links.py` | `docs-governance` | `public` | - |
| `scripts/docs/check_nav_order.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_no_legacy_root_paths.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_no_orphan_docs.py` | `docs-governance` | `public` | `docs-build`, `docs-lint-names` |
| `scripts/docs/check_no_placeholders.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_observability_acceptance_checklist.py` | `docs-governance` | `public` | - |
| `scripts/docs/check_observability_docs_checklist.py` | `docs-governance` | `public` | `docs-build`, `docs-lint-names` |
| `scripts/docs/check_observability_surface_drift.py` | `docs-governance` | `public` | `ops-observability-validate` |
| `scripts/docs/check_openapi_examples.py` | `docs-governance` | `public` | `docs-build`, `ops-api-smoke`, `ops-openapi-validate` |
| `scripts/docs/check_ops_doc_duplication.py` | `docs-governance` | `public` | `ci-ops-doc-duplication` |
| `scripts/docs/check_ops_docs_make_targets.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_ops_observability_links.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_ops_readme_canonical_links.py` | `docs-governance` | `public` | `ci-ops-readme-canonical-links` |
| `scripts/docs/check_ops_readmes_make_only.py` | `docs-governance` | `public` | `ci-ops-readme-make-only` |
| `scripts/docs/check_public_surface_docs.py` | `docs-governance` | `public` | - |
| `scripts/docs/check_reference_templates.sh` | `docs-governance` | `public` | - |
| `scripts/docs/check_runbook_map_registration.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/docs/check_runbooks_contract.py` | `docs-governance` | `public` | `docs-build`, `ops-drill-runner` |
| `scripts/docs/check_script_headers.py` | `docs-governance` | `public` | `docs-build`, `scripts-audit`, `scripts-lint` |
| `scripts/docs/check_script_locations.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/docs/check_suite_id_docs.py` | `docs-governance` | `public` | - |
| `scripts/docs/check_terminology_units_ssot.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/check_title_case.sh` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/extract_code_blocks.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/generate_architecture_map.py` | `docs-governance` | `public` | `architecture-check`, `docs-build` |
| `scripts/docs/generate_concept_graph.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/generate_crates_map.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/generate_k8s_install_matrix.py` | `docs-governance` | `public` | - |
| `scripts/docs/generate_k8s_values_doc.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/generate_make_targets_catalog.py` | `docs-governance` | `public` | - |
| `scripts/docs/generate_make_targets_inventory.py` | `docs-governance` | `public` | `docs-build`, `ops-make-targets-doc` |
| `scripts/docs/generate_makefiles_surface.py` | `docs-governance` | `public` | - |
| `scripts/docs/generate_observability_surface.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/generate_openapi_docs.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/generate_ops_contracts_doc.py` | `docs-governance` | `public` | `docs-build`, `ops-contracts-check`, `ops-gen` |
| `scripts/docs/generate_ops_schema_docs.py` | `docs-governance` | `public` | `docs-build`, `ops-contracts-check`, `ops-gen` |
| `scripts/docs/generate_ops_surface.py` | `docs-governance` | `public` | `docs-build`, `ops-contracts-check`, `ops-gen` |
| `scripts/docs/generate_repo_surface.py` | `docs-governance` | `public` | - |
| `scripts/docs/generate_scripts_graph.py` | `docs-governance` | `public` | `scripts-graph` |
| `scripts/docs/generate_sli_doc.py` | `docs-governance` | `public` | `ci-sli-docs-drift` |
| `scripts/docs/generate_slos_doc.py` | `docs-governance` | `public` | `ci-slo-docs-drift` |
| `scripts/docs/legacy-terms-allowlist.txt` | `docs-governance` | `public` | - |
| `scripts/docs/lint_depth.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/lint_doc_contracts.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/lint_doc_status.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/lint_glossary_links.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/naming_inventory.py` | `docs-governance` | `public` | `docs-lint-names` |
| `scripts/docs/render_diagrams.sh` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/rewrite_legacy_terms.py` | `docs-governance` | `public` | - |
| `scripts/docs/run_blessed_snippets.py` | `docs-governance` | `public` | `docs-build` |
| `scripts/docs/spellcheck_docs.py` | `docs-governance` | `public` | - |
| `scripts/fixtures/derive-release-111.sh` | `dataset-ops` | `internal` | - |
| `scripts/fixtures/fetch-medium.sh` | `dataset-ops` | `public` | `fetch-fixtures`, `ops-publish-medium` |
| `scripts/fixtures/fetch-real-datasets.sh` | `dataset-ops` | `public` | `fetch-real-datasets` |
| `scripts/fixtures/run-medium-ingest.sh` | `dataset-ops` | `public` | `ingest-sharded-medium`, `run-medium-ingest` |
| `scripts/fixtures/run-medium-serve.sh` | `dataset-ops` | `public` | `run-medium-serve` |
| `scripts/gen/generate_scripts_readme.py` | `platform` | `public` | - |
| `scripts/gen/generate_scripts_surface.py` | `platform` | `public` | - |
| `scripts/generate_scripts_readme.py` | `platform` | `public` | `scripts-index` |
| `scripts/internal/__init__.py` | `platform` | `internal` | - |
| `scripts/internal/effects-lint.sh` | `platform` | `internal` | - |
| `scripts/internal/env_dump.sh` | `platform` | `internal` | - |
| `scripts/internal/exec.sh` | `platform` | `internal` | - |
| `scripts/internal/migrate_paths.sh` | `platform` | `internal` | - |
| `scripts/internal/naming-intent-lint.sh` | `platform` | `internal` | - |
| `scripts/internal/openapi-generate.sh` | `platform` | `internal` | - |
| `scripts/internal/paths.py` | `platform` | `internal` | - |
| `scripts/internal/repo_root.sh` | `platform` | `internal` | - |
| `scripts/layout/allowed_root.json` | `repo-surface` | `public` | - |
| `scripts/layout/build_artifacts_index.py` | `repo-surface` | `public` | `artifacts-index` |
| `scripts/layout/build_run_artifact_index.py` | `repo-surface` | `public` | `ops-artifacts-index-run` |
| `scripts/layout/check_artifacts_allowlist.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_artifacts_policy.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_cargo_dev_metadata.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_cargo_invocations_scoped.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_chart_canonical_path.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ci_entrypoints.py` | `repo-surface` | `public` | `ci-workflow-contract` |
| `scripts/layout/check_dataset_manifest_lock.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_duplicate_script_intent.py` | `repo-surface` | `public` | `scripts-lint` |
| `scripts/layout/check_e2e_scenarios.py` | `repo-surface` | `public` | `ops-e2e-validate` |
| `scripts/layout/check_forbidden_root_files.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_forbidden_root_names.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_generated_dirs_policy.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_generated_policy.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/layout/check_help_excludes_internal.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_internal_targets_not_in_docs.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_kind_cluster_contract_drift.sh` | `repo-surface` | `public` | `ops-kind-cluster-drift-check` |
| `scripts/layout/check_make_lane_reports.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_make_public_scripts.py` | `repo-surface` | `public` | `no-direct-scripts`, `scripts-audit`, `scripts-lint`, `scripts-test` |
| `scripts/layout/check_make_safety.py` | `repo-surface` | `public` | `ci-make-safety`, `path-contract-check` |
| `scripts/layout/check_make_target_ownership.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_make_targets_catalog_drift.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_makefile_headers.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_makefile_target_boundaries.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_makefiles_contract.py` | `repo-surface` | `public` | `makefiles-contract`, `release` |
| `scripts/layout/check_makefiles_index_drift.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_no_direct_script_runs.sh` | `repo-surface` | `public` | `_lint-configs`, `no-direct-scripts` |
| `scripts/layout/check_no_empty_dirs.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/layout/check_no_forbidden_paths.sh` | `repo-surface` | `public` | `ci-forbid-raw-paths`, `layout-check`, `path-contract-check` |
| `scripts/layout/check_no_hidden_defaults.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/layout/check_no_mixed_script_name_variants.py` | `repo-surface` | `public` | `scripts-lint` |
| `scripts/layout/check_obs_pack_ssot.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/layout/check_obs_script_name_collisions.py` | `repo-surface` | `public` | `ops-observability-validate` |
| `scripts/layout/check_ops_artifacts_writes.py` | `repo-surface` | `public` | `layout-check`, `ops-layout-lint` |
| `scripts/layout/check_ops_canonical_entrypoints.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/layout/check_ops_canonical_shims.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_concept_ownership.py` | `repo-surface` | `public` | `layout-check`, `ops-layout-lint` |
| `scripts/layout/check_ops_cross_area_script_refs.py` | `repo-surface` | `public` | `ops-lint-all` |
| `scripts/layout/check_ops_index_surface.py` | `repo-surface` | `public` | `ci-ops-index-surface`, `layout-check`, `ops-check-legacy` |
| `scripts/layout/check_ops_layout_contract.py` | `repo-surface` | `public` | `layout-check`, `ops-layout-lint` |
| `scripts/layout/check_ops_lib_canonical.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_run_entrypoints.py` | `repo-surface` | `public` | `ci-ops-run-entrypoints`, `ops-lint` |
| `scripts/layout/check_ops_script_names.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/layout/check_ops_script_targets.sh` | `repo-surface` | `public` | `ops-script-coverage` |
| `scripts/layout/check_ops_single_owner_contracts.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_single_validators.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_stack_order.sh` | `repo-surface` | `public` | `ops-stack-order-check`, `ops-stack-validate` |
| `scripts/layout/check_ops_surface_drift.py` | `repo-surface` | `public` | `ops-contracts-check` |
| `scripts/layout/check_ops_workspace.sh` | `repo-surface` | `public` | `layout-check`, `ops-layout-lint` |
| `scripts/layout/check_public_entrypoint_cap.py` | `repo-surface` | `public` | `scripts-lint` |
| `scripts/layout/check_public_surface.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_public_target_aliases.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_public_target_budget.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_public_target_descriptions.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_public_targets_documented.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_realdata_scenarios.py` | `repo-surface` | `public` | `ops-e2e-validate` |
| `scripts/layout/check_repo_hygiene.sh` | `repo-surface` | `public` | `_check`, `_coverage`, `_fmt`, `_lint-configs`, `_test`, `_test-all`, `_test-contracts`, `layout-check` |
| `scripts/layout/check_root_determinism.sh` | `repo-surface` | `public` | `root-determinism` |
| `scripts/layout/check_root_diff_alarm.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_root_local_lane_isolation.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_root_makefile_hygiene.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_root_no_cargo_dev_deps.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_root_shape.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_script_naming_convention.py` | `repo-surface` | `public` | `scripts-lint` |
| `scripts/layout/check_script_relative_calls.py` | `repo-surface` | `public` | `scripts-audit`, `scripts-lint` |
| `scripts/layout/check_scripts_buckets.py` | `repo-surface` | `public` | `scripts-audit`, `scripts-lint` |
| `scripts/layout/check_scripts_readme_drift.sh` | `repo-surface` | `public` | `_lint-configs` |
| `scripts/layout/check_scripts_submodules.py` | `repo-surface` | `public` | `ops-lint-all` |
| `scripts/layout/check_slo_contracts.py` | `repo-surface` | `public` | `ci-sli-contract`, `ci-slo-config-validate`, `ci-slo-metrics-contract` |
| `scripts/layout/check_slo_no_loosen_without_approval.py` | `repo-surface` | `public` | `ci-slo-no-loosen` |
| `scripts/layout/check_stack_manifest_consolidation.sh` | `repo-surface` | `public` | `ops-stack-validate` |
| `scripts/layout/check_symlink_index.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_symlink_policy.py` | `repo-surface` | `public` | `configs-check`, `layout-check` |
| `scripts/layout/check_tool_versions.py` | `repo-surface` | `public` | `ops-helm-version-check`, `ops-jq-version-check`, `ops-k6-version-check`, `ops-kind-version-check`, `ops-kubectl-version-check`, `ops-lint`, `ops-yq-version-check` |
| `scripts/layout/check_workflows_make_only.py` | `repo-surface` | `public` | `ci-workflows-make-only`, `layout-check` |
| `scripts/layout/clean_artifacts.py` | `repo-surface` | `public` | `artifacts-clean` |
| `scripts/layout/clean_make_artifacts.py` | `repo-surface` | `public` | `clean-all`, `clean-safe` |
| `scripts/layout/clean_ops_generated.py` | `repo-surface` | `public` | `ops-gen-clean` |
| `scripts/layout/dataset_id_lint.py` | `repo-surface` | `public` | `dataset-id-lint` |
| `scripts/layout/explain_public_target.py` | `repo-surface` | `public` | `explain` |
| `scripts/layout/generate_ops_stack_versions.py` | `repo-surface` | `public` | `ops-stack-versions-sync` |
| `scripts/layout/generate_ops_surface_meta.py` | `repo-surface` | `public` | `ops-gen` |
| `scripts/layout/graph_public_target.py` | `repo-surface` | `public` | `graph` |
| `scripts/layout/list_internal_targets.py` | `repo-surface` | `public` | `internal-list` |
| `scripts/layout/make_doctor.py` | `repo-surface` | `public` | `doctor` |
| `scripts/layout/make_prereqs.py` | `repo-surface` | `public` | `prereqs` |
| `scripts/layout/make_report.py` | `repo-surface` | `public` | - |
| `scripts/layout/make_target_graph.py` | `repo-surface` | `public` | - |
| `scripts/layout/migrate.sh` | `repo-surface` | `public` | `layout-migrate` |
| `scripts/layout/public_make_targets.py` | `repo-surface` | `public` | - |
| `scripts/layout/public_surface.py` | `repo-surface` | `public` | - |
| `scripts/layout/render_public_help.py` | `repo-surface` | `public` | `gates`, `help`, `list` |
| `scripts/layout/replace_paths.sh` | `repo-surface` | `public` | `layout-migrate` |
| `scripts/layout/root_whitelist.json` | `repo-surface` | `public` | - |
| `scripts/layout/run_gate.py` | `repo-surface` | `public` | - |
| `scripts/layout/validate_ops_contracts.py` | `repo-surface` | `public` | `ops-contracts-check`, `ops-gen`, `ops-k8s-contracts` |
| `scripts/layout/validate_ops_env.py` | `repo-surface` | `public` | `ops-env-print`, `ops-env-validate` |
| `scripts/layout/write_make_area_report.py` | `repo-surface` | `public` | - |
| `scripts/lib/errors.sh` | `platform` | `internal` | - |
| `scripts/ops/check_k8s_flakes.py` | `platform` | `public` | `ops-k8s-tests` |
| `scripts/ops/check_k8s_test_contract.py` | `platform` | `public` | `ops-k8s-tests` |
| `scripts/policy/find_relaxations.sh` | `platform` | `internal` | - |
| `scripts/public/check-allow-env-schema.py` | `platform` | `public` | `policy-allow-env-lint` |
| `scripts/public/check-cli-commands.sh` | `platform` | `public` | `cli-command-surface` |
| `scripts/public/check-markdown-links.sh` | `platform` | `public` | `_lint-docs`, `docs-build` |
| `scripts/public/config-drift-check.py` | `platform` | `public` | `config-drift`, `configs-check` |
| `scripts/public/config-print.py` | `platform` | `public` | `config-print` |
| `scripts/public/config-validate.py` | `platform` | `public` | `configs-check` |
| `scripts/public/contracts/check_breaking_contract_change.py` | `platform` | `public` | `api-contract-check`, `ci-openapi-drift` |
| `scripts/public/contracts/check_endpoints_contract.py` | `platform` | `public` | `api-contract-check` |
| `scripts/public/contracts/check_error_codes_contract.py` | `platform` | `public` | `api-contract-check` |
| `scripts/public/contracts/check_sqlite_indexes_contract.py` | `platform` | `public` | `critical-query-check` |
| `scripts/public/contracts/check_v1_surface.py` | `platform` | `public` | `api-contract-check` |
| `scripts/public/contracts/gen_openapi.py` | `platform` | `public` | `api-contract-check` |
| `scripts/public/generate-config-key-registry.py` | `platform` | `public` | - |
| `scripts/public/no-network-unit-tests.sh` | `platform` | `public` | - |
| `scripts/public/observability/check_alerts_contract.py` | `platform` | `public` | `ops-alerts-validate`, `ops-metrics-check` |
| `scripts/public/observability/check_dashboard_contract.py` | `platform` | `public` | `ops-dashboards-validate`, `ops-metrics-check` |
| `scripts/public/observability/check_metrics_contract.py` | `platform` | `public` | `ops-metrics-check`, `ops-observability-validate` |
| `scripts/public/observability/check_runtime_metrics.py` | `platform` | `public` | `ops-metrics-check` |
| `scripts/public/observability/check_tracing_contract.py` | `platform` | `public` | `ops-observability-validate`, `ops-traces-check` |
| `scripts/public/observability/lint_runbooks.py` | `platform` | `public` | `ops-metrics-check` |
| `scripts/public/openapi-diff-check.sh` | `platform` | `public` | `api-contract-check`, `openapi-drift`, `ops-openapi-validate` |
| `scripts/public/ops-policy-audit.py` | `platform` | `public` | `ops-policy-audit` |
| `scripts/public/perf/check_baseline_update_policy.sh` | `platform` | `public` | - |
| `scripts/public/perf/check_percent_regression.py` | `platform` | `public` | `stack-full` |
| `scripts/public/perf/check_pinned_queries_lock.py` | `platform` | `public` | - |
| `scripts/public/perf/check_prereqs.sh` | `platform` | `public` | - |
| `scripts/public/perf/check_regression.py` | `platform` | `public` | - |
| `scripts/public/perf/check_runbook_suite_names.py` | `platform` | `public` | - |
| `scripts/public/perf/check_spike_assertions.py` | `platform` | `public` | `ops-load-spike-proof` |
| `scripts/public/perf/cold_start_benchmark.sh` | `platform` | `public` | - |
| `scripts/public/perf/cold_start_prefetch_5pods.sh` | `platform` | `public` | - |
| `scripts/public/perf/compare_redis.sh` | `platform` | `public` | - |
| `scripts/public/perf/generate_report.py` | `platform` | `public` | - |
| `scripts/public/perf/load_under_rollback.sh` | `platform` | `public` | - |
| `scripts/public/perf/load_under_rollout.sh` | `platform` | `public` | - |
| `scripts/public/perf/prepare_perf_store.sh` | `platform` | `public` | - |
| `scripts/public/perf/run_critical_queries.py` | `platform` | `public` | `critical-query-check` |
| `scripts/public/perf/run_e2e_perf.sh` | `platform` | `public` | - |
| `scripts/public/perf/run_nightly_perf.sh` | `platform` | `public` | - |
| `scripts/public/perf/run_suite.sh` | `platform` | `public` | - |
| `scripts/public/perf/run_suites_from_manifest.py` | `platform` | `public` | - |
| `scripts/public/perf/score_k6.py` | `platform` | `public` | - |
| `scripts/public/perf/update_baseline.sh` | `platform` | `public` | - |
| `scripts/public/perf/validate_results.py` | `platform` | `public` | - |
| `scripts/public/perf/validate_suite_manifest.py` | `platform` | `public` | - |
| `scripts/public/policy-audit.py` | `platform` | `public` | `policy-audit` |
| `scripts/public/policy-drift-diff.sh` | `platform` | `public` | `policy-drift-diff` |
| `scripts/public/policy-enforcement-status.py` | `platform` | `public` | `policy-enforcement-status` |
| `scripts/public/policy-lint.sh` | `platform` | `public` | `_lint-configs`, `policy-lint` |
| `scripts/public/policy-schema-drift.py` | `platform` | `public` | `policy-schema-drift` |
| `scripts/public/qc-fixtures-gate.sh` | `platform` | `public` | `ci-qc-fixtures` |
| `scripts/public/query-plan-gate.sh` | `platform` | `public` | `query-plan-gate` |
| `scripts/public/report-bundle.sh` | `platform` | `public` | `ops-report` |
| `scripts/public/require-crate-docs.sh` | `platform` | `public` | `crate-structure` |
| `scripts/public/stack/build_stack_report.py` | `platform` | `public` | `stack-full` |
| `scripts/public/stack/validate_stack_report.py` | `platform` | `public` | `stack-full` |
| `scripts/python/__init__.py` | `platform` | `internal` | - |
| `scripts/python/bijux_scripts/__init__.py` | `platform` | `internal` | - |
| `scripts/python/bijux_scripts/json_helpers.py` | `platform` | `internal` | - |
| `scripts/python/bijux_scripts/paths.py` | `platform` | `internal` | - |
| `scripts/python/bijux_scripts/reporting.py` | `platform` | `internal` | - |
| `scripts/python/bijux_scripts/runner.py` | `platform` | `internal` | - |
| `scripts/release/update-compat-matrix.sh` | `release-engineering` | `public` | `release-update-compat-matrix` |
| `scripts/release/validate-compat-matrix.sh` | `release-engineering` | `public` | `compat-matrix-validate` |
| `scripts/requirements.lock.txt` | `platform` | `internal` | - |
| `scripts/run_drill.sh` | `platform` | `internal` | - |
| `scripts/tests/test_paths.py` | `platform` | `internal` | - |
| `scripts/tools/__init__.py` | `platform` | `internal` | - |
| `scripts/tools/json_helpers.py` | `platform` | `internal` | - |
| `scripts/tools/path_utils.py` | `platform` | `internal` | - |
| `scripts/tools/reporting.py` | `platform` | `internal` | - |
