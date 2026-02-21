# Scope: canonical CI wrappers delegated to stable atlasctl entrypoints.
# Public targets are surfaced via root help/catalog.
SHELL := /bin/sh

ci: ## Canonical CI entrypoint
	@./bin/atlasctl ci run --json --out-dir artifacts/reports/atlasctl/suite-ci

internal/ci/run: ## Internal CI wrapper: canonical run
	@./bin/atlasctl ci run --json

ci-fast: ## CI fast lane wrapper
	@./bin/atlasctl ci fast --json

ci-all: ## CI all lanes wrapper
	@./bin/atlasctl ci all --json

ci-pr: ## CI PR lane (fast checks only)
	@./bin/atlasctl ci pr --json

ci-nightly: ## CI nightly lane (includes slow checks)
	@./bin/atlasctl ci nightly --json

ci-contracts: ## CI contracts lane wrapper
	@./bin/atlasctl ci contracts --json

ci-docs: ## CI docs lane wrapper
	@./bin/atlasctl ci docs --json

ci-ops: ## CI ops lane wrapper
	@./bin/atlasctl ci ops --json

ci-release: ## CI release lane wrapper
	@./bin/atlasctl ci release --json

ci-release-all: ## CI release full lane wrapper
	@./bin/atlasctl ci release-all --json

ci-init: ## CI helper lane: initialize isolate/tmp directories
	@./bin/atlasctl ci init --json

ci-artifacts: ## Show CI artifact output locations
	@./bin/atlasctl ci artifacts --json

ci-help: ## Show CI command help
	@./bin/atlasctl help ci

.PHONY: ci internal/ci/run ci-fast ci-all ci-pr ci-nightly ci-contracts ci-docs ci-ops ci-release ci-release-all ci-init ci-artifacts ci-help
