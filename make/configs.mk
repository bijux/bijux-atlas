# Scope: configs wrapper targets delegated to canonical control-plane commands.
# Public targets: configs, configs-doctor
SHELL := /bin/sh
BIJUX ?= bijux
BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas

CONFIGS_STRICT ?=

configs: ## Canonical configs gate
	@$(DEV_ATLAS) configs doctor $(CONFIGS_STRICT) --format $(FORMAT)

configs-doctor: ## Run configs doctor checks
	@$(DEV_ATLAS) configs doctor $(CONFIGS_STRICT) --format $(FORMAT)

configs-validate: ## Run configs validation checks
	@$(DEV_ATLAS) configs validate $(CONFIGS_STRICT) --format $(FORMAT)

configs-lint: ## Run configs lint checks
	@$(DEV_ATLAS) configs lint $(CONFIGS_STRICT) --format $(FORMAT)

configs-inventory: ## List configs inventory (verification smoke target)
	@$(DEV_ATLAS) configs inventory --format $(FORMAT)

.PHONY: configs configs-doctor configs-validate configs-lint configs-inventory
