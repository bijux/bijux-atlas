# Scope: configs wrapper targets delegated to canonical control-plane commands.
# Public targets: configs, configs-doctor
SHELL := /bin/sh

CONFIGS_STRICT ?=

configs: ## Canonical configs gate
	@$(DEV_ATLAS) configs doctor $(CONFIGS_STRICT) --format text

configs-doctor: ## Run configs doctor checks
	@$(DEV_ATLAS) configs doctor $(CONFIGS_STRICT) --format json

configs-validate: ## Run configs validation checks
	@$(DEV_ATLAS) configs validate $(CONFIGS_STRICT) --format json

configs-lint: ## Run configs lint checks
	@$(DEV_ATLAS) configs lint $(CONFIGS_STRICT) --format json

configs-inventory: ## List configs inventory (verification smoke target)
	@$(DEV_ATLAS) configs inventory --format json

configs-check: ## Back-compat alias to canonical configs doctor wrapper
	@$(MAKE) -s configs-doctor

.PHONY: configs configs-doctor configs-validate configs-lint configs-inventory configs-check
