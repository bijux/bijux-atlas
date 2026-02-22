# Scope: atlasctl policy/check wrappers.
# These targets run atlasctl's own check domains directly.
SHELL := /bin/sh
PYTHONPATH ?= packages/atlasctl/src
ATLASCTL_ARTIFACT_ROOT ?= artifacts/atlasctl
ATLASCTL ?= ./bin/atlasctl

atlasctl-check: ## Run all atlasctl checks across all groups
	@./bin/atlasctl check run --group all

atlasctl-check-contracts: ## Run atlasctl contracts checks
	@./bin/atlasctl check run --group contracts

atlasctl-check-docs: ## Run atlasctl docs checks
	@./bin/atlasctl check run --group docs

atlasctl-check-layout: ## Validate repository layout/root-shape checks
	@./bin/atlasctl check layout

atlasctl-check-make: ## Run atlasctl makefile checks
	@./bin/atlasctl check run --group make

atlasctl-check-ops: ## Run atlasctl ops checks
	@./bin/atlasctl check run --group ops

atlasctl-check-python: ## Run atlasctl python checks
	@./bin/atlasctl check run --group python

atlasctl-check-repo: ## Run atlasctl repo checks
	@./bin/atlasctl check run --group repo

atlasctl-registry-list: ## Print atlasctl registry inventory
	@./bin/atlasctl registry checks

internal/cli-check:
	@./bin/atlasctl --version >/dev/null 2>&1 || { \
		echo "atlasctl CLI is not runnable via ./bin/atlasctl"; \
		echo "run: make internal/deps/sync or make dev-bootstrap"; \
		exit 2; \
	}

# Internal atlasctl dependency/tooling wrappers.
internal/deps/check-venv: ## Validate dependency install/import in a clean temporary venv
	@./bin/atlasctl deps check-venv

internal/deps/cold-start: ## Measure atlasctl import cold-start budget
	@./bin/atlasctl deps cold-start --runs 3 --max-ms 500

internal/deps/lock: ## Refresh python lockfile deterministically via atlasctl
	@./bin/atlasctl deps lock

internal/deps/sync: ## Install dependencies from lock into active interpreter
	@./bin/atlasctl deps sync

.PHONY: atlasctl-check atlasctl-check-contracts atlasctl-check-docs atlasctl-check-layout atlasctl-check-make atlasctl-check-ops atlasctl-check-python atlasctl-check-repo atlasctl-registry-list internal/cli-check internal/deps/check-venv internal/deps/cold-start internal/deps/lock internal/deps/sync
