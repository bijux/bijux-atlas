SHELL := /bin/sh
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/env.mk
include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/ci.mk
include makefiles/docs.mk
include makefiles/path_contract.mk
include makefiles/registry.mk
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

scripts-graph: ## Generate make-target to scripts call graph
	@python3 ./scripts/docs/generate_scripts_graph.py

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

chart-validate: ## Validate chart via lint/template and values contract schema checks
	@$(MAKE) chart-verify
	@./scripts/contracts/generate_chart_values_schema.py
	@./scripts/contracts/check_chart_values_contract.py

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

dataset-id-lint: ## Validate DatasetId/DatasetKey contract usage across ops fixtures
	@python3 ./scripts/layout/dataset_id_lint.py

config-validate: ## Validate config schemas/contracts and regenerate config key registry
	@python3 ./scripts/public/generate-config-key-registry.py
	@python3 ./scripts/public/config-validate.py
	@python3 ./scripts/public/config-drift-check.py

config-print: ## Print canonical merged config payload as JSON
	@python3 ./scripts/public/config-print.py

config-drift: ## Check config/schema/docs drift without regeneration
	@python3 ./scripts/public/config-drift-check.py

ci: ## Run CI-equivalent meta pipeline mapped to workflow jobs
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-root-layout
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-script-entrypoints
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-fmt
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-clippy
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-test-nextest
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-policy-lint
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-policy-schema-drift
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-config-check
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-ssot-drift
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-docs-build
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-openapi-drift
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-api-contract
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-query-plan-gate
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-sqlite-schema-drift
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-sqlite-index-drift
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-ingest-determinism
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-qc-fixtures
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-workflows-make-only
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-forbid-raw-paths
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-make-safety
	@ISO_ROOT=artifacts/isolates/ci $(MAKE) ci-make-help-drift

local: ## Fast local loop (fmt + lint + test)
	@ISO_ROOT=artifacts/isolates/local $(MAKE) fmt
	@ISO_ROOT=artifacts/isolates/local $(MAKE) lint
	@ISO_ROOT=artifacts/isolates/local $(MAKE) test

local-full: ## Full local loop (fmt + lint + audit + test + coverage + docs)
	@ISO_ROOT=artifacts/isolates/local-full $(MAKE) fmt
	@ISO_ROOT=artifacts/isolates/local-full $(MAKE) lint
	@ISO_ROOT=artifacts/isolates/local-full $(MAKE) audit
	@ISO_ROOT=artifacts/isolates/local-full $(MAKE) test
	@ISO_ROOT=artifacts/isolates/local-full $(MAKE) coverage
	@ISO_ROOT=artifacts/isolates/local-full $(MAKE) docs
	@ISO_ROOT=artifacts/isolates/local-full $(MAKE) docs-freeze

contracts: ## Contracts meta pipeline (generate + format + drift checks)
	@ISO_ROOT=artifacts/isolates/contracts $(MAKE) ssot-check
	@ISO_ROOT=artifacts/isolates/contracts $(MAKE) policy-lint
	@ISO_ROOT=artifacts/isolates/contracts $(MAKE) policy-schema-drift
	@ISO_ROOT=artifacts/isolates/contracts $(MAKE) openapi-drift
	@ISO_ROOT=artifacts/isolates/contracts $(MAKE) docs-freeze

hygiene: ## Repo hygiene checks (layout + symlink + tracked-noise gates)
	@ISO_ROOT=artifacts/isolates/hygiene $(MAKE) layout-check
	@ISO_ROOT=artifacts/isolates/hygiene $(MAKE) scripts-audit
	@ISO_ROOT=artifacts/isolates/hygiene $(MAKE) ci-workflows-make-only
	@ISO_ROOT=artifacts/isolates/hygiene $(MAKE) ci-make-help-drift
	@ISO_ROOT=artifacts/isolates/hygiene $(MAKE) path-contract-check

fetch-real-datasets:
	@./scripts/fixtures/fetch-real-datasets.sh

ssot-check:
	@./scripts/contracts/check_all.sh

policy-lint:
	@./scripts/public/policy-lint.sh

policy-schema-drift:
	@./scripts/public/policy-schema-drift.py

release-update-compat-matrix:
	@[ -n "$$TAG" ] || { echo "usage: make release-update-compat-matrix TAG=<tag>"; exit 2; }
	@./scripts/release/update-compat-matrix.sh "$$TAG"

