# Scope: canonical CI wrappers delegated to stable command entrypoints.
# Public targets are surfaced via root help/catalog.
SHELL := /bin/sh
BIJUX ?= bijux
BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas

ci: ## Canonical CI entrypoint
	@$(MAKE) -s dev-doctor && $(MAKE) -s dev-check-ci

ci-fast: ## CI fast lane wrapper
	@$(BIJUX_DEV_ATLAS) check run --suite ci_fast --format json

ci-nightly: ## CI nightly lane (includes slow checks)
	@$(BIJUX_DEV_ATLAS) check run --suite deep --include-internal --include-slow --format json

ci-docs: ## CI docs lane wrapper
	@$(BIJUX_DEV_ATLAS) check run --domain docs --format json

ci-cosign-sign: ## CI release signing wrapper
	@./bin/atlasctl ci cosign-sign

ci-cosign-verify: ## CI release signature verification wrapper
	@./bin/atlasctl ci cosign-verify

ci-chart-package-release: ## CI release chart packaging wrapper
	@./bin/atlasctl ci chart-package-release

ci-init-tmp: ## CI init tmp/isolation dirs wrapper
	@./bin/atlasctl ci init-tmp

ci-dependency-lock-refresh: ## CI dependency lock refresh wrapper
	@$(BIJUX_DEV_ATLAS) check run --domain root --tag lint --format json

ci-help: ## Show CI command help
	@$(BIJUX) dev atlas --help

.PHONY: ci ci-fast ci-nightly ci-docs ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-init-tmp ci-dependency-lock-refresh ci-help
