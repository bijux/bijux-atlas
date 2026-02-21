# Scope: deprecated cargo-dev compatibility aliases.
# Public targets: none
SHELL := /bin/sh

_DEPRECATED := "deprecated target: use \`make dev-*\` wrappers from makefiles/dev.mk or \`./bin/atlasctl dev ...\` directly"

dev-fmt: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

dev-lint: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

dev-check: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

dev-test: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

dev-test-all: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

dev-audit: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

dev-coverage: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

dev-ci: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

dev-clean: ## DEV_ONLY=1
	@echo $(_DEPRECATED) >&2; exit 2

.PHONY: dev-fmt dev-lint dev-check dev-test dev-test-all dev-audit dev-coverage dev-ci dev-clean
