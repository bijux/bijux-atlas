# Scope: scripts area internal targets and wrappers.
# Public targets: none
SHELL := /bin/sh
PYRUN := ./scripts/bin/bijux-atlas-scripts run
SCRIPTS_VENV := artifacts/scripts/.venv

bootstrap-tools:
	@./scripts/areas/bootstrap/install_tools.sh

scripts-index:
	@$(PYRUN) scripts/areas/gen/generate_scripts_readme.py

scripts-graph: ## Generate make-target to scripts call graph
	@$(PYRUN) scripts/areas/docs/generate_scripts_graph.py

no-direct-scripts:
	@./scripts/areas/layout/check_no_direct_script_runs.sh
	@$(PYRUN) scripts/areas/layout/check_make_public_scripts.py

scripts-lint: ## Lint script surface (shellcheck + header + make/public gate + optional ruff)
	@$(MAKE) scripts-audit
	@$(PYRUN) scripts/areas/docs/check_script_headers.py
	@$(PYRUN) scripts/areas/layout/check_make_public_scripts.py
	@$(PYRUN) scripts/areas/layout/check_scripts_buckets.py
	@$(PYRUN) scripts/areas/layout/check_script_relative_calls.py
	@$(PYRUN) scripts/areas/layout/check_script_naming_convention.py
	@$(PYRUN) scripts/areas/layout/check_no_mixed_script_name_variants.py
	@$(PYRUN) scripts/areas/layout/check_duplicate_script_intent.py
	@./scripts/areas/check/no-duplicate-script-names.sh
	@./scripts/areas/check/no-direct-path-usage.sh
	@$(PYRUN) scripts/areas/check/check-script-help.py
	@$(PYRUN) scripts/areas/check/check-script-errors.py
	@$(PYRUN) scripts/areas/check/check-script-write-roots.py
	@$(PYRUN) scripts/areas/check/check-script-tool-guards.py
	@$(PYRUN) scripts/areas/check/check-script-ownership.py
	@$(PYRUN) scripts/areas/check/check-python-lock.py
	@$(PYRUN) scripts/areas/check/check-bin-entrypoints.py
	@$(PYRUN) scripts/areas/check/check-no-adhoc-python.py
	@./ops/_lint/naming.sh
	@$(PYRUN) ops/_lint/no-shadow-configs.py
	@$(PYRUN) scripts/areas/layout/check_public_entrypoint_cap.py
	@SHELLCHECK_STRICT=1 $(MAKE) -s ops-shellcheck
	@if command -v shellcheck >/dev/null 2>&1; then find scripts/areas/public scripts/areas/internal scripts/areas/dev -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional for local scripts lint)"; fi
	@if command -v shfmt >/dev/null 2>&1; then shfmt -d scripts ops/load/scripts; else echo "shfmt not installed (optional)"; fi
	@if command -v ruff >/dev/null 2>&1; then ruff check scripts ops/load/scripts; else echo "ruff not installed (optional)"; fi

scripts-format: ## Format scripts (python + shell where available)
	@if command -v ruff >/dev/null 2>&1; then ruff format scripts; else echo "ruff not installed (optional)"; fi
	@if command -v shfmt >/dev/null 2>&1; then find scripts ops/load/scripts -type f -name '*.sh' -print0 | xargs -0 shfmt -w; else echo "shfmt not installed (optional)"; fi

scripts-test: ## Run scripts-focused tests
	@$(PYRUN) scripts/areas/layout/check_make_public_scripts.py
	@$(PYRUN) scripts/areas/layout/check_script_entrypoints.py
	@$(PYRUN) scripts/areas/layout/check_scripts_top_level.py
	@$(PYRUN) ops/load/scripts/validate_suite_manifest.py
	@$(PYRUN) ops/load/scripts/check_pinned_queries_lock.py
	@python3 -m unittest scripts.areas.tests.test_paths
	@$(MAKE) -s internal/scripts/install-dev
	@PYTHONPATH=tools/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/ruff" check tools/bijux-atlas-scripts/src tools/bijux-atlas-scripts/tests
	@if [ -x "$(SCRIPTS_VENV)/bin/mypy" ]; then PYTHONPATH=tools/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/mypy" tools/bijux-atlas-scripts/src; else echo "mypy not installed (optional)"; fi
	@PYTHONPATH=tools/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/pytest" -q tools/bijux-atlas-scripts/tests
	@./scripts/bin/bijux-atlas-scripts validate-output --schema configs/contracts/scripts-tool-output.schema.json --file tools/bijux-atlas-scripts/tests/goldens/tool-output.example.json
	@./scripts/bin/bijux-atlas-scripts surface --json > artifacts/scripts/surface.json
	@./scripts/bin/bijux-atlas-scripts validate-output --schema configs/contracts/scripts-surface-output.schema.json --file artifacts/scripts/surface.json
	@./scripts/bin/bijux-atlas-scripts --run-id scripts-test --profile local doctor --json > artifacts/scripts/doctor.json
	@./scripts/bin/bijux-atlas-scripts validate-output --schema configs/contracts/scripts-doctor-output.schema.json --file artifacts/scripts/doctor.json

