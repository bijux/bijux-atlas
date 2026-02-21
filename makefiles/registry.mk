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
REGISTRY_OPS_QUICK_TARGETS := stack-full stack-full-chaos ops-full ops-full-pr ops-full-nightly

REGISTRY_OPS_DESC := ops
REGISTRY_OPS_TARGETS := stack-full stack-full-chaos ops-layout-lint ops-surface ops-help ops-stack-up ops-up ops-stack-down ops-down ops-stack-validate ops-stack-smoke ops-stack-health-report ops-stack-version ops-stack-versions-sync ops-stack-uninstall ops-stack-slow-store ops-kind-up ops-kind-down ops-kind-reset ops-kind-metrics-server-up ops-kind-registry-up ops-kind-image-resolution-test ops-kind-disk-pressure ops-kind-cpu-throttle ops-kind-network-latency ops-kind-context-guard ops-kind-namespace-guard ops-kind-cleanup-leftovers ops-kind-version-drift-test ops-kind-cluster-drift-check ops-kind-validate ops-minio-up ops-minio-down ops-minio-reset ops-minio-ready ops-minio-bucket-check ops-prom-up ops-prom-down ops-prom-ready ops-prom-scrape-atlas-check ops-grafana-up ops-grafana-down ops-grafana-ready ops-grafana-datasource-check ops-grafana-dashboards-check ops-otel-up ops-otel-down ops-otel-spans-check ops-otel-required-check ops-redis-up ops-redis-down ops-redis-optional-check ops-redis-used-check ops-redis-rate-limit-check ops-toxi-up ops-toxi-down ops-toxi-latency-inject ops-toxi-cut-store ops-stack-order-check ops-stack-security-check ops-reset ops-clean ops-env-print ops-prereqs ops-doctor ops-contracts-check ops-contract-check ops-e2e-validate ops-k8s-contracts ops-cluster-sanity ops-datasets-fetch ops-publish ops-publish-medium ops-release-update ops-release-rollback ops-catalog-validate ops-cache-status ops-cache-pin-set ops-dataset-qc ops-dataset-qc-diff ops-drill-corruption-dataset ops-dataset-promotion-sim ops-dataset-multi-release-test ops-dataset-federated-registry-test ops-deploy ops-undeploy ops-clean-uninstall ops-redeploy ops-offline ops-perf ops-multi-registry ops-ingress ops-warm ops-warmup ops-warm-datasets ops-warm-top ops-warm-shards ops-soak ops-smoke ops-diff-smoke ops-gc-smoke ops-metrics-check ops-traces-check ops-k8s-tests ops-k8s-template-tests ops-load-manifest-validate ops-load-prereqs ops-load-smoke ops-load-shedding ops-load-spike-proof ops-load-spike-chaos ops-load-full ops-load-soak ops-drill-upgrade-under-load ops-drill-rollback-under-load ops-load-ci ops-load-nightly ops-drill-store-outage ops-drill-minio-outage ops-drill-prom-outage ops-drill-otel-outage ops-drill-toxiproxy-latency ops-drill-overload ops-drill-memory-growth ops-drill-rate-limit ops-drill-corruption ops-drill-pod-churn ops-drill-upgrade ops-drill-rollback ops-drill-runner ops-drill-suite ops-upgrade-drill ops-rollback-drill ops-realdata ops-local-reset ops-local-full ops-report ops-readiness-scorecard ops-incident-repro-kit ops-script-coverage ops-shellcheck ops-lint ops-fmt ops-gen ops-gen-clean ops-gen-check ops-make-targets-doc ops-kind-version-check ops-k6-version-check ops-helm-version-check ops-kubectl-version-check ops-jq-version-check ops-yq-version-check ops-kubeconform-version-check ops-tool-check ops-tools-check ops-tools-print ops-values-validate ops-openapi-validate ops-dashboards-validate ops-alerts-validate ops-observability-validate ops-observability-smoke ops-open-grafana ops-local-full-stack ops-obs-install ops-obs-uninstall ops-obs-validate ops-obs-up ops-obs-down ops-obs-drill ops-observability-pack-verify ops-observability-pack-smoke ops-observability-pack-export ops-observability-pack-version-check ops-observability-pack-upgrade-check ops-observability-pack-health ops-observability-pack-conformance-report ops-observability-pack-idempotency ops-observability-pack-reinstall observability-pack-test observability-pack-drills obs/update-goldens ops-api-protection ops-graceful-degradation ops-release-matrix ops-baseline-policy-check ops-perf-baseline-update ops-ci ops-ci-nightly ops-ref-grade-local ops-ref-grade-pr ops-ref-grade-nightly ops-full ops-full-pr ops-full-nightly ops-idempotency-check ops-perf-prepare-store ops-perf-e2e ops-perf-nightly ops-perf-cold-start ops-perf-warm-start ops-perf-cold-start-prefetch-5pods ops-perf-compare-redis ops-perf-report ops-perf-suite e2e-local e2e-k8s-install-gate e2e-k8s-suite e2e-perf e2e-realdata observability-check layout-check layout-migrate governance-check

REGISTRY_RELEASE_DESC := release/surface
REGISTRY_RELEASE_TARGETS := fmt lint check test test-all coverage audit openapi-drift ci ssot-check crate-structure crate-docs-contract cli-command-surface query-plan-gate critical-query-check docker-build docker-smoke docker-scan docker-push chart-package chart-verify chart-validate release-dry-run release bump clean deep-clean debug

REGISTRY_CI_DESC := ci-mapping
REGISTRY_CI_TARGETS := ci-fast ci-contracts ci-docs ci-ops ci-workflows-make-only ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify ci-security-advisory-render governance-check

REGISTRY_TOOLING_DESC := tooling
REGISTRY_TOOLING_TARGETS := bootstrap bootstrap-tools doctor prereqs pins/check pins/update dataset-id-lint scripts-index scripts-graph scripts-all scripts-audit scripts-lint scripts-check scripts-format scripts-test scripts-clean artifacts-index artifacts-clean isolate-clean no-direct-scripts help

REGISTRY_META_DESC := meta
REGISTRY_META_TARGETS := ci local local-full contracts hygiene architecture-check path-contract-check
