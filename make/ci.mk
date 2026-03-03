# Scope: canonical CI wrappers delegated to stable command entrypoints.
# Public targets: ci, ci-fast, ci-pr, ci-nightly, ci-docs, ci-dependency-lock-refresh
SHELL := /bin/sh

ci: ## Canonical CI entrypoint
	@$(MAKE) -s dev-doctor && $(MAKE) -s dev-check-ci

ci-fast: ## CI fast lane wrapper
	@$(DEV_ATLAS) check run --suite ci_fast --format $(FORMAT)

ci-pr: ## CI PR lane wrapper
	@$(DEV_ATLAS) check run --suite ci_pr --format $(FORMAT)

ci-nightly: ## CI nightly lane (includes slow checks)
	@$(DEV_ATLAS) check run --suite ci_nightly --include-internal --include-slow --format $(FORMAT)

ci-docs: ## CI docs lane wrapper
	@$(DEV_ATLAS) check run --domain docs --format $(FORMAT)

ci-dependency-lock-refresh: ## CI dependency lock refresh wrapper
	@$(DEV_ATLAS) check run --domain root --tag lint --format $(FORMAT)

ci-help: ## Show CI command help
	@$(DEV_ATLAS) --help

.PHONY: ci ci-fast ci-pr ci-nightly ci-docs ci-dependency-lock-refresh ci-help
