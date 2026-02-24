SHELL := /bin/sh

configs: ## Canonical configs gate
	@$(DEV_ATLAS) configs doctor --format text

configs-doctor: ## Run configs doctor checks
	@$(DEV_ATLAS) configs doctor --format json

configs-validate: ## Run configs validation checks
	@$(DEV_ATLAS) configs validate --format json

configs-lint: ## Run configs lint checks
	@$(DEV_ATLAS) configs lint --format json

configs-inventory: ## List configs inventory (verification smoke target)
	@$(DEV_ATLAS) configs inventory --format json

.PHONY: configs configs-doctor configs-validate configs-lint configs-inventory
