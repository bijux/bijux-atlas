# Scope: canonical CI wrappers delegated to stable atlasctl entrypoints.
# Public targets are surfaced via root help/catalog.
SHELL := /bin/sh

ci: ## Canonical CI entrypoint
	@./bin/atlasctl ci pr --json

ci-fast: ## CI fast lane wrapper
	@./bin/atlasctl ci run --json --lane fmt --lane lint --lane test --lane contracts

ci-nightly: ## CI nightly lane (includes slow checks)
	@./bin/atlasctl ci nightly --json

ci-docs: ## CI docs lane wrapper
	@./bin/atlasctl ci docs --json

ci-cosign-sign: ## CI release signing wrapper
	@./bin/atlasctl ci cosign-sign

ci-cosign-verify: ## CI release signature verification wrapper
	@./bin/atlasctl ci cosign-verify

ci-chart-package-release: ## CI release chart packaging wrapper
	@./bin/atlasctl ci chart-package-release

ci-init-tmp: ## CI init tmp/isolation dirs wrapper
	@./bin/atlasctl ci init-tmp

ci-dependency-lock-refresh: ## CI dependency lock refresh wrapper
	@./bin/atlasctl ci deps --json

ci-help: ## Show CI command help
	@./bin/atlasctl help ci

.PHONY: ci ci-fast ci-nightly ci-docs ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-init-tmp ci-dependency-lock-refresh ci-help