.PHONY: help layout-check layout-migrate governance-check bootstrap bootstrap-tools scripts-index scripts-graph scripts-lint scripts-format scripts-test scripts-audit scripts-clean artifacts-index artifacts-clean isolate-clean docker-build docker-smoke chart-package chart-verify no-direct-scripts doctor dataset-id-lint config-validate config-print config-drift fetch-real-datasets ssot-check policy-lint policy-schema-drift release-update-compat-matrix ci local local-full contracts hygiene clean deep-clean debug bump release-dry-run release


scripts-lint: ## Lint script surface (shellcheck + header + make/public gate + optional ruff)
	@$(MAKE) scripts-audit
	@python3 ./scripts/docs/check_script_headers.py
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./scripts/layout/check_scripts_buckets.py
	@python3 ./scripts/layout/check_script_relative_calls.py
	@SHELLCHECK_STRICT=1 $(MAKE) -s ops-shellcheck
	@if command -v shellcheck >/dev/null 2>&1; then find scripts/public scripts/internal scripts/dev -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional for local scripts lint)"; fi
	@if command -v shfmt >/dev/null 2>&1; then shfmt -d scripts ops/load/scripts; else echo "shfmt not installed (optional)"; fi
	@if command -v ruff >/dev/null 2>&1; then ruff check scripts ops/load/scripts; else echo "ruff not installed (optional)"; fi

scripts-format: ## Format scripts (python + shell where available)
	@if command -v ruff >/dev/null 2>&1; then ruff format scripts; else echo "ruff not installed (optional)"; fi
	@if command -v shfmt >/dev/null 2>&1; then find scripts ops/load/scripts -type f -name '*.sh' -print0 | xargs -0 shfmt -w; else echo "shfmt not installed (optional)"; fi

scripts-test: ## Run scripts-focused tests
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./ops/load/scripts/validate_suite_manifest.py
	@python3 ./ops/load/scripts/check_pinned_queries_lock.py
	@python3 -m unittest scripts.tests.test_paths

scripts-audit: ## Audit script headers, taxonomy buckets, and no-implicit-cwd contract
	@python3 ./scripts/docs/check_script_headers.py
	@python3 ./scripts/layout/check_scripts_buckets.py
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./scripts/layout/check_script_relative_calls.py

scripts-clean: ## Remove generated script artifacts
	@rm -rf artifacts/scripts


artifacts-index: ## Generate artifacts index for inspection UIs
	@python3 ./scripts/layout/build_artifacts_index.py

artifacts-clean: ## Clean old artifacts with safe retention
	@python3 ./scripts/layout/clean_artifacts.py

isolate-clean: ## Remove isolate output directories safely
	@find artifacts/isolates -mindepth 1 -maxdepth 1 -type d -exec rm -r {} + 2>/dev/null || true

clean: ## Safe clean for generated local outputs
	@$(MAKE) scripts-clean
	@$(MAKE) isolate-clean
	@rm -rf artifacts/perf/results artifacts/ops/latest

deep-clean: ## Extended clean (prints and then removes generated outputs)
	@printf '%s\n' 'Deleting: artifacts/isolates artifacts/scripts artifacts/perf/results artifacts/ops'
	@$(MAKE) clean
	@rm -rf artifacts/ops

debug: ## Print internal make/env variables for maintainers
	@printf 'MAKE_PRIMARY_GOAL=%s\n' "$(MAKE_PRIMARY_GOAL)"
	@printf 'MAKE_RUN_ID=%s\n' "$(MAKE_RUN_ID)"
	@printf 'ISO_ROOT=%s\n' "$(ISO_ROOT)"
	@printf 'CARGO_TARGET_DIR=%s\n' "$(CARGO_TARGET_DIR)"
	@printf 'ATLAS_NS=%s\n' "$(ATLAS_NS)"
	@printf 'ATLAS_BASE_URL=%s\n' "$(ATLAS_BASE_URL)"
	@printf 'OPS_RUN_ID=%s\n' "$(OPS_RUN_ID)"

bump: ## Bump workspace version (usage: make bump VERSION=x.y.z)
	@[ -n "$$VERSION" ] || { echo "usage: make bump VERSION=x.y.z"; exit 2; }
	@cargo set-version --workspace "$$VERSION"

release-dry-run: ## Build + docs + ops smoke release rehearsal
	@ISO_ROOT=artifacts/isolates/release-dry-run $(MAKE) fmt
	@ISO_ROOT=artifacts/isolates/release-dry-run $(MAKE) lint
	@ISO_ROOT=artifacts/isolates/release-dry-run $(MAKE) test
	@ISO_ROOT=artifacts/isolates/release-dry-run $(MAKE) docs
	@ISO_ROOT=artifacts/isolates/release-dry-run $(MAKE) ops-full-pr

release: ## Release entrypoint (currently dry-run only)
	@$(MAKE) release-dry-run
