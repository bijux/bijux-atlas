# Scope: canonical CI wrappers delegated to stable command entrypoints.
# Public targets: ci, ci-fast, ci-pr, ci-nightly, ci-docs, ci-dependency-lock-refresh
SHELL := /bin/sh

ci: ## Canonical CI entrypoint
	@$(MAKE) -s dev-doctor && $(MAKE) -s dev-check-ci

ci-fast: ## CI fast lane wrapper
	@$(DEV_ATLAS) check run --suite ci_fast --include-internal --include-slow --format $(FORMAT)

ci-pr: ## CI PR lane wrapper
	@$(DEV_ATLAS) check run --suite ci_pr --include-internal --include-slow --allow-git --format $(FORMAT)

ci-nightly: ## CI nightly lane (includes slow checks)
	@$(DEV_ATLAS) check run --suite ci_nightly --include-internal --include-slow --format $(FORMAT)

ci-docs: ## CI docs lane wrapper
	@$(DEV_ATLAS) check run --suite docs_required --include-internal --include-slow --format $(FORMAT)

ci-dependency-lock-refresh: ## CI dependency lock refresh wrapper
	@$(DEV_ATLAS) ci verify dependency-lock --allow-git --format $(FORMAT)

.PHONY: ci ci-fast ci-pr ci-nightly ci-docs ci-dependency-lock-refresh
