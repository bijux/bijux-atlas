SHELL := /bin/sh

docs-req-lock-refresh: ## Refresh docs requirements lock deterministically
	@./bin/atlasctl docs requirements lock-refresh --report text

docs-build: ## Build docs + link-check + spell-check + lint
	@./bin/atlasctl docs build --report text

docs-serve: ## Serve docs locally
	@./bin/atlasctl docs serve --report text

docs-freeze: ## Generated docs must be up-to-date with SSOT contracts
	@./bin/atlasctl docs freeze --report text

docs-hardening: ## Run full docs hardening pipeline
	@./bin/atlasctl docs test --report text

docs-all: ## Canonical all-docs gate: must pass all docs sub-gates
	@./bin/atlasctl docs build --report text --all

docs-check: ## Docs contract check alias (same as docs-build)
	@./bin/atlasctl docs check --report text --emit-artifacts

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

docs-build-all: ## Build docs with full checks enabled
	@./bin/atlasctl docs build --report text --all

docs-check-all: ## Run full docs check gate
	@./bin/atlasctl docs check --report text --emit-artifacts --all

docs: ## Canonical docs gate
	@./bin/atlasctl docs check --report text

internal/docs/public: ## Public docs alias implementation (root wrapper only)
	@./bin/atlasctl docs check --report text

internal/docs/check: ## Fast docs verification
	@./bin/atlasctl docs check --report text

internal/docs/build: ## Build docs artifacts
	@./bin/atlasctl docs build --report text

internal/docs/fmt: ## Docs formatting helpers
	@./bin/atlasctl docs fmt --report text

internal/docs/lint: ## Docs lint checks
	@./bin/atlasctl docs lint --report text

internal/docs/test: ## Docs tests/contract checks
	@./bin/atlasctl docs test --report text

internal/docs/clean: ## Clean docs generated outputs only
	@./bin/atlasctl docs clean --report text

internal/docs/all: ## Uniform docs all target
	@./bin/atlasctl docs build --report text --all

.PHONY: docs docs-all docs-build docs-build-all docs-check docs-check-all docs-serve docs-freeze docs-fmt docs-fmt-all docs-lint docs-lint-all docs-test docs-test-all docs-hardening docs-req-lock-refresh internal/docs/public internal/docs/check internal/docs/build internal/docs/fmt internal/docs/lint internal/docs/test internal/docs/clean internal/docs/all
