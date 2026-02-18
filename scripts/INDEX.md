# Scripts Index

Generated file. Do not edit manually.

| Script | Owner | Stability | Called By |
|---|---|---|---|
| `scripts/ENTRYPOINTS.md` | `platform` | `internal` | - |
| `scripts/_internal/check-root-layout.sh` | `internal` | `internal` | - |
| `scripts/_internal/run_suite_wrapper_legacy.sh` | `internal` | `internal` | - |
| `scripts/bin/isolate` | `platform` | `public` | `audit`, `check`, `coverage`, `fmt`, `lint`, `test`, `test-all` |
| `scripts/bin/require-isolate` | `platform` | `public` | `_audit`, `_check`, `_coverage`, `_fmt`, `_lint-clippy`, `_lint-configs`, `_lint-docs`, `_lint-rustfmt`, `_test`, `_test-all`, `audit`, `check`, `cli-command-surface`, `coverage`, `crate-docs-contract`, `crate-structure`, `fmt`, `lint`, `test`, `test-all` |
| `scripts/bootstrap/install_tools.sh` | `developer-experience` | `public` | `bootstrap-tools` |
| `scripts/contracts/check_all.sh` | `contracts` | `public` | `ssot-check` |
| `scripts/contracts/check_breaking_contract_change.py` | `contracts` | `internal` | `api-contract-check` |
| `scripts/contracts/check_chart_values_contract.py` | `contracts` | `public` | `chart-validate`, `ops-values-validate` |
| `scripts/contracts/check_cli_ssot.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_config_keys_contract.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_contract_drift.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_endpoints_contract.py` | `contracts` | `internal` | `api-contract-check` |
| `scripts/contracts/check_error_codes_contract.py` | `contracts` | `internal` | `api-contract-check` |
| `scripts/contracts/check_sqlite_indexes_contract.py` | `contracts` | `internal` | `critical-query-check` |
| `scripts/contracts/check_v1_surface.py` | `contracts` | `internal` | `api-contract-check` |
| `scripts/contracts/format_contracts.py` | `contracts` | `internal` | - |
| `scripts/contracts/gen_openapi.py` | `contracts` | `internal` | `api-contract-check` |
| `scripts/contracts/generate_chart_values_schema.py` | `contracts` | `public` | `chart-validate`, `ops-values-validate` |
| `scripts/contracts/generate_contract_artifacts.py` | `contracts` | `public` | `docs-freeze` |
| `scripts/demo/demo.sh` | `platform` | `private` | - |
| `scripts/docs/check_adr_headers.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_broken_examples.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_concept_ids.sh` | `docs-governance` | `public` | - |
| `scripts/docs/check_concept_registry.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_crate_docs_contract.sh` | `docs-governance` | `public` | `crate-docs-contract`, `crate-structure`, `docs` |
| `scripts/docs/check_critical_make_targets_referenced.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_doc_filename_style.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_doc_naming.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_docker_entrypoints.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_docs_deterministic.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_docs_make_only.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_docs_make_targets_exist.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_duplicate_topics.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_example_configs.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_full_stack_page.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_generated_contract_docs.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_index_pages.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_k8s_docs_contract.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_load_docs_contract.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_make_help_drift.py` | `docs-governance` | `public` | `ci-make-help-drift`, `docs` |
| `scripts/docs/check_make_targets_documented.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_make_targets_drift.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_mkdocs_site_links.py` | `docs-governance` | `public` | - |
| `scripts/docs/check_nav_order.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_no_legacy_root_paths.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_no_orphan_docs.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_no_placeholders.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_openapi_examples.py` | `docs-governance` | `public` | `docs`, `ops-openapi-validate`, `ops-smoke` |
| `scripts/docs/check_ops_docs_make_targets.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_reference_templates.sh` | `docs-governance` | `public` | - |
| `scripts/docs/check_runbooks_contract.py` | `docs-governance` | `public` | `docs`, `ops-drill-runner` |
| `scripts/docs/check_script_headers.py` | `docs-governance` | `public` | `docs`, `scripts-audit` |
| `scripts/docs/check_terminology_units_ssot.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_title_case.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/extract_code_blocks.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_architecture_map.py` | `docs-governance` | `public` | `architecture-check`, `docs` |
| `scripts/docs/generate_concept_graph.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_crates_map.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_k8s_install_matrix.py` | `docs-governance` | `public` | - |
| `scripts/docs/generate_k8s_values_doc.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_make_targets_inventory.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_openapi_docs.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_scripts_graph.py` | `docs-governance` | `public` | `scripts-graph` |
| `scripts/docs/lint_depth.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/lint_doc_contracts.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/lint_doc_status.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/lint_glossary_links.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/render_diagrams.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/run_blessed_snippets.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/spellcheck_docs.py` | `docs-governance` | `public` | - |
| `scripts/fixtures/derive-release-111.sh` | `dataset-ops` | `internal` | - |
| `scripts/fixtures/fetch-medium.sh` | `dataset-ops` | `public` | `fetch-fixtures`, `ops-publish-medium` |
| `scripts/fixtures/fetch-real-datasets.sh` | `dataset-ops` | `public` | `fetch-real-datasets` |
| `scripts/fixtures/run-medium-ingest.sh` | `dataset-ops` | `public` | `ingest-sharded-medium`, `run-medium-ingest` |
| `scripts/fixtures/run-medium-serve.sh` | `dataset-ops` | `public` | `run-medium-serve` |
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
| `scripts/layout/build_artifacts_index.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_artifacts_allowlist.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_artifacts_policy.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_chart_canonical_path.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_forbidden_root_files.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_forbidden_root_names.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_kind_cluster_contract_drift.sh` | `repo-surface` | `public` | `ops-kind-cluster-drift-check` |
| `scripts/layout/check_make_public_scripts.py` | `repo-surface` | `public` | `no-direct-scripts`, `scripts-audit`, `scripts-test` |
| `scripts/layout/check_make_safety.py` | `repo-surface` | `public` | `ci-make-safety`, `path-contract-check` |
| `scripts/layout/check_no_direct_script_runs.sh` | `repo-surface` | `public` | `_lint-configs`, `no-direct-scripts` |
| `scripts/layout/check_no_forbidden_paths.sh` | `repo-surface` | `public` | `ci-forbid-raw-paths`, `layout-check`, `path-contract-check` |
| `scripts/layout/check_ops_canonical_shims.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_lib_canonical.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_script_targets.sh` | `repo-surface` | `public` | `ops-script-coverage` |
| `scripts/layout/check_ops_stack_order.sh` | `repo-surface` | `public` | `ops-stack-order-check`, `ops-stack-validate` |
| `scripts/layout/check_ops_workspace.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_repo_hygiene.sh` | `repo-surface` | `public` | `_check`, `_coverage`, `_fmt`, `_lint-configs`, `_test`, `_test-all`, `layout-check` |
| `scripts/layout/check_root_shape.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_script_relative_calls.py` | `repo-surface` | `public` | `scripts-audit` |
| `scripts/layout/check_scripts_buckets.py` | `repo-surface` | `public` | `scripts-audit` |
| `scripts/layout/check_scripts_readme_drift.sh` | `repo-surface` | `public` | `_lint-configs` |
| `scripts/layout/check_stack_manifest_consolidation.sh` | `repo-surface` | `public` | `ops-stack-validate` |
| `scripts/layout/check_symlink_index.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_symlink_policy.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_tool_versions.py` | `repo-surface` | `public` | `ops-helm-version-check`, `ops-jq-version-check`, `ops-k6-version-check`, `ops-kind-version-check`, `ops-kubectl-version-check`, `ops-yq-version-check` |
| `scripts/layout/check_workflows_make_only.py` | `repo-surface` | `public` | `ci-workflows-make-only`, `layout-check` |
| `scripts/layout/clean_artifacts.py` | `repo-surface` | `public` | `artifacts-clean` |
| `scripts/layout/dataset_id_lint.py` | `repo-surface` | `public` | `dataset-id-lint` |
| `scripts/layout/migrate.sh` | `repo-surface` | `public` | `layout-migrate` |
| `scripts/layout/replace_paths.sh` | `repo-surface` | `public` | `layout-migrate` |
| `scripts/layout/root_whitelist.json` | `repo-surface` | `public` | - |
| `scripts/layout/validate_ops_env.py` | `repo-surface` | `public` | `ops-env-print`, `ops-env-validate` |
| `scripts/ops/check_k8s_flakes.py` | `platform` | `public` | `ops-k8s-tests` |
| `scripts/ops/check_k8s_test_contract.py` | `platform` | `public` | `ops-k8s-tests` |
| `scripts/public/check-cli-commands.sh` | `platform` | `public` | `cli-command-surface` |
| `scripts/public/check-markdown-links.sh` | `platform` | `public` | `_lint-docs`, `docs` |
| `scripts/public/config-drift-check.py` | `platform` | `public` | `config-drift`, `config-validate` |
| `scripts/public/config-print.py` | `platform` | `public` | `config-print` |
| `scripts/public/config-validate.py` | `platform` | `public` | `config-validate` |
| `scripts/public/generate-config-key-registry.py` | `platform` | `public` | `config-validate` |
| `scripts/public/no-network-unit-tests.sh` | `platform` | `public` | - |
| `scripts/public/observability/check_alerts_contract.py` | `platform` | `public` | `ops-alerts-validate`, `ops-metrics-check` |
| `scripts/public/observability/check_dashboard_contract.py` | `platform` | `public` | `ops-dashboards-validate`, `ops-metrics-check` |
| `scripts/public/observability/check_metrics_contract.py` | `platform` | `public` | `ops-metrics-check`, `ops-observability-validate` |
| `scripts/public/observability/check_runtime_metrics.py` | `platform` | `public` | `ops-metrics-check` |
| `scripts/public/observability/check_tracing_contract.py` | `platform` | `public` | `ops-observability-validate`, `ops-traces-check` |
| `scripts/public/observability/lint_runbooks.py` | `platform` | `public` | `ops-metrics-check` |
| `scripts/public/openapi-diff-check.sh` | `platform` | `public` | `api-contract-check`, `openapi-drift`, `ops-openapi-validate` |
| `scripts/public/perf/__pycache__/check_spike_assertions.cpython-310.pyc` | `platform` | `public` | - |
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
| `scripts/public/policy-lint.sh` | `platform` | `public` | `_lint-configs`, `policy-lint` |
| `scripts/public/policy-schema-drift.py` | `platform` | `public` | `policy-schema-drift` |
| `scripts/public/qc-fixtures-gate.sh` | `platform` | `public` | `ci-qc-fixtures` |
| `scripts/public/query-plan-gate.sh` | `platform` | `public` | `query-plan-gate` |
| `scripts/public/report_bundle.sh` | `platform` | `public` | `ops-report` |
| `scripts/public/require-crate-docs.sh` | `platform` | `public` | `crate-structure` |
| `scripts/public/stack/build_stack_report.py` | `platform` | `public` | `stack-full` |
| `scripts/public/stack/validate_stack_report.py` | `platform` | `public` | `stack-full` |
| `scripts/release/update-compat-matrix.sh` | `release-engineering` | `public` | `release-update-compat-matrix` |
| `scripts/release/validate-compat-matrix.sh` | `release-engineering` | `public` | `compat-matrix-validate` |
| `scripts/tests/test_paths.py` | `platform` | `internal` | - |
| `scripts/tools/__init__.py` | `platform` | `internal` | - |
| `scripts/tools/json_helpers.py` | `platform` | `internal` | - |
| `scripts/tools/path_utils.py` | `platform` | `internal` | - |
| `scripts/tools/reporting.py` | `platform` | `internal` | - |
