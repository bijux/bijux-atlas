SHELL := /bin/sh

include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/ci.mk
include makefiles/docs.mk
include makefiles/help.mk
include makefiles/layout.mk
include makefiles/ops.mk
include makefiles/policies.mk

bootstrap:
	@python3 --version
	@command -v pip >/dev/null 2>&1 || { echo "missing pip" >&2; exit 1; }
	@python3 -m pip install -r configs/docs/requirements.txt >/dev/null
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
	@echo 'policy: local-noise is allowed locally; CI stays clean'
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

.PHONY: help layout-check layout-migrate governance-check bootstrap bootstrap-tools scripts-index scripts-lint scripts-format scripts-test artifacts-index artifacts-clean docker-build docker-smoke chart-package chart-verify no-direct-scripts doctor fetch-real-datasets ssot-check policy-lint policy-schema-drift release-update-compat-matrix


scripts-lint: ## Lint script surface (shellcheck + header + make/public gate + optional ruff)
	@python3 ./scripts/docs/check_script_headers.py
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./scripts/layout/check_script_relative_calls.py
	@SHELLCHECK_STRICT=1 $(MAKE) -s ops-shellcheck
	@if command -v shfmt >/dev/null 2>&1; then shfmt -d scripts ops/load/scripts; else echo "shfmt not installed (optional)"; fi
	@if command -v ruff >/dev/null 2>&1; then ruff check scripts ops/load/scripts; else echo "ruff not installed (optional)"; fi

scripts-format: ## Format scripts (python + shell where available)
	@if command -v ruff >/dev/null 2>&1; then ruff format scripts; else echo "ruff not installed (optional)"; fi
	@if command -v shfmt >/dev/null 2>&1; then find scripts ops/load/scripts -type f -name '*.sh' -print0 | xargs -0 shfmt -w; else echo "shfmt not installed (optional)"; fi

scripts-test: ## Run scripts-focused tests
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./ops/load/scripts/validate_suite_manifest.py
	@python3 ./ops/load/scripts/check_pinned_queries_lock.py


artifacts-index: ## Generate artifacts index for inspection UIs
	@python3 ./scripts/layout/build_artifacts_index.py

artifacts-clean: ## Clean old artifacts with safe retention
	@python3 ./scripts/layout/clean_artifacts.py
