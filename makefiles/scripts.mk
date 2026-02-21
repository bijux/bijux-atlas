# Scope: scripts area internal targets and wrappers.
# Public targets: none
SHELL := /bin/sh
SCRIPTS_VENV := artifacts/atlasctl/venv/.venv
export PYTHONDONTWRITEBYTECODE := 1
export RUFF_CACHE_DIR := $(CURDIR)/artifacts/atlasctl/ruff
export MYPY_CACHE_DIR := $(CURDIR)/artifacts/atlasctl/mypy
export HYPOTHESIS_STORAGE_DIRECTORY := $(CURDIR)/artifacts/atlasctl/hypothesis/examples

bootstrap-tools:
	@if command -v uv >/dev/null 2>&1; then \
		uv sync --project packages/atlasctl; \
	else \
		$(MAKE) -s internal/scripts/install-lock; \
	fi

scripts-index:
	@$(ATLAS_SCRIPTS) inventory scripts-migration --format md --out-dir docs/_generated
	@$(ATLAS_SCRIPTS) inventory scripts-migration --format json --out-dir docs/_generated

scripts-graph: ## Generate make-target to scripts call graph
	@$(ATLAS_SCRIPTS) make graph root-local > docs/development/scripts-graph.md

no-direct-scripts:
	@./packages/atlasctl/src/atlasctl/shell/layout/check_no_direct_script_runs.sh
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_public_scripts.py

scripts-lint: ## Lint script surface (shellcheck + header + make/public gate + optional ruff)
	@$(MAKE) -s internal/scripts/install-lock
	@$(MAKE) scripts-audit
	@$(ATLAS_SCRIPTS) docs script-headers-check --report text
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_public_scripts.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_scripts_buckets.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_script_relative_calls.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_script_naming_convention.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/policies/policies/check_no_mixed_script_name_variants.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/policies/policies/check_duplicate_script_intent.py
	@$(ATLAS_SCRIPTS) check duplicate-script-names
	@$(ATLAS_SCRIPTS) check layout
	@$(ATLAS_SCRIPTS) check cli-help
	@$(ATLAS_SCRIPTS) check script-errors
	@$(ATLAS_SCRIPTS) check script-write-roots
	@$(ATLAS_SCRIPTS) check script-tool-guards
	@$(ATLAS_SCRIPTS) check ownership
	@$(ATLAS_SCRIPTS) check script-shim-expiry
	@$(ATLAS_SCRIPTS) check script-shims-minimal
	@$(ATLAS_SCRIPTS) check invocation-parity
	@$(ATLAS_SCRIPTS) check python-lock
	@$(ATLAS_SCRIPTS) check bin-entrypoints
	@$(ATLAS_SCRIPTS) check root-bin-shims
	@./ops/_lint/no-bin-symlinks.sh
	@./ops/_lint/no-scripts-bin-dir.sh
	@$(ATLAS_SCRIPTS) check no-adhoc-python
	@$(ATLAS_SCRIPTS) check venv-location-policy
	@$(ATLAS_SCRIPTS) check python-runtime-artifacts --fix
	@$(ATLAS_SCRIPTS) check python-runtime-artifacts
	@$(ATLAS_SCRIPTS) check make-scripts-refs
	@$(ATLAS_SCRIPTS) check repo-script-boundaries
	@$(ATLAS_SCRIPTS) check atlas-cli-contract
	@$(ATLAS_SCRIPTS) check scripts-surface-docs-drift
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_command_allowlist.py
	@./ops/_lint/naming.sh
	@$(PY_RUN) ./packages/atlasctl/src/atlasctl/layout/no_shadow.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/public_surface/checks/check_public_entrypoint_cap.py
	@SHELLCHECK_STRICT=1 $(MAKE) -s ops-shellcheck
	@if command -v shellcheck >/dev/null 2>&1; then find packages/atlasctl/src/atlasctl/checks/layout -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional for local scripts lint)"; fi
	@if command -v shfmt >/dev/null 2>&1; then shfmt -d scripts ops/load/scripts; else echo "shfmt not installed (optional)"; fi
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/ruff" check --config packages/atlasctl/pyproject.toml packages/atlasctl/src packages/atlasctl/tests

scripts-format: ## Format scripts (python + shell where available)
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/ruff" format --config packages/atlasctl/pyproject.toml packages/atlasctl/src packages/atlasctl/tests
	@if command -v shfmt >/dev/null 2>&1; then find scripts ops/load/scripts -type f -name '*.sh' -print0 | xargs -0 shfmt -w; else echo "shfmt not installed (optional)"; fi

