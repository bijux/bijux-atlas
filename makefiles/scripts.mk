# Scope: auxiliary atlasctl tooling wrappers.
# Public targets: scripts-* and deps-* wrappers only.
SHELL := /bin/sh

bootstrap-tools: ## Bootstrap developer tooling
	@./bin/atlasctl deps sync

scripts-index: ## Generate control-plane inventory artifacts
	@./bin/atlasctl inventory all --format md --out-dir docs/_generated
	@./bin/atlasctl inventory all --format json --out-dir docs/_generated

scripts-graph: ## Generate scripts graph documentation
	@./bin/atlasctl docs generate-scripts-graph --report text

no-direct-scripts: ## Check direct script invocation policy
	@./bin/atlasctl check run checks_repo_no_direct_script_runs

scripts-lint: ## Lint scripts/python surface via atlasctl
	@./bin/atlasctl lint scripts --report json

scripts-format: ## Format scripts/python surface via atlasctl
	@./bin/atlasctl lint scripts --report json

scripts-test: ## Run scripts-focused tests via atlasctl
	@./bin/atlasctl test run unit
	@./bin/atlasctl test run integration

scripts-check: ## Canonical scripts gate (repo checks + python lint)
	@./bin/atlasctl ci scripts --json

scripts-all: ## Full scripts lane
	@./bin/atlasctl ci scripts --json

scripts-clean: ## Clean scripts artifacts
	@./bin/atlasctl clean artifacts --scope scripts

scripts-audit: ## Audit scripts surface
	@./bin/atlasctl check run --group repo --json

scripts-coverage: ## Deprecated: scripts coverage removed
	@echo "[DEPRECATED] scripts-coverage removed; use atlasctl test/coverage suites" >&2
	@exit 2

scripts-deps-audit: ## Deprecated: scripts deps audit removed
	@echo "[DEPRECATED] scripts-deps-audit removed; use atlasctl ci deps" >&2
	@exit 2

deps-lock: ## Refresh python lockfile deterministically via atlasctl
	@./bin/atlasctl deps lock

deps-sync: ## Install scripts deps from lock into active interpreter
	@./bin/atlasctl deps sync

deps-check-venv: ## Validate dependency install/import in a clean temporary venv
	@./bin/atlasctl deps check-venv

deps-cold-start: ## Measure atlasctl import cold-start budget
	@./bin/atlasctl deps cold-start --runs 3 --max-ms 500

packages-lock: ## Backward-compatible alias for deps-lock
	@./bin/atlasctl deps lock

scripts-run: ## Run atlasctl command (usage: make scripts-run CMD="doctor --json")
	@[ -n "$${CMD:-}" ] || { echo "usage: make scripts-run CMD='doctor --json'" >&2; exit 2; }
	@./bin/atlasctl $${CMD}

.PHONY: bootstrap-tools no-direct-scripts scripts-all scripts-audit scripts-check scripts-clean scripts-format scripts-graph scripts-index scripts-lint scripts-test scripts-coverage scripts-deps-audit deps-lock deps-sync deps-check-venv deps-cold-start packages-lock scripts-run
