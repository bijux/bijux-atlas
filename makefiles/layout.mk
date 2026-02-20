# Scope: repository layout/path contract internal targets.
# Public targets: none
SHELL := /bin/sh

layout-check: ## Validate repository layout contract and root shape
	@$(ATLAS_SCRIPTS) check layout

layout-migrate: ## Apply deterministic layout/path migration helpers
	@./scripts/areas/layout/replace_paths.sh --apply
	@$(ATLAS_SCRIPTS) migrate layout

layout-fix: ## Repair known layout/symlink drift deterministically
	@$(MAKE) layout-migrate

.PHONY: layout-check layout-migrate layout-fix
