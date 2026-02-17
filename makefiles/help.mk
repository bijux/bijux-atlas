SHELL := /bin/sh

.DEFAULT_GOAL := help

help: ## Show categorized make targets
	@printf '%s\n' \
	  'dev:' \
	  '  dev-fmt dev-lint dev-check dev-test dev-test-all dev-coverage dev-audit dev-ci dev-clean' \
	  'docs:' \
	  '  docs docs-serve docs-freeze docs-hardening' \
	  'contracts:' \
	  '  ssot-check policy-lint policy-schema-drift openapi-drift ops-values-validate ops-openapi-validate ops-dashboards-validate ops-alerts-validate' \
	  'ops quick:' \
	  '  ops-full ops-full-pr ops-full-nightly' \
	  'ops:' \
	  '  ops-stack-up ops-up ops-stack-down ops-down ops-stack-validate ops-stack-smoke ops-stack-health-report ops-stack-version ops-stack-uninstall ops-stack-slow-store ops-reset ops-clean ops-env-print ops-cluster-sanity ops-publish ops-publish-medium ops-deploy ops-offline ops-perf ops-multi-registry ops-ingress ops-warm ops-soak ops-smoke ops-metrics-check ops-traces-check ops-k8s-tests ops-k8s-template-tests ops-load-manifest-validate ops-load-prereqs ops-load-smoke ops-load-full ops-load-ci ops-load-nightly ops-drill-store-outage ops-drill-minio-outage ops-drill-prom-outage ops-drill-otel-outage ops-drill-toxiproxy-latency ops-drill-overload ops-drill-memory-growth ops-drill-corruption ops-drill-pod-churn ops-drill-upgrade ops-drill-rollback ops-upgrade-drill ops-rollback-drill ops-realdata ops-report ops-script-coverage ops-shellcheck ops-kind-version-check ops-k6-version-check ops-helm-version-check ops-kubectl-version-check ops-kubeconform-version-check ops-tool-check ops-tools-check ops-values-validate ops-openapi-validate ops-dashboards-validate ops-alerts-validate ops-observability-validate ops-observability-smoke ops-obs-install ops-obs-uninstall ops-obs-validate ops-obs-up ops-obs-down ops-release-matrix ops-baseline-policy-check ops-perf-baseline-update ops-ci ops-ci-nightly ops-full ops-full-pr ops-full-nightly ops-perf-prepare-store ops-perf-e2e ops-perf-nightly ops-perf-cold-start ops-perf-cold-start-prefetch-5pods ops-perf-compare-redis ops-perf-report ops-perf-suite e2e-local e2e-k8s-install-gate e2e-k8s-suite e2e-perf e2e-realdata observability-check layout-check layout-migrate governance-check' \
	  'release/surface:' \
	  '  fmt lint check test test-all coverage audit openapi-drift ci ssot-check crate-structure crate-docs-contract cli-command-surface docker-build docker-smoke chart-package chart-verify' \
	  'ci-mapping:' \
	  '  ci-root-layout ci-script-entrypoints ci-fmt ci-clippy ci-test-nextest ci-deny ci-audit ci-license-check ci-policy-lint ci-policy-schema-drift ci-ssot-drift ci-crate-structure ci-crate-docs-contract ci-cli-command-surface ci-release-binaries ci-docs-build ci-latency-regression ci-store-conformance ci-openapi-drift ci-query-plan-gate ci-compatibility-matrix-validate ci-runtime-security-scan-image ci-coverage ci-workflows-make-only' \
	  'tooling:' \
	  '  bootstrap bootstrap-tools doctor scripts-index scripts-lint scripts-test artifacts-index artifacts-clean no-direct-scripts help'

.PHONY: help
