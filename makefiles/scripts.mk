# Scope: scripts area internal targets and wrappers.
# Public targets: none
SHELL := /bin/sh
SCRIPTS_VENV := artifacts/isolate/py/scripts/.venv
export PYTHONDONTWRITEBYTECODE := 1
export RUFF_CACHE_DIR := $(CURDIR)/artifacts/bijux-atlas-scripts/ruff
export MYPY_CACHE_DIR := $(CURDIR)/artifacts/bijux-atlas-scripts/mypy
export HYPOTHESIS_STORAGE_DIRECTORY := $(CURDIR)/artifacts/bijux-atlas-scripts/hypothesis/examples

bootstrap-tools:
	@if command -v uv >/dev/null 2>&1; then \
		uv sync --project packages/bijux-atlas-scripts; \
	else \
		$(MAKE) -s internal/scripts/install-lock; \
	fi

scripts-index:
	@$(PY_RUN) scripts/areas/gen/generate_scripts_readme.py

scripts-graph: ## Generate make-target to scripts call graph
	@$(PY_RUN) scripts/areas/docs/generate_scripts_graph.py

no-direct-scripts:
	@./scripts/areas/layout/check_no_direct_script_runs.sh
	@$(PY_RUN) scripts/areas/layout/check_make_public_scripts.py

scripts-lint: ## Lint script surface (shellcheck + header + make/public gate + optional ruff)
	@$(MAKE) -s internal/scripts/install-lock
	@$(MAKE) scripts-audit
	@$(PY_RUN) scripts/areas/docs/check_script_headers.py
	@$(PY_RUN) scripts/areas/layout/check_make_public_scripts.py
	@$(PY_RUN) scripts/areas/layout/check_scripts_buckets.py
	@$(PY_RUN) scripts/areas/layout/check_script_relative_calls.py
	@$(PY_RUN) scripts/areas/layout/check_script_naming_convention.py
	@$(PY_RUN) scripts/areas/layout/check_no_mixed_script_name_variants.py
	@$(PY_RUN) scripts/areas/layout/check_duplicate_script_intent.py
	@./scripts/areas/check/no-duplicate-script-names.sh
	@./scripts/areas/check/no-direct-path-usage.sh
	@$(PY_RUN) scripts/areas/check/check-script-help.py
	@$(PY_RUN) scripts/areas/check/check-script-errors.py
	@$(PY_RUN) scripts/areas/check/check-script-write-roots.py
	@$(PY_RUN) scripts/areas/check/check-script-tool-guards.py
	@$(PY_RUN) scripts/areas/check/check-script-ownership.py
	@$(PY_RUN) scripts/areas/check/check-script-shim-expiry.py
	@$(PY_RUN) scripts/areas/check/check-script-shims-minimal.py
	@$(PY_RUN) scripts/areas/check/check-invocation-parity.py
	@$(PY_RUN) scripts/areas/check/check-python-lock.py
	@$(PY_RUN) scripts/areas/check/check-bin-entrypoints.py
	@$(PY_RUN) scripts/areas/check/check-root-bin-shims.py
	@./ops/_lint/no-bin-symlinks.sh
	@./ops/_lint/no-scripts-bin-dir.sh
	@$(PY_RUN) scripts/areas/check/check-no-adhoc-python.py
	@$(PY_RUN) scripts/areas/check/check-venv-location-policy.py
	@$(PY_RUN) scripts/areas/check/check-python-runtime-artifacts.py --fix
	@$(PY_RUN) scripts/areas/check/check-python-runtime-artifacts.py
	@$(PY_RUN) scripts/areas/check/check-no-make-scripts-references.py
	@$(PY_RUN) scripts/areas/check/check-repo-script-boundaries.py
	@$(PY_RUN) scripts/areas/check/check-atlas-scripts-cli-contract.py
	@$(PY_RUN) scripts/areas/check/check-scripts-surface-docs-drift.py
	@$(PY_RUN) scripts/areas/layout/check_make_command_allowlist.py
	@./ops/_lint/naming.sh
	@$(PY_RUN) ./packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout/no_shadow.py
	@$(PY_RUN) scripts/areas/layout/check_public_entrypoint_cap.py
	@SHELLCHECK_STRICT=1 $(MAKE) -s ops-shellcheck
	@if command -v shellcheck >/dev/null 2>&1; then find scripts/areas/public scripts/areas/internal -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional for local scripts lint)"; fi
	@if command -v shfmt >/dev/null 2>&1; then shfmt -d scripts ops/load/scripts; else echo "shfmt not installed (optional)"; fi
	@PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/ruff" check packages/bijux-atlas-scripts/src packages/bijux-atlas-scripts/tests

