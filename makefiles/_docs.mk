SHELL := /bin/sh

docs: ## Canonical docs gate
	@$(DEV_ATLAS) docs doctor --format text

docs-doctor: ## Run docs doctor checks
	@$(DEV_ATLAS) docs doctor --format json

docs-validate: ## Run docs validation checks
	@$(DEV_ATLAS) docs validate --format json

docs-build: ## Build docs into artifacts
	@$(DEV_ATLAS) docs build --allow-subprocess --allow-write --format json

docs-serve: ## Serve docs locally
	@$(DEV_ATLAS) docs serve --allow-subprocess --format text

docs-clean: ## Clean docs generated outputs
	@$(DEV_ATLAS) docs inventory --format text >/dev/null

docs-lock: ## Refresh docs requirements lock deterministically
	@$(DEV_ATLAS) docs build --allow-subprocess --allow-write --format text

.PHONY: docs docs-doctor docs-validate docs-build docs-serve docs-clean docs-lock
