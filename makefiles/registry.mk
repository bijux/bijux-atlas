# Scope: legacy registry metadata for docs and migration support.
# Public targets: none
SHELL := /bin/sh

# Central make target registry (SSOT for help/docs rendering).
REGISTRY_CATEGORIES := DEV DOCS CONTRACTS OPS_QUICK OPS RELEASE CI TOOLING META

REGISTRY_DEV_DESC := dev
REGISTRY_DEV_TARGETS := fmt fmt-all lint lint-all test test-all audit audit-all coverage coverage-all check check-all all all-all ci local local-full

REGISTRY_DOCS_DESC := docs
REGISTRY_DOCS_TARGETS := docs docs-serve docs-freeze docs-hardening

REGISTRY_CONTRACTS_DESC := contracts
REGISTRY_CONTRACTS_TARGETS := contracts ssot-check budgets/check policy-lint policy-schema-drift policy-audit policy-enforcement-status policy-allow-env-lint policies/boundaries-check policy-drift-diff ops-policy-audit rename-lint docs-lint-names config-validate config-print config-drift openapi-drift api-contract-check telemetry-contracts telemetry-verify ops-values-validate ops-chart-render-diff ops-openapi-validate ops-dashboards-validate ops-alerts-validate perf/baseline-update perf/regression-check perf/triage perf/compare

REGISTRY_OPS_QUICK_DESC := ops quick
REGISTRY_OPS_QUICK_TARGETS := ops-check ops-test

REGISTRY_OPS_DESC := ops
REGISTRY_OPS_TARGETS := ops-help ops-surface ops-env-validate ops-env-print pins/check pins/update ops-check ops-gen ops-gen-check ops-fmt ops-lint ops-test ops-up ops-down ops-clean

REGISTRY_RELEASE_DESC := release/surface
REGISTRY_RELEASE_TARGETS := fmt lint check test test-all coverage audit openapi-drift ci ssot-check crate-structure crate-docs-contract cli-command-surface query-plan-gate critical-query-check docker-build docker-smoke docker-scan docker-push chart-package chart-verify chart-validate release-dry-run release bump clean deep-clean debug

REGISTRY_CI_DESC := ci-mapping
REGISTRY_CI_TARGETS := ci-fast ci-contracts ci-docs ci-ops ci-workflows-make-only ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify ci-security-advisory-render governance-check

REGISTRY_TOOLING_DESC := tooling
REGISTRY_TOOLING_TARGETS := bootstrap bootstrap-tools doctor prereqs pins/check pins/update dataset-id-lint scripts-index scripts-graph scripts-all scripts-audit scripts-lint scripts-check scripts-format scripts-test scripts-clean artifacts-index artifacts-clean isolate-clean no-direct-scripts help

REGISTRY_META_DESC := meta
REGISTRY_META_TARGETS := ci local local-full contracts hygiene architecture-check path-contract-check
