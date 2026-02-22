# Scope: atlasctl policy/check wrappers.
# These targets run atlasctl's own check domains directly.
SHELL := /bin/sh

atlasctl-check: ## Run all atlasctl checks across all groups
	@./bin/atlasctl check run --group all

atlasctl-check-repo: ## Run atlasctl repo checks
	@./bin/atlasctl check run --group repo

atlasctl-check-make: ## Run atlasctl makefile checks
	@./bin/atlasctl check run --group make

atlasctl-check-contracts: ## Run atlasctl contracts checks
	@./bin/atlasctl check run --group contracts

atlasctl-check-docs: ## Run atlasctl docs checks
	@./bin/atlasctl check run --group docs

atlasctl-check-ops: ## Run atlasctl ops checks
	@./bin/atlasctl check run --group ops

atlasctl-check-python: ## Run atlasctl python checks
	@./bin/atlasctl check run --group python

atlasctl-check-layout: ## Validate repository layout/root-shape checks
	@./bin/atlasctl check layout

# Compatibility alias for existing docs/workflows.
layout-check: ## Validate repository layout contract and root shape
	@./bin/atlasctl check layout

# Internal atlasctl dependency/tooling wrappers (migrated from scripts.mk)
deps-check-venv: ## Validate dependency install/import in a clean temporary venv
	@./bin/atlasctl deps check-venv

deps-cold-start: ## Measure atlasctl import cold-start budget
	@./bin/atlasctl deps cold-start --runs 3 --max-ms 500

deps-lock: ## Refresh python lockfile deterministically via atlasctl
	@./bin/atlasctl deps lock

deps-sync: ## Install dependencies from lock into active interpreter
	@./bin/atlasctl deps sync

registry-list: ## Print atlasctl registry inventory
	@./bin/atlasctl registry list

.PHONY: atlasctl-check atlasctl-check-repo atlasctl-check-make atlasctl-check-contracts atlasctl-check-docs atlasctl-check-ops atlasctl-check-python atlasctl-check-layout layout-check deps-check-venv deps-cold-start deps-lock deps-sync registry-list