internal/scripts/fmt-alias: ## Alias for scripts-format
	@$(MAKE) -s scripts-format

scripts-test: ## Run scripts-focused tests
	@$(MAKE) -s internal/scripts/install-lock
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_public_scripts.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_script_entrypoints.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_scripts_top_level.py
	@$(PY_RUN) ops/load/scripts/validate_suite_manifest.py
	@$(PY_RUN) ops/load/scripts/check_pinned_queries_lock.py
	@$(MAKE) -s internal/scripts/install-lock
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/ruff" check --config packages/atlasctl/pyproject.toml packages/atlasctl/src packages/atlasctl/tests
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/mypy" packages/atlasctl/src/atlasctl/core packages/atlasctl/src/atlasctl/contracts
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/python" -m compileall -q packages/atlasctl/src
	@$(ATLAS_SCRIPTS) test run unit
	@$(ATLAS_SCRIPTS) test run integration
	@$(ATLAS_SCRIPTS) validate-output --schema atlasctl.surface.v1 --file packages/atlasctl/tests/goldens/samples/surface.sample.json
	@$(ATLAS_SCRIPTS) surface --json > artifacts/scripts/surface.json
	@$(ATLAS_SCRIPTS) validate-output --schema atlasctl.surface.v1 --file artifacts/scripts/surface.json
	@$(ATLAS_SCRIPTS) --run-id scripts-test --profile local doctor --json > artifacts/scripts/doctor.json

scripts-coverage: ## Optional coverage run for atlasctl package
	@$(MAKE) -s internal/scripts/install-lock
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/pytest" -q --cov=atlasctl --cov-report=term-missing packages/atlasctl/tests || true

scripts-deps-audit: ## Dependency policy audit for scripts package (pip-audit if available)
	@$(MAKE) -s internal/scripts/install-lock
	@{ "$(SCRIPTS_VENV)/bin/python" -m pip_audit --local --requirement packages/atlasctl/requirements.lock.txt || echo "pip-audit unavailable; skipping"; }

internal/scripts/test-hermetic: ## Run scripts package tests with --no-network guard enabled
	@$(MAKE) -s scripts-install
	@BIJUX_SCRIPTS_TEST_NO_NETWORK=1 $(ATLAS_SCRIPTS) test run unit

scripts-check: ## Run scripts lint + tests as a single gate
	@$(MAKE) -s internal/scripts/install-lock
	@$(ATLAS_SCRIPTS) python lint --json >/dev/null
	@$(ATLAS_SCRIPTS) check all
	@$(ATLAS_SCRIPTS) --quiet legacy check --report text
	@$(ATLAS_SCRIPTS) check duplicate-script-names
	@$(ATLAS_SCRIPTS) check layout
	@$(ATLAS_SCRIPTS) check no-python-shebang-outside-packages
	@$(ATLAS_SCRIPTS) check no-direct-python-invocations
	@$(ATLAS_SCRIPTS) check no-direct-bash-invocations
	@$(ATLAS_SCRIPTS) check python-migration-exceptions-expiry
	@$(ATLAS_SCRIPTS) check bijux-boundaries
	@$(ATLAS_SCRIPTS) check cli-help
	@$(ATLAS_SCRIPTS) check root-bin-shims
	@./ops/_lint/no-bin-symlinks.sh
	@./ops/_lint/no-scripts-bin-dir.sh
	@$(ATLAS_SCRIPTS) check script-errors
	@$(ATLAS_SCRIPTS) check script-write-roots
	@$(ATLAS_SCRIPTS) check script-tool-guards
	@$(ATLAS_SCRIPTS) check ownership
	@$(ATLAS_SCRIPTS) check script-shims-minimal
	@$(ATLAS_SCRIPTS) check python-lock
	@$(ATLAS_SCRIPTS) check scripts-lock-sync
	@$(ATLAS_SCRIPTS) check no-adhoc-python
	@$(ATLAS_SCRIPTS) check make-scripts-refs
	@$(ATLAS_SCRIPTS) check repo-script-boundaries
	@$(ATLAS_SCRIPTS) check atlas-cli-contract
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_command_allowlist.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_script_entrypoints.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_scripts_top_level.py
	@if command -v shellcheck >/dev/null 2>&1; then find scripts/areas/check scripts/bin -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional)"; fi
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/ruff" check --config packages/atlasctl/pyproject.toml scripts/areas/check packages/atlasctl/src packages/atlasctl/tests
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/mypy" packages/atlasctl/src/atlasctl/core packages/atlasctl/src/atlasctl/contracts

