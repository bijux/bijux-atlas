# Scope: atlasctl policy/check wrappers.
# These targets run atlasctl's own check domains directly.
SHELL := /bin/sh

atlasctl-check: ## Run all atlasctl checks across all groups
	@./bin/atlasctl check run --group all

atlasctl-check-repo: ## Run atlasctl repo checks
	@./bin/atlasctl check run --group repo

atlasctl-check-make: ## Run atlasctl makefile checks
	@./bin/atlasctl check run --group make

atlasctl-check-contracts: ## Run atlasctl contracts checks
	@./bin/atlasctl check run --group contracts

atlasctl-check-docs: ## Run atlasctl docs checks
	@./bin/atlasctl check run --group docs

atlasctl-check-ops: ## Run atlasctl ops checks
	@./bin/atlasctl check run --group ops

atlasctl-check-python: ## Run atlasctl python checks
	@./bin/atlasctl check run --group python

.PHONY: atlasctl-check atlasctl-check-repo atlasctl-check-make atlasctl-check-contracts atlasctl-check-docs atlasctl-check-ops atlasctl-check-python
