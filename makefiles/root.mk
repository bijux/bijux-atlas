SHELL := /bin/sh

help:
	@printf '%s\n' \
	  'dev:' \
	  '  dev-fmt dev-lint dev-check dev-test dev-test-all dev-coverage dev-audit dev-ci dev-clean' \
	  'docs:' \
	  '  docs docs-serve docs-freeze docs-hardening' \
	  'ops:' \
	  '  ops-up ops-down ops-reset ops-publish-medium ops-deploy ops-warm ops-soak ops-smoke ops-metrics-check ops-traces-check ops-k8s-tests ops-k8s-template-tests ops-load-prereqs ops-load-smoke ops-load-full ops-drill-store-outage ops-drill-corruption ops-drill-pod-churn ops-drill-upgrade ops-drill-rollback ops-report ops-script-coverage ops-shellcheck ops-kind-version-check ops-k6-version-check ops-helm-version-check ops-kubectl-version-check ops-tools-check ops-values-validate ops-openapi-validate ops-dashboards-validate ops-alerts-validate ops-release-matrix ops-baseline-policy-check ops-ci ops-perf-prepare-store ops-perf-e2e ops-perf-nightly ops-perf-cold-start ops-perf-cold-start-prefetch-5pods ops-perf-compare-redis ops-perf-suite e2e-local e2e-k8s-install-gate e2e-k8s-suite e2e-perf e2e-realdata observability-check layout-check layout-migrate' \
	  'release/surface:' \
	  '  fmt lint check test test-all coverage audit openapi-drift ci ssot-check crate-structure crate-docs-contract cli-command-surface docker-build docker-smoke chart-package chart-verify' \
	  'tooling:' \
	  '  bootstrap bootstrap-tools doctor scripts-index help'

layout-check:
	@./scripts/layout/check_root_shape.sh
	@./scripts/layout/check_ops_canonical_shims.sh
	@./scripts/layout/check_repo_hygiene.sh
	@./scripts/layout/check_artifacts_allowlist.sh

layout-migrate:
	@./scripts/layout/migrate.sh

bootstrap:
	@python3 --version
	@command -v pip >/dev/null 2>&1 || { echo "missing pip" >&2; exit 1; }
	@python3 -m pip install -r ops/docs/requirements.txt >/dev/null
	@command -v k6 >/dev/null 2>&1 || echo "k6 not found (optional for non-perf workflows)"
	@command -v kind >/dev/null 2>&1 || echo "kind not found (required for k8s e2e)"
	@command -v kubectl >/dev/null 2>&1 || echo "kubectl not found (required for k8s e2e)"

bootstrap-tools:
	@./scripts/bootstrap/install_tools.sh

scripts-index:
	@python3 ./scripts/generate_scripts_readme.py

docker-build:
	@docker build -t bijux-atlas:local -f Dockerfile .

docker-smoke:
	@docker run --rm bijux-atlas:local --version >/dev/null
	@echo "docker smoke passed"

chart-package:
	@mkdir -p artifacts/chart
	@helm package charts/bijux-atlas --destination artifacts/chart

chart-verify:
	@helm lint charts/bijux-atlas
	@helm template atlas charts/bijux-atlas >/dev/null

no-direct-scripts:
	@./scripts/layout/check_no_direct_script_runs.sh

doctor:
	@printf 'rustc: '; rustc --version
	@printf 'cargo: '; cargo --version
	@printf 'python3: '; python3 --version
	@printf 'k6: '; (command -v k6 >/dev/null 2>&1 && k6 version 2>/dev/null | head -n1) || echo 'missing'
	@printf 'kind: '; (command -v kind >/dev/null 2>&1 && kind version 2>/dev/null | head -n1) || echo 'missing'
	@printf 'kubectl: '; (command -v kubectl >/dev/null 2>&1 && kubectl version --client 2>/dev/null | head -n1) || echo 'missing'
	@printf 'helm: '; (command -v helm >/dev/null 2>&1 && helm version --short 2>/dev/null) || echo 'missing'
	@$(MAKE) -s ops-tools-check

fetch-real-datasets:
	@./scripts/fixtures/fetch-real-datasets.sh

ssot-check:
	@./scripts/contracts/check_all.sh

policy-lint:
	@./scripts/policy-lint.sh

policy-schema-drift:
	@./scripts/policy-schema-drift.py

release-update-compat-matrix:
	@[ -n "$$TAG" ] || { echo "usage: make release-update-compat-matrix TAG=<tag>"; exit 2; }
	@./scripts/release/update-compat-matrix.sh "$$TAG"

.PHONY: help layout-check layout-migrate bootstrap bootstrap-tools scripts-index docker-build docker-smoke chart-package chart-verify no-direct-scripts doctor fetch-real-datasets ssot-check policy-lint policy-schema-drift release-update-compat-matrix
