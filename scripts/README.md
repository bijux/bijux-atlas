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
| `scripts/contracts/check_breaking_contract_change.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_chart_values_contract.py` | `contracts` | `public` | `ops-values-validate` |
| `scripts/contracts/check_cli_ssot.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_config_keys_contract.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_contract_drift.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_endpoints_contract.py` | `contracts` | `internal` | - |
| `scripts/contracts/check_error_codes_contract.py` | `contracts` | `internal` | - |
| `scripts/contracts/format_contracts.py` | `contracts` | `internal` | - |
| `scripts/contracts/generate_chart_values_schema.py` | `contracts` | `public` | `ops-values-validate` |
| `scripts/contracts/generate_contract_artifacts.py` | `contracts` | `public` | `docs-freeze` |
| `scripts/demo/demo.sh` | `platform` | `private` | - |
| `scripts/docs/check_adr_headers.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_broken_examples.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_concept_ids.sh` | `docs-governance` | `public` | - |
| `scripts/docs/check_concept_registry.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_crate_docs_contract.sh` | `docs-governance` | `public` | `crate-docs-contract`, `crate-structure`, `docs` |
| `scripts/docs/check_doc_naming.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_docker_entrypoints.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_docs_make_only.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_duplicate_topics.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_example_configs.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_full_stack_page.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_generated_contract_docs.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_index_pages.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_k8s_docs_contract.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_load_docs_contract.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_make_targets_documented.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_make_targets_drift.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_mkdocs_site_links.py` | `docs-governance` | `public` | - |
| `scripts/docs/check_nav_order.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_no_orphan_docs.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_openapi_examples.py` | `docs-governance` | `public` | `docs`, `ops-openapi-validate`, `ops-smoke` |
| `scripts/docs/check_ops_docs_make_targets.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_reference_templates.sh` | `docs-governance` | `public` | - |
| `scripts/docs/check_runbooks_contract.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_script_headers.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_terminology_units_ssot.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/check_title_case.sh` | `docs-governance` | `public` | `docs` |
| `scripts/docs/extract_code_blocks.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_concept_graph.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_crates_map.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_k8s_install_matrix.py` | `docs-governance` | `public` | - |
| `scripts/docs/generate_k8s_values_doc.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_make_targets_inventory.py` | `docs-governance` | `public` | `docs` |
| `scripts/docs/generate_openapi_docs.py` | `docs-governance` | `public` | `docs` |
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
| `scripts/fixtures/run-medium-ingest.sh` | `dataset-ops` | `public` | `run-medium-ingest` |
| `scripts/fixtures/run-medium-serve.sh` | `dataset-ops` | `public` | `run-medium-serve` |
| `scripts/generate_scripts_readme.py` | `platform` | `public` | `scripts-index` |
| `scripts/internal/effects-lint.sh` | `platform` | `internal` | - |
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
| `scripts/layout/check_make_public_scripts.py` | `repo-surface` | `public` | `no-direct-scripts`, `scripts-test` |
| `scripts/layout/check_no_direct_script_runs.sh` | `repo-surface` | `public` | `_lint-configs`, `no-direct-scripts` |
| `scripts/layout/check_no_forbidden_paths.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_canonical_shims.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_lib_canonical.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_ops_script_targets.sh` | `repo-surface` | `public` | `ops-script-coverage` |
| `scripts/layout/check_ops_workspace.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_repo_hygiene.sh` | `repo-surface` | `public` | `_check`, `_coverage`, `_fmt`, `_lint-configs`, `_test`, `_test-all`, `layout-check` |
| `scripts/layout/check_root_shape.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_script_relative_calls.py` | `repo-surface` | `public` | - |
| `scripts/layout/check_scripts_readme_drift.sh` | `repo-surface` | `public` | `_lint-configs` |
| `scripts/layout/check_symlink_index.sh` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_symlink_policy.py` | `repo-surface` | `public` | `layout-check` |
| `scripts/layout/check_tool_versions.py` | `repo-surface` | `public` | `ops-helm-version-check`, `ops-k6-version-check`, `ops-kind-version-check`, `ops-kubectl-version-check` |
| `scripts/layout/check_workflows_make_only.py` | `repo-surface` | `public` | `ci-workflows-make-only`, `layout-check` |
| `scripts/layout/clean_artifacts.py` | `repo-surface` | `public` | `artifacts-clean` |
| `scripts/layout/migrate.sh` | `repo-surface` | `public` | `layout-migrate` |
| `scripts/layout/replace_paths.sh` | `repo-surface` | `public` | `layout-migrate` |
| `scripts/layout/root_whitelist.json` | `repo-surface` | `public` | - |
| `scripts/layout/validate_ops_env.py` | `repo-surface` | `public` | `ops-env-print`, `ops-env-validate` |
| `scripts/observability/check_alerts_contract.py` | `operations` | `public` | `ops-alerts-validate`, `ops-metrics-check` |
| `scripts/observability/check_dashboard_contract.py` | `operations` | `public` | `ops-dashboards-validate`, `ops-metrics-check` |
| `scripts/observability/check_metrics_contract.py` | `operations` | `public` | `ops-metrics-check`, `ops-observability-validate` |
| `scripts/observability/check_runtime_metrics.py` | `operations` | `public` | `ops-metrics-check` |
| `scripts/observability/check_tracing_contract.py` | `operations` | `public` | `ops-observability-validate`, `ops-traces-check` |
| `scripts/observability/lint_runbooks.py` | `operations` | `public` | `ops-metrics-check` |
| `scripts/ops/check_k8s_flakes.py` | `platform` | `public` | `ops-k8s-tests` |
| `scripts/ops/check_k8s_test_contract.py` | `platform` | `public` | `ops-k8s-tests` |
| `scripts/perf/check_baseline_update_policy.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/check_pinned_queries_lock.py` | `performance-compat` | `internal` | - |
| `scripts/perf/check_regression.py` | `performance-compat` | `internal` | - |
| `scripts/perf/check_runbook_suite_names.py` | `performance-compat` | `internal` | - |
| `scripts/perf/cold_start_benchmark.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/cold_start_prefetch_5pods.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/compare_redis.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/generate_report.py` | `performance-compat` | `internal` | - |
| `scripts/perf/load_under_rollback.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/load_under_rollout.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/prepare_perf_store.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/run_e2e_perf.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/run_nightly_perf.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/run_suite.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/run_suites_from_manifest.py` | `performance-compat` | `internal` | - |
| `scripts/perf/score_k6.py` | `performance-compat` | `internal` | - |
| `scripts/perf/update_baseline.sh` | `performance-compat` | `internal` | - |
| `scripts/perf/validate_results.py` | `performance-compat` | `internal` | - |
| `scripts/perf/validate_suite_manifest.py` | `performance-compat` | `internal` | - |
| `scripts/public/check-cli-commands.sh` | `platform` | `public` | `cli-command-surface` |
| `scripts/public/check-markdown-links.sh` | `platform` | `public` | `_lint-docs`, `docs` |
| `scripts/public/no-network-unit-tests.sh` | `platform` | `public` | - |
| `scripts/public/openapi-diff-check.sh` | `platform` | `public` | `openapi-drift`, `ops-openapi-validate` |
| `scripts/public/policy-lint.sh` | `platform` | `public` | `_lint-configs`, `policy-lint` |
| `scripts/public/policy-schema-drift.py` | `platform` | `public` | `policy-schema-drift` |
| `scripts/public/query-plan-gate.sh` | `platform` | `public` | `query-plan-gate` |
| `scripts/public/report_bundle.sh` | `platform` | `public` | `ops-report` |
| `scripts/public/require-crate-docs.sh` | `platform` | `public` | `crate-structure` |
| `scripts/release/update-compat-matrix.sh` | `release-engineering` | `public` | `release-update-compat-matrix` |
| `scripts/release/validate-compat-matrix.sh` | `release-engineering` | `public` | `compat-matrix-validate` |
| `scripts/tools/json_helpers.py` | `platform` | `internal` | - |
