SHELL := /bin/sh
BIJUX ?= bijux
BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas

docs: ## Canonical docs gate
	@$(BIJUX_DEV_ATLAS) docs doctor --format text

docs-doctor: ## Run docs doctor checks
	@$(BIJUX_DEV_ATLAS) docs doctor --format json

docs-validate: ## Run docs validation checks
	@$(BIJUX_DEV_ATLAS) docs validate --format json

docs-build: ## Build docs into artifacts
	@$(BIJUX_DEV_ATLAS) docs build --allow-subprocess --allow-write --format json

docs-serve: ## Serve docs locally
	@$(BIJUX_DEV_ATLAS) docs serve --allow-subprocess --format text

docs-clean: ## Clean docs generated outputs
	@$(BIJUX_DEV_ATLAS) docs grep atlasctl --format text >/dev/null

docs-lock: ## Refresh docs requirements lock deterministically
	@$(BIJUX_DEV_ATLAS) docs build --allow-subprocess --allow-write --format text

.PHONY: docs docs-doctor docs-validate docs-build docs-serve docs-clean docs-lock
