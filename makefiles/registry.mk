SHELL := /bin/sh

# Central make target registry (SSOT for help/docs rendering).
REGISTRY_CATEGORIES := DEV DOCS CONTRACTS OPS_QUICK OPS RELEASE CI TOOLING META

REGISTRY_DEV_DESC := dev
REGISTRY_DEV_TARGETS := dev-fmt dev-lint dev-check dev-test dev-test-all dev-coverage dev-audit dev-ci dev-clean local local-full

REGISTRY_DOCS_DESC := docs
REGISTRY_DOCS_TARGETS := docs docs-serve docs-freeze docs-hardening

REGISTRY_CONTRACTS_DESC := contracts
REGISTRY_CONTRACTS_TARGETS := contracts ssot-check policy-lint policy-schema-drift config-validate config-print config-drift openapi-drift ops-values-validate ops-chart-render-diff ops-openapi-validate ops-dashboards-validate ops-alerts-validate

REGISTRY_OPS_QUICK_DESC := ops quick
REGISTRY_OPS_QUICK_TARGETS := ops-full ops-full-pr ops-full-nightly

REGISTRY_OPS_DESC := ops
REGISTRY_OPS_TARGETS := ops-stack-up ops-up ops-stack-down ops-down ops-stack-validate ops-stack-smoke ops-stack-health-report ops-stack-version ops-stack-uninstall ops-stack-slow-store ops-reset ops-clean ops-env-print ops-doctor ops-cluster-sanity ops-publish ops-publish-medium ops-deploy ops-offline ops-perf ops-multi-registry ops-ingress ops-warm ops-soak ops-smoke ops-metrics-check ops-traces-check ops-k8s-tests ops-k8s-template-tests ops-load-manifest-validate ops-load-prereqs ops-load-smoke ops-load-full ops-load-ci ops-load-nightly ops-drill-store-outage ops-drill-minio-outage ops-drill-prom-outage ops-drill-otel-outage ops-drill-toxiproxy-latency ops-drill-overload ops-drill-memory-growth ops-drill-rate-limit ops-drill-corruption ops-drill-pod-churn ops-drill-upgrade ops-drill-rollback ops-upgrade-drill ops-rollback-drill ops-realdata ops-report ops-script-coverage ops-shellcheck ops-kind-version-check ops-k6-version-check ops-helm-version-check ops-kubectl-version-check ops-kubeconform-version-check ops-tool-check ops-tools-check ops-values-validate ops-openapi-validate ops-dashboards-validate ops-alerts-validate ops-observability-validate ops-observability-smoke ops-open-grafana ops-obs-install ops-obs-uninstall ops-obs-validate ops-obs-up ops-obs-down ops-release-matrix ops-baseline-policy-check ops-perf-baseline-update ops-ci ops-ci-nightly ops-full ops-full-pr ops-full-nightly ops-perf-prepare-store ops-perf-e2e ops-perf-nightly ops-perf-cold-start ops-perf-cold-start-prefetch-5pods ops-perf-compare-redis ops-perf-report ops-perf-suite e2e-local e2e-k8s-install-gate e2e-k8s-suite e2e-perf e2e-realdata observability-check layout-check layout-migrate governance-check

REGISTRY_RELEASE_DESC := release/surface
REGISTRY_RELEASE_TARGETS := fmt lint check test test-all coverage audit openapi-drift ci ssot-check crate-structure crate-docs-contract cli-command-surface docker-build docker-smoke chart-package chart-verify release-dry-run release bump clean deep-clean debug

REGISTRY_CI_DESC := ci-mapping
REGISTRY_CI_TARGETS := ci-root-layout ci-script-entrypoints ci-fmt ci-clippy ci-test-nextest ci-deny ci-audit ci-license-check ci-policy-lint ci-policy-schema-drift ci-config-check ci-ssot-drift ci-crate-structure ci-crate-docs-contract ci-cli-command-surface ci-release-binaries ci-docs-build ci-latency-regression ci-store-conformance ci-openapi-drift ci-query-plan-gate ci-compatibility-matrix-validate ci-runtime-security-scan-image ci-coverage ci-workflows-make-only

REGISTRY_TOOLING_DESC := tooling
REGISTRY_TOOLING_TARGETS := bootstrap bootstrap-tools doctor scripts-index scripts-graph scripts-audit scripts-lint scripts-format scripts-test scripts-clean artifacts-index artifacts-clean isolate-clean no-direct-scripts help

REGISTRY_META_DESC := meta
REGISTRY_META_TARGETS := ci local local-full contracts hygiene path-contract-check
