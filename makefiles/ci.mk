# Scope: canonical CI wrappers delegated to stable atlasctl entrypoints.
# Public targets are surfaced via root help/catalog.
SHELL := /bin/sh

ci: ## Canonical CI entrypoint
	@./bin/atlasctl ci run --json --out-dir artifacts/reports/atlasctl/suite-ci

ci-fast: ## CI fast lane wrapper
	@./bin/atlasctl ci fast --json

ci-nightly: ## CI nightly lane (includes slow checks)
	@./bin/atlasctl ci nightly --json

ci-help: ## Show CI command help
	@./bin/atlasctl help ci

.PHONY: ci ci-fast ci-nightly ci-help
