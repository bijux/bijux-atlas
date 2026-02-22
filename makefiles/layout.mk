# Scope: repository layout/path contract internal targets.
# Public targets: none
SHELL := /bin/sh

layout-check: ## Validate repository layout contract and root shape
	@./bin/atlasctl check layout

layout-migrate: ## Apply deterministic layout/path migration helpers
	@./bin/atlasctl migration layout

layout-fix: ## Repair known layout/symlink drift deterministically
	@./bin/atlasctl migration layout

.PHONY: layout-check layout-migrate layout-fix
