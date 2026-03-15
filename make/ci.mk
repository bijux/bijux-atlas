# Scope: canonical CI wrappers delegated to stable command entrypoints.
# Public targets: ci, ci-fast, ci-pr, ci-nightly, ci-docs, ci-dependency-lock-refresh
SHELL := /bin/sh

ci: ## Canonical CI entrypoint
	@$(MAKE) -s dev-doctor && $(MAKE) -s dev-check-ci

ci-fast: ## CI fast lane wrapper
	@$(DEV_ATLAS) suites run --suite ci_fast --mode all --format $(FORMAT)

ci-pr: ## CI PR lane wrapper
	@$(DEV_ATLAS) suites run --suite ci_pr --mode all --format $(FORMAT)

ci-nightly: ## CI nightly lane (includes slow checks)
	@$(DEV_ATLAS) suites run --suite ci_nightly --mode all --format $(FORMAT)

ci-docs: ## CI docs lane wrapper
	@$(DEV_ATLAS) suites run --suite docs_required --mode all --format $(FORMAT)

ci-dependency-lock-refresh: ## CI dependency lock refresh wrapper
	@$(DEV_ATLAS) suites run --suite repo_required --mode all --tag lint --format $(FORMAT)

.PHONY: ci ci-fast ci-pr ci-nightly ci-docs ci-dependency-lock-refresh
