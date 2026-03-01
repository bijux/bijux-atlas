# Scope: docs wrapper targets delegated to bijux-dev-atlas docs surfaces.
# Public targets: docs, docs-doctor
SHELL := /bin/sh

docs: ## Canonical docs gate
	@$(DEV_ATLAS) docs doctor --format json

docs-doctor: ## Run docs doctor checks
	@$(DEV_ATLAS) docs doctor --format json

docs-validate: ## Run docs validation checks
	@$(DEV_ATLAS) docs validate --format json

docs-external-links: ## Run docs external link checks
	@$(DEV_ATLAS) docs external-links --allow-network --format json

docs-registry: ## Build docs registry and generated docs indexes
	@$(DEV_ATLAS) docs registry build --allow-write --format json

docs-registry-validate: ## Validate docs registry coverage and contracts
	@$(DEV_ATLAS) docs registry validate --format json

docs-build: ## Build docs into artifacts
	@$(DEV_ATLAS) docs build --allow-subprocess --allow-write --format json

docs-serve: ## Serve docs locally
	@$(DEV_ATLAS) docs serve --allow-subprocess --allow-network --format text

docs-clean: ## Clean docs generated outputs
	@$(DEV_ATLAS) docs inventory --format text >/dev/null

docs-lock: ## Refresh docs requirements lock deterministically
	@$(DEV_ATLAS) docs build --allow-subprocess --allow-write --format text

docs-reference-regenerate: ## Regenerate docs operations reference pages from SSOT inputs
	@$(DEV_ATLAS) docs reference generate --allow-subprocess --allow-write --format json

docs-reference-check: ## Check docs operations reference pages are regenerated
	@$(DEV_ATLAS) docs reference check --allow-subprocess --format json

.PHONY: docs docs-doctor docs-validate docs-external-links docs-registry docs-registry-validate docs-build docs-serve docs-clean docs-lock docs-reference-regenerate docs-reference-check