scripts-all: ## Canonical scripts gate: all script-related gates must pass
	@$(MAKE) internal/scripts/venv
	@$(MAKE) internal/scripts/install-lock
	@$(MAKE) internal/scripts/lock-check
	@$(MAKE) scripts-audit
	@$(MAKE) scripts-lint
	@$(MAKE) scripts-check
	@$(MAKE) scripts-test
	@$(MAKE) scripts-deps-audit

internal/scripts/venv:
	@python3 -m venv "$(SCRIPTS_VENV)"
	@"$(SCRIPTS_VENV)/bin/pip" --disable-pip-version-check install --upgrade pip >/dev/null

internal/scripts/install-lock:
	@$(MAKE) -s internal/scripts/venv
	@"$(SCRIPTS_VENV)/bin/pip" --disable-pip-version-check install --requirement packages/atlasctl/requirements.lock.txt >/dev/null

internal/scripts/sbom: ## Emit scripts package dependency SBOM JSON
	@$(ATLAS_SCRIPTS) check generate-scripts-sbom --lock packages/atlasctl/requirements.lock.txt --out artifacts/evidence/scripts/sbom/$${RUN_ID:-local}/sbom.json

internal/scripts/lock-check:
	@$(ATLAS_SCRIPTS) check python-lock
	@$(ATLAS_SCRIPTS) check scripts-lock-sync

deps-lock: ## Refresh python lockfile deterministically via atlasctl
	@$(ATLAS_SCRIPTS) deps lock

deps-sync: ## Install scripts deps from lock into active interpreter
	@$(ATLAS_SCRIPTS) deps sync

deps-check-venv: ## Validate dependency install/import in a clean temporary venv
	@$(ATLAS_SCRIPTS) deps check-venv

deps-cold-start: ## Measure atlasctl import cold-start budget
	@$(ATLAS_SCRIPTS) deps cold-start --runs 3 --max-ms 500

packages-lock: ## Backward-compatible alias for deps-lock
	@$(MAKE) -s deps-lock

scripts-audit: ## Audit script headers, taxonomy buckets, and no-implicit-cwd contract
	@$(ATLAS_SCRIPTS) docs script-headers-check --report text
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_scripts_buckets.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_public_scripts.py
	@$(PY_RUN) packages/atlasctl/src/atlasctl/checks/layout/scripts/check_script_relative_calls.py

internal/scripts/install-dev:
	@$(MAKE) -s internal/scripts/install-lock

internal/scripts/install:
	@$(MAKE) -s internal/scripts/install-lock

internal/scripts/run:
	@[ -n "$${CMD:-}" ] || { echo "usage: make scripts-run CMD='doctor --json'" >&2; exit 2; }
	@$(ATLAS_SCRIPTS) $${CMD}

scripts-clean: ## Remove generated script artifacts
	@rm -rf artifacts/scripts

internal/scripts/check: ## Deterministic scripts check lane
	@start="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; status=pass; fail=""; \
	if ! $(MAKE) scripts-check; then status=fail; fail="scripts-check failed"; fi; \
	end="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	$(ATLAS_SCRIPTS) report make-area-write --path "$${ISO_ROOT:-artifacts/isolate/scripts/$${RUN_ID:-scripts-check}}/report.scripts.check.json" --lane "scripts/check" --run-id "$${RUN_ID:-scripts-check}" --status "$$status" --start "$$start" --end "$$end" --artifact "$${ISO_ROOT:-artifacts/isolate/scripts/$${RUN_ID:-scripts-check}}" --failure "$$fail" >/dev/null; \
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

.PHONY: bootstrap-tools no-direct-scripts scripts-all scripts-audit scripts-check scripts-clean scripts-format scripts-graph scripts-index scripts-lint scripts-test scripts-coverage scripts-deps-audit deps-lock deps-sync deps-check-venv deps-cold-start internal/scripts/test-hermetic internal/scripts/sbom internal/scripts/fmt-alias internal/scripts/venv internal/scripts/install-lock internal/scripts/lock-check packages-lock internal/scripts/check internal/scripts/build internal/scripts/fmt internal/scripts/lint internal/scripts/test internal/scripts/clean internal/scripts/install-dev internal/scripts/install internal/scripts/run internal/scripts/all
