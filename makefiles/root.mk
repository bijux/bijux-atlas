SHELL := /bin/sh

include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/docs.mk
include makefiles/ops.mk
include makefiles/policies.mk

.DEFAULT_GOAL := help

help:
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
	  '  ops-stack-up ops-up ops-stack-down ops-down ops-stack-validate ops-stack-smoke ops-stack-health-report ops-stack-version ops-stack-uninstall ops-stack-slow-store ops-reset ops-clean ops-env-print ops-cluster-sanity ops-publish ops-publish-medium ops-deploy ops-offline ops-perf ops-multi-registry ops-ingress ops-warm ops-soak ops-smoke ops-metrics-check ops-traces-check ops-k8s-tests ops-k8s-template-tests ops-load-manifest-validate ops-load-prereqs ops-load-smoke ops-load-full ops-load-ci ops-load-nightly ops-drill-store-outage ops-drill-minio-outage ops-drill-prom-outage ops-drill-otel-outage ops-drill-toxiproxy-latency ops-drill-overload ops-drill-memory-growth ops-drill-corruption ops-drill-pod-churn ops-drill-upgrade ops-drill-rollback ops-upgrade-drill ops-rollback-drill ops-realdata ops-report ops-script-coverage ops-shellcheck ops-kind-version-check ops-k6-version-check ops-helm-version-check ops-kubectl-version-check ops-kubeconform-version-check ops-tool-check ops-tools-check ops-values-validate ops-openapi-validate ops-dashboards-validate ops-alerts-validate ops-observability-validate ops-observability-smoke ops-obs-install ops-obs-uninstall ops-obs-validate ops-obs-up ops-obs-down ops-release-matrix ops-baseline-policy-check ops-perf-baseline-update ops-ci ops-ci-nightly ops-full ops-full-pr ops-full-nightly ops-perf-prepare-store ops-perf-e2e ops-perf-nightly ops-perf-cold-start ops-perf-cold-start-prefetch-5pods ops-perf-compare-redis ops-perf-report ops-perf-suite e2e-local e2e-k8s-install-gate e2e-k8s-suite e2e-perf e2e-realdata observability-check layout-check layout-migrate' \
	  'release/surface:' \
	  '  fmt lint check test test-all coverage audit openapi-drift ci ssot-check crate-structure crate-docs-contract cli-command-surface docker-build docker-smoke chart-package chart-verify' \
	  'tooling:' \
	  '  bootstrap bootstrap-tools doctor scripts-index scripts-lint scripts-test artifacts-index artifacts-clean no-direct-scripts help'

layout-check:
	@./scripts/layout/check_root_shape.sh
	@./scripts/layout/check_forbidden_root_names.sh
	@./scripts/layout/check_no_forbidden_paths.sh
	@./scripts/layout/check_ops_canonical_shims.sh
	@./scripts/layout/check_repo_hygiene.sh
	@./scripts/layout/check_artifacts_allowlist.sh
	@./scripts/layout/check_symlink_index.sh
	@./scripts/layout/check_chart_canonical_path.sh

layout-migrate:
	@./scripts/layout/replace_paths.sh --apply
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
	@docker build -t bijux-atlas:local -f docker/Dockerfile .

docker-smoke:
	@docker run --rm bijux-atlas:local --version >/dev/null
	@echo "docker smoke passed"

chart-package:
	@mkdir -p artifacts/chart
	@helm package ops/k8s/charts/bijux-atlas --destination artifacts/chart

chart-verify:
	@helm lint ops/k8s/charts/bijux-atlas
	@helm template atlas ops/k8s/charts/bijux-atlas >/dev/null

no-direct-scripts:
	@./scripts/layout/check_no_direct_script_runs.sh
	@python3 ./scripts/layout/check_make_public_scripts.py

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

.PHONY: help layout-check layout-migrate bootstrap bootstrap-tools scripts-index scripts-lint scripts-test artifacts-index artifacts-clean docker-build docker-smoke chart-package chart-verify no-direct-scripts doctor fetch-real-datasets ssot-check policy-lint policy-schema-drift release-update-compat-matrix


scripts-lint: ## Lint script surface (shellcheck + header + make/public gate + optional ruff)
	@python3 ./scripts/docs/check_script_headers.py
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./scripts/layout/check_script_relative_calls.py
	@SHELLCHECK_STRICT=1 $(MAKE) -s ops-shellcheck
	@if command -v shfmt >/dev/null 2>&1; then shfmt -d scripts ops/load/scripts; else echo "shfmt not installed (optional)"; fi
	@if command -v ruff >/dev/null 2>&1; then ruff check scripts ops/load/scripts; else echo "ruff not installed (optional)"; fi

scripts-test: ## Run scripts-focused tests
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./ops/load/scripts/validate_suite_manifest.py
	@python3 ./ops/load/scripts/check_pinned_queries_lock.py


artifacts-index: ## Generate artifacts index for inspection UIs
	@python3 ./scripts/layout/build_artifacts_index.py

artifacts-clean: ## Clean old artifacts with safe retention
	@python3 ./scripts/layout/clean_artifacts.py
