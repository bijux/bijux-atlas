SHELL := /bin/sh
BIJUX ?= bijux
BIJUX_DEV_ATLAS ?= cargo run -q -p bijux-dev-atlas --

configs: ## Canonical configs gate
	@$(BIJUX_DEV_ATLAS) configs doctor --format text

configs-doctor: ## Run configs doctor checks
	@$(BIJUX_DEV_ATLAS) configs doctor --format json

configs-validate: ## Run configs validation checks
	@$(BIJUX_DEV_ATLAS) configs validate --format json

configs-lint: ## Run configs lint checks
	@$(BIJUX_DEV_ATLAS) configs lint --format json

configs-inventory: ## List configs inventory (verification smoke target)
	@$(BIJUX_DEV_ATLAS) configs inventory --format json

.PHONY: configs configs-doctor configs-validate configs-lint configs-inventory
