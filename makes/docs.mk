# Scope: docs wrapper targets delegated to bijux-dev-atlas docs surfaces.
# Public targets: docs, docs-doctor
SHELL := /bin/sh
BIJUX ?= bijux
BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas

docs: ## Canonical docs gate
	@$(DEV_ATLAS) docs doctor --format $(FORMAT)

docs-doctor: ## Run docs doctor checks
	@$(DEV_ATLAS) docs doctor --format $(FORMAT)

docs-check: ## Validate and build the docs surface through dev-atlas
	@$(MAKE) -s docs-validate FORMAT=$(FORMAT)
	@$(MAKE) -s docs-build FORMAT=$(FORMAT)

docs-validate: ## Run docs validation checks
	@$(DEV_ATLAS) docs validate --format $(FORMAT)

docs-external-links: ## Run docs external link checks
	@$(DEV_ATLAS) docs external-links --allow-network --format $(FORMAT)

docs-build: ## Build docs into artifacts
	@$(DEV_ATLAS) docs build --allow-subprocess --allow-write --format $(FORMAT)

docs-serve: ## Serve docs locally
	@$(DEV_ATLAS) docs serve --allow-subprocess --allow-network --format $(FORMAT)

docs-clean: ## Clean docs generated outputs
	@$(DEV_ATLAS) docs clean --allow-subprocess --allow-write --format $(FORMAT)

docs-reference-regenerate: ## Regenerate docs operations reference pages from SSOT inputs
	@$(DEV_ATLAS) docs reference generate --allow-subprocess --allow-write --format $(FORMAT)

docs-reference-check: ## Check docs operations reference pages are regenerated
	@$(DEV_ATLAS) docs reference check --allow-subprocess --format $(FORMAT)

.PHONY: docs docs-doctor docs-check docs-validate docs-external-links docs-build docs-serve docs-clean docs-reference-regenerate docs-reference-check
