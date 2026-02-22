SHELL := /bin/sh

docs-req-lock-refresh: ## Refresh docs requirements lock deterministically
	@./bin/atlasctl docs requirements lock-refresh --report text

docs: ## Canonical docs check gate
	@./bin/atlasctl docs check --report text

docs-all: ## Canonical docs check gate (full variant)
	@./bin/atlasctl docs check --report text --all

docs-build: ## Build docs + link-check + spell-check + lint
	@./bin/atlasctl docs build --report text

docs-build-all: ## Build docs with full checks enabled
	@./bin/atlasctl docs build --report text --all

docs-serve: ## Serve docs locally
	@./bin/atlasctl docs serve --report text

docs-freeze: ## Generated docs must be up-to-date with SSOT contracts
	@./bin/atlasctl docs freeze --report text

docs-fmt: ## Render docs diagrams and formatting surfaces
	@./bin/atlasctl docs fmt --report text

docs-fmt-all: ## Full docs format pipeline
	@./bin/atlasctl docs fmt --report text --all

docs-lint: ## Run docs lint checks
	@./bin/atlasctl docs lint --report text

docs-lint-all: ## Run full docs lint checks
	@./bin/atlasctl docs lint --report text --all

docs-test: ## Run docs verification tests (freeze + links + nav)
	@./bin/atlasctl docs test --report text

docs-test-all: ## Run full docs verification tests
	@./bin/atlasctl docs test --report text --all

docs-clean: ## Clean docs generated outputs
	@./bin/atlasctl docs clean --report text

.PHONY: docs docs-all docs-build docs-build-all docs-serve docs-freeze docs-fmt docs-fmt-all docs-lint docs-lint-all docs-test docs-test-all docs-clean docs-req-lock-refresh
