SHELL := /bin/sh

docs: ## Canonical docs gate
	@./bin/atlasctl docs check --report text

docs-serve: ## Serve docs locally
	@./bin/atlasctl docs serve --report text

docs-clean: ## Clean docs generated outputs
	@./bin/atlasctl docs clean --report text

docs-lock: ## Refresh docs requirements lock deterministically
	@./bin/atlasctl docs requirements lock-refresh --report text

.PHONY: docs docs-serve docs-clean docs-lock