scripts-check: ## Run scripts lint + tests as a single gate
	@./scripts/areas/check/no-duplicate-script-names.sh
	@./scripts/areas/check/no-direct-path-usage.sh
	@$(PYRUN) scripts/areas/check/check-script-help.py
	@$(PYRUN) scripts/areas/check/check-script-errors.py
	@$(PYRUN) scripts/areas/check/check-script-write-roots.py
	@$(PYRUN) scripts/areas/check/check-script-tool-guards.py
	@$(PYRUN) scripts/areas/check/check-script-ownership.py
	@$(PYRUN) scripts/areas/check/check-python-lock.py
	@$(PYRUN) scripts/areas/check/check-no-adhoc-python.py
	@$(PYRUN) scripts/areas/layout/check_script_entrypoints.py
	@$(PYRUN) scripts/areas/layout/check_scripts_top_level.py
	@if command -v shellcheck >/dev/null 2>&1; then find scripts/areas/check scripts/bin scripts/areas/ci scripts/areas/dev -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional)"; fi
	@if command -v ruff >/dev/null 2>&1; then ruff check scripts/areas/check scripts/areas/gen scripts/areas/python tools/bijux-atlas-scripts/src tools/bijux-atlas-scripts/tests; else echo "ruff not installed (optional)"; fi
	@python3 -m unittest scripts.areas.tests.test_paths

scripts-all: ## Canonical scripts gate: all script-related gates must pass
	@$(MAKE) scripts-audit
	@$(MAKE) scripts-lint
	@$(MAKE) scripts-check
	@$(MAKE) scripts-test

scripts-audit: ## Audit script headers, taxonomy buckets, and no-implicit-cwd contract
	@$(PYRUN) scripts/areas/docs/check_script_headers.py
	@$(PYRUN) scripts/areas/layout/check_scripts_buckets.py
	@$(PYRUN) scripts/areas/layout/check_make_public_scripts.py
	@$(PYRUN) scripts/areas/layout/check_script_relative_calls.py

internal/scripts/install-dev:
	@python3 -m venv "$(SCRIPTS_VENV)"
	@"$(SCRIPTS_VENV)/bin/pip" install --upgrade pip >/dev/null
	@"$(SCRIPTS_VENV)/bin/pip" install -r tools/bijux-atlas-scripts/requirements.lock.txt >/dev/null

scripts-clean: ## Remove generated script artifacts
	@rm -rf artifacts/scripts

internal/scripts/check: ## Deterministic scripts check lane
	@start="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; status=pass; fail=""; \
	if ! $(MAKE) scripts-check; then status=fail; fail="scripts-check failed"; fi; \
	end="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	python3 ./scripts/areas/layout/write_make_area_report.py --path "$${ISO_ROOT:-artifacts/isolate/scripts/$${RUN_ID:-scripts-check}}/report.scripts.check.json" --lane "scripts/check" --status "$$status" --start "$$start" --end "$$end" --artifact "$${ISO_ROOT:-artifacts/isolate/scripts/$${RUN_ID:-scripts-check}}" --failure "$$fail" >/dev/null; \
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

.PHONY: bootstrap-tools no-direct-scripts scripts-all scripts-audit scripts-check scripts-clean scripts-format scripts-graph scripts-index scripts-lint scripts-test internal/scripts/check internal/scripts/build internal/scripts/fmt internal/scripts/lint internal/scripts/test internal/scripts/clean internal/scripts/install-dev internal/scripts/all
