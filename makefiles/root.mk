SHELL := /bin/sh

help:
	@printf '%s\n' \
	  'dev:' \
	  '  dev-fmt dev-lint dev-check dev-test dev-test-all dev-coverage dev-audit dev-ci dev-clean' \
	  'docs:' \
	  '  docs docs-serve docs-freeze docs-hardening' \
	  'ops:' \
	  '  ops-up ops-down ops-reset ops-publish-medium ops-deploy ops-warm ops-soak ops-smoke ops-metrics-check ops-traces-check ops-k8s-tests ops-k8s-template-tests ops-load-prereqs ops-load-smoke ops-load-full ops-drill-store-outage ops-drill-corruption ops-drill-pod-churn ops-drill-upgrade ops-drill-rollback ops-report ops-script-coverage ops-kind-version-check ops-k6-version-check ops-helm-version-check ops-kubectl-version-check ops-values-validate ops-openapi-validate ops-dashboards-validate ops-alerts-validate ops-release-matrix ops-baseline-policy-check ops-ci ops-perf-prepare-store ops-perf-e2e ops-perf-nightly ops-perf-cold-start ops-perf-cold-start-prefetch-5pods ops-perf-compare-redis ops-perf-suite e2e-local e2e-k8s-install-gate e2e-k8s-suite e2e-perf e2e-realdata observability-check layout-check layout-migrate' \
	  'release/surface:' \
	  '  fmt lint check test test-all coverage audit openapi-drift ci ssot-check crate-structure crate-docs-contract cli-command-surface' \
	  'tooling:' \
	  '  bootstrap doctor help'

layout-check:
	@./scripts/layout/check_root_shape.sh

layout-migrate:
	@./scripts/layout/migrate.sh

bootstrap:
	@python3 --version
	@command -v pip >/dev/null 2>&1 || { echo "missing pip" >&2; exit 1; }
	@python3 -m pip install -r ops/docs/requirements.txt >/dev/null
	@command -v k6 >/dev/null 2>&1 || echo "k6 not found (optional for non-perf workflows)"
	@command -v kind >/dev/null 2>&1 || echo "kind not found (required for k8s e2e)"
	@command -v kubectl >/dev/null 2>&1 || echo "kubectl not found (required for k8s e2e)"


doctor:
	@printf 'rustc: '; rustc --version
	@printf 'cargo: '; cargo --version
	@printf 'python3: '; python3 --version
	@printf 'k6: '; (command -v k6 >/dev/null 2>&1 && k6 version 2>/dev/null | head -n1) || echo 'missing'
	@printf 'kind: '; (command -v kind >/dev/null 2>&1 && kind version 2>/dev/null | head -n1) || echo 'missing'
	@printf 'kubectl: '; (command -v kubectl >/dev/null 2>&1 && kubectl version --client --short 2>/dev/null) || echo 'missing'
	@printf 'helm: '; (command -v helm >/dev/null 2>&1 && helm version --short 2>/dev/null) || echo 'missing'

fetch-real-datasets:
	@./scripts/fixtures/fetch-real-datasets.sh

ssot-check:
	@./scripts/contracts/check_all.sh

.PHONY: help layout-check layout-migrate bootstrap doctor fetch-real-datasets ssot-check
