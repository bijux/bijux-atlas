# Scope: scripts area internal targets and wrappers.
# Public targets: none
SHELL := /bin/sh

bootstrap-tools:
	@./scripts/bootstrap/install_tools.sh

scripts-index:
	@python3 ./scripts/generate_scripts_readme.py

scripts-graph: ## Generate make-target to scripts call graph
	@python3 ./scripts/docs/generate_scripts_graph.py

no-direct-scripts:
	@./scripts/layout/check_no_direct_script_runs.sh
	@python3 ./scripts/layout/check_make_public_scripts.py

scripts-lint: ## Lint script surface (shellcheck + header + make/public gate + optional ruff)
	@$(MAKE) scripts-audit
	@python3 ./scripts/docs/check_script_headers.py
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./scripts/layout/check_scripts_buckets.py
	@python3 ./scripts/layout/check_script_relative_calls.py
	@python3 ./scripts/layout/check_script_naming_convention.py
	@python3 ./scripts/layout/check_no_mixed_script_name_variants.py
	@python3 ./scripts/layout/check_duplicate_script_intent.py
	@./scripts/check/no-duplicate-script-names.sh
	@./scripts/check/no-direct-path-usage.sh
	@python3 ./scripts/check/check-script-help.py
	@python3 ./scripts/check/check-script-errors.py
	@python3 ./scripts/check/check-script-write-roots.py
	@python3 ./scripts/check/check-script-tool-guards.py
	@python3 ./scripts/check/check-script-ownership.py
	@python3 ./scripts/check/check-python-lock.py
	@python3 ./scripts/check/check-bin-entrypoints.py
	@./ops/_lint/naming.sh
	@python3 ./ops/_lint/no-shadow-configs.py
	@python3 ./scripts/layout/check_public_entrypoint_cap.py
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

scripts-check: ## Run scripts lint + tests as a single gate
	@./scripts/check/no-duplicate-script-names.sh
	@./scripts/check/no-direct-path-usage.sh
	@python3 ./scripts/check/check-script-help.py
	@python3 ./scripts/check/check-script-errors.py
	@python3 ./scripts/check/check-script-write-roots.py
	@python3 ./scripts/check/check-script-tool-guards.py
	@python3 ./scripts/check/check-script-ownership.py
	@python3 ./scripts/check/check-python-lock.py
	@if command -v shellcheck >/dev/null 2>&1; then find scripts/check scripts/bin scripts/ci scripts/dev -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional)"; fi
	@if command -v ruff >/dev/null 2>&1; then ruff check scripts/check scripts/gen scripts/python; else echo "ruff not installed (optional)"; fi
	@python3 -m unittest scripts.tests.test_paths

scripts-all: ## Canonical scripts gate: all script-related gates must pass
	@$(MAKE) scripts-audit
	@$(MAKE) scripts-lint
	@$(MAKE) scripts-check
	@$(MAKE) scripts-test

scripts-audit: ## Audit script headers, taxonomy buckets, and no-implicit-cwd contract
	@python3 ./scripts/docs/check_script_headers.py
	@python3 ./scripts/layout/check_scripts_buckets.py
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./scripts/layout/check_script_relative_calls.py

scripts-clean: ## Remove generated script artifacts
	@rm -rf artifacts/scripts

internal/scripts/check: ## Deterministic scripts check lane
	@start="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; status=pass; fail=""; \
	if ! $(MAKE) scripts-check; then status=fail; fail="scripts-check failed"; fi; \
	end="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	python3 ./scripts/layout/write_make_area_report.py --path "$${ISO_ROOT:-artifacts/isolate/scripts/$${RUN_ID:-scripts-check}}/report.scripts.check.json" --lane "scripts/check" --status "$$status" --start "$$start" --end "$$end" --artifact "$${ISO_ROOT:-artifacts/isolate/scripts/$${RUN_ID:-scripts-check}}" --failure "$$fail" >/dev/null; \
	[ "$$status" = "pass" ] || { $(call fail_banner,scripts/check); exit 1; }

internal/scripts/build: ## Build script inventories/graphs
	@$(MAKE) scripts-index
	@$(MAKE) scripts-graph

internal/scripts/fmt: ## Scripts formatting
	@$(MAKE) scripts-format

internal/scripts/lint: ## Scripts lint
	@$(MAKE) scripts-lint

internal/scripts/test: ## Scripts tests
	@$(MAKE) scripts-test

internal/scripts/clean: ## Scripts generated-output cleanup
	@$(MAKE) scripts-clean

internal/scripts/all: ## Uniform scripts all target
	@$(MAKE) internal/scripts/check
	@$(MAKE) internal/scripts/lint
	@$(MAKE) internal/scripts/test
	@$(MAKE) internal/scripts/build

.PHONY: bootstrap-tools no-direct-scripts scripts-all scripts-audit scripts-check scripts-clean scripts-format scripts-graph scripts-index scripts-lint scripts-test internal/scripts/check internal/scripts/build internal/scripts/fmt internal/scripts/lint internal/scripts/test internal/scripts/clean internal/scripts/all