scripts-format: ## Format scripts (python + shell where available)
	@PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/ruff" format packages/bijux-atlas-scripts/src packages/bijux-atlas-scripts/tests
	@if command -v shfmt >/dev/null 2>&1; then find scripts ops/load/scripts -type f -name '*.sh' -print0 | xargs -0 shfmt -w; else echo "shfmt not installed (optional)"; fi

internal/scripts/fmt-alias: ## Alias for scripts-format
	@$(MAKE) -s scripts-format

scripts-test: ## Run scripts-focused tests
	@$(MAKE) -s internal/scripts/install-lock
	@$(PY_RUN) scripts/areas/layout/check_make_public_scripts.py
	@$(PY_RUN) scripts/areas/layout/check_script_entrypoints.py
	@$(PY_RUN) scripts/areas/layout/check_scripts_top_level.py
	@$(PY_RUN) ops/load/scripts/validate_suite_manifest.py
	@$(PY_RUN) ops/load/scripts/check_pinned_queries_lock.py
	@python3 -m unittest scripts.areas.tests.test_paths
	@$(MAKE) -s internal/scripts/install-lock
	@PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/ruff" check packages/bijux-atlas-scripts/src packages/bijux-atlas-scripts/tests
	@PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/mypy" --ignore-missing-imports packages/bijux-atlas-scripts/tests
	@PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/pytest" -q packages/bijux-atlas-scripts/tests
	@$(ATLAS_SCRIPTS) validate-output --schema configs/contracts/scripts-tool-output.schema.json --file packages/bijux-atlas-scripts/tests/goldens/tool-output.example.json
	@$(ATLAS_SCRIPTS) surface --json > artifacts/scripts/surface.json
	@$(ATLAS_SCRIPTS) validate-output --schema configs/contracts/scripts-surface-output.schema.json --file artifacts/scripts/surface.json
	@$(ATLAS_SCRIPTS) --run-id scripts-test --profile local doctor --json > artifacts/scripts/doctor.json
	@$(ATLAS_SCRIPTS) validate-output --schema configs/contracts/scripts-doctor-output.schema.json --file artifacts/scripts/doctor.json

scripts-coverage: ## Optional coverage run for bijux-atlas-scripts package
	@$(MAKE) -s internal/scripts/install-lock
	@PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/pytest" -q --cov=bijux_atlas_scripts --cov-report=term-missing packages/bijux-atlas-scripts/tests || true

scripts-deps-audit: ## Dependency policy audit for scripts package (pip-audit if available)
	@$(MAKE) -s internal/scripts/install-lock
	@{ "$(SCRIPTS_VENV)/bin/python" -m pip_audit --local --requirement packages/bijux-atlas-scripts/requirements.lock.txt || echo "pip-audit unavailable; skipping"; }

internal/scripts/test-hermetic: ## Run scripts package tests with --no-network guard enabled
	@$(MAKE) -s scripts-install
	@BIJUX_SCRIPTS_TEST_NO_NETWORK=1 PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/pytest" -q packages/bijux-atlas-scripts/tests

