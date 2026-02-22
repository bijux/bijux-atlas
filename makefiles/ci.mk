# Scope: canonical CI wrappers delegated to stable atlasctl entrypoints.
# Public targets are surfaced via root help/catalog.
SHELL := /bin/sh

ci: ## Canonical CI entrypoint
	@./bin/atlasctl ci pr --json

ci-fast: ## CI fast lane wrapper
	@./bin/atlasctl ci run --json --lane fmt --lane lint --lane test --lane contracts

ci-nightly: ## CI nightly lane (includes slow checks)
	@./bin/atlasctl ci nightly --json

ci-help: ## Show CI command help
	@./bin/atlasctl help ci

.PHONY: ci ci-fast ci-nightly ci-help
