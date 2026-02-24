SHELL := /bin/sh
BIJUX ?= bijux
BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas

configs: ## Canonical configs gate
	@$(BIJUX_DEV_ATLAS) configs doctor --format text

configs-doctor: ## Run configs doctor checks
	@$(BIJUX_DEV_ATLAS) configs doctor --format json

configs-validate: ## Run configs validation checks
	@$(BIJUX_DEV_ATLAS) configs validate --format json

configs-lint: ## Run configs lint checks
	@$(BIJUX_DEV_ATLAS) configs lint --format json

.PHONY: configs configs-doctor configs-validate configs-lint