scripts-check: ## Run scripts lint + tests as a single gate
	@$(MAKE) -s internal/scripts/install-lock
	@$(ATLAS_SCRIPTS) python lint --json >/dev/null
	@$(ATLAS_SCRIPTS) check all
	@$(ATLAS_SCRIPTS) --quiet legacy check --report text
	@./scripts/areas/check/no-duplicate-script-names.sh
	@./scripts/areas/check/no-direct-path-usage.sh
	@$(PY_RUN) scripts/areas/check/check-no-python-executable-outside-tools.py
	@$(PY_RUN) scripts/areas/check/check-no-direct-python-invocations.py
	@$(PY_RUN) scripts/areas/check/check-no-direct-bash-invocations.py
	@$(PY_RUN) scripts/areas/check/check-python-migration-exceptions-expiry.py
	@$(PY_RUN) scripts/areas/check/check-bijux-atlas-scripts-boundaries.py
	@$(ATLAS_SCRIPTS) check cli-help
	@$(PY_RUN) scripts/areas/check/check-root-bin-shims.py
	@./ops/_lint/no-bin-symlinks.sh
	@./ops/_lint/no-scripts-bin-dir.sh
	@$(PY_RUN) scripts/areas/check/check-script-errors.py
	@$(PY_RUN) scripts/areas/check/check-script-write-roots.py
	@$(PY_RUN) scripts/areas/check/check-script-tool-guards.py
	@$(ATLAS_SCRIPTS) check ownership
	@$(PY_RUN) scripts/areas/check/check-script-shims-minimal.py
	@$(PY_RUN) scripts/areas/check/check-python-lock.py
	@$(PY_RUN) scripts/areas/check/check-scripts-lock-sync.py
	@$(PY_RUN) scripts/areas/check/check-no-adhoc-python.py
	@$(PY_RUN) scripts/areas/check/check-no-make-scripts-references.py
	@$(PY_RUN) scripts/areas/check/check-repo-script-boundaries.py
	@$(PY_RUN) scripts/areas/check/check-atlas-scripts-cli-contract.py
	@$(PY_RUN) scripts/areas/layout/check_make_command_allowlist.py
	@$(PY_RUN) scripts/areas/layout/check_script_entrypoints.py
	@$(PY_RUN) scripts/areas/layout/check_scripts_top_level.py
	@if command -v shellcheck >/dev/null 2>&1; then find scripts/areas/check scripts/bin -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional)"; fi
	@PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/ruff" check scripts/areas/check scripts/areas/gen scripts/areas/python packages/bijux-atlas-scripts/src packages/bijux-atlas-scripts/tests
	@PYTHONPATH=packages/bijux-atlas-scripts/src "$(SCRIPTS_VENV)/bin/mypy" --ignore-missing-imports packages/bijux-atlas-scripts/src packages/bijux-atlas-scripts/tests
	@python3 -m unittest scripts.areas.tests.test_paths

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
	@"$(SCRIPTS_VENV)/bin/pip" --disable-pip-version-check install --requirement packages/bijux-atlas-scripts/requirements.lock.txt >/dev/null

internal/scripts/sbom: ## Emit scripts package dependency SBOM JSON
	@$(PY_RUN) scripts/areas/check/generate-scripts-sbom.py --lock packages/bijux-atlas-scripts/requirements.lock.txt --out artifacts/evidence/scripts/sbom/$${RUN_ID:-local}/sbom.json

internal/scripts/lock-check:
	@$(PY_RUN) scripts/areas/check/check-python-lock.py
	@$(PY_RUN) scripts/areas/check/check-scripts-lock-sync.py

packages-lock: ## Refresh python lockfile deterministically from requirements.in
	@python3 -c 'from pathlib import Path; src=Path("packages/bijux-atlas-scripts/requirements.in"); dst=Path("packages/bijux-atlas-scripts/requirements.lock.txt"); lines=[ln.strip() for ln in src.read_text(encoding="utf-8").splitlines() if ln.strip() and not ln.strip().startswith("#")]; dst.write_text("\\n".join(sorted(set(lines)))+"\\n", encoding="utf-8"); print(f"updated {dst}")'

scripts-audit: ## Audit script headers, taxonomy buckets, and no-implicit-cwd contract
	@$(PY_RUN) scripts/areas/docs/check_script_headers.py
	@$(PY_RUN) scripts/areas/layout/check_scripts_buckets.py
	@$(PY_RUN) scripts/areas/layout/check_make_public_scripts.py
	@$(PY_RUN) scripts/areas/layout/check_script_relative_calls.py

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
	PYTHONPATH=packages/bijux-atlas-scripts/src python3 -m bijux_atlas_scripts.reporting.make_area_report --path "$${ISO_ROOT:-artifacts/isolate/scripts/$${RUN_ID:-scripts-check}}/report.scripts.check.json" --lane "scripts/check" --status "$$status" --start "$$start" --end "$$end" --artifact "$${ISO_ROOT:-artifacts/isolate/scripts/$${RUN_ID:-scripts-check}}" --failure "$$fail" >/dev/null; \
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

.PHONY: bootstrap-tools no-direct-scripts scripts-all scripts-audit scripts-check scripts-clean scripts-format scripts-graph scripts-index scripts-lint scripts-test scripts-coverage scripts-deps-audit internal/scripts/test-hermetic internal/scripts/sbom internal/scripts/fmt-alias internal/scripts/venv internal/scripts/install-lock internal/scripts/lock-check packages-lock internal/scripts/check internal/scripts/build internal/scripts/fmt internal/scripts/lint internal/scripts/test internal/scripts/clean internal/scripts/install-dev internal/scripts/install internal/scripts/run internal/scripts/all
