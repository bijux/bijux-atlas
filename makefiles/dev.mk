# Scope: canonical developer + CI wrappers delegated to stable atlasctl DEV entrypoints.
# Public targets: none
SHELL := /bin/sh

fmt: ## Rust formatter check (same as `atlasctl dev fmt`)
	@./bin/atlasctl dev fmt

lint: ## Rust lint/policy lane (same as `atlasctl dev lint`)
	@./bin/atlasctl dev lint

test: ## Rust tests lane (same as `atlasctl dev test`)
	@./bin/atlasctl dev test

audit: ## Rust security/audit lane (same as `atlasctl dev audit`)
	@./bin/atlasctl dev audit

coverage: ## Rust coverage lane (same as `atlasctl dev coverage`)
	@./bin/atlasctl dev coverage

dev-fmt: ## Stable DEV wrapper: formatter lane via atlasctl
	@./bin/atlasctl dev fmt

dev-lint: ## Stable DEV wrapper: lint lane via atlasctl
	@./bin/atlasctl dev lint

dev-test: ## Stable DEV wrapper: test lane via atlasctl
	@./bin/atlasctl dev test

dev-coverage: ## Stable DEV wrapper: coverage lane via atlasctl
	@./bin/atlasctl dev coverage

ci: ## CI entrypoint mirror
	@./bin/atlasctl dev ci run --json --out-dir artifacts/reports/atlasctl/suite-ci

internal/dev/fmt: ## Internal DEV wrapper: fmt
	@./bin/atlasctl dev fmt

internal/dev/lint: ## Internal DEV wrapper: lint
	@./bin/atlasctl dev lint

internal/dev/check: ## Internal DEV wrapper: check
	@./bin/atlasctl dev check

internal/dev/test: ## Internal DEV wrapper: test
	@./bin/atlasctl dev test

internal/dev/test-all: ## Internal DEV wrapper: test all (ignored included)
	@./bin/atlasctl dev test --all

internal/dev/audit: ## Internal DEV wrapper: audit
	@./bin/atlasctl dev audit

internal/dev/coverage: ## Internal DEV wrapper: coverage
	@./bin/atlasctl dev coverage

internal/dev/ci: ## Internal DEV wrapper: canonical ci run
	@./bin/atlasctl dev ci run --json

test-all: ## Run tests including ignored (`atlasctl dev test --all`)
	@./bin/atlasctl dev test --all

test-contracts: ## Run contracts-only test slice (`atlasctl dev test --contracts`)
	@./bin/atlasctl dev test --contracts

ci-core: ## Canonical CI run wrapper (`atlasctl dev ci run --json`)
	@./bin/atlasctl dev ci run --json

ci-fast: ## CI fast lane wrapper
	@./bin/atlasctl dev ci fast

ci-contracts: ## CI contracts lane wrapper
	@./bin/atlasctl dev ci contracts

ci-docs: ## CI docs lane wrapper
	@./bin/atlasctl dev ci docs

ci-ops: ## CI ops lane wrapper
	@./bin/atlasctl dev ci ops

ci-init-iso-dirs: ## CI helper: initialize isolate directories
	@./bin/atlasctl dev ci init-iso-dirs

ci-init-tmp: ## CI helper: initialize temp directories
	@./bin/atlasctl dev ci init-tmp

ci-dependency-lock-refresh: ## CI helper: refresh dependency lock artifacts
	@./bin/atlasctl dev ci dependency-lock-refresh

ci-release-compat-matrix-verify: ## CI release lane: compatibility matrix verification
	@./bin/atlasctl dev ci release-compat-matrix-verify

ci-release-build-artifacts: ## CI release lane: build artifacts
	@./bin/atlasctl dev ci release-build-artifacts

ci-release-notes-render: ## CI release lane: render release notes
	@./bin/atlasctl dev ci release-notes-render

ci-release-publish-gh: ## CI release lane: publish GitHub release
	@./bin/atlasctl dev ci release-publish-gh

ci-cosign-sign: ## CI release lane: sign artifacts
	@./bin/atlasctl dev ci cosign-sign

ci-cosign-verify: ## CI release lane: verify signatures
	@./bin/atlasctl dev ci cosign-verify

ci-chart-package-release: ## CI release lane: package chart
	@./bin/atlasctl dev ci chart-package-release

ci-reproducible-verify: ## CI release lane: reproducibility verification
	@./bin/atlasctl dev ci reproducible-verify

ci-security-advisory-render: ## CI release lane: render security advisory
	@./bin/atlasctl dev ci security-advisory-render

governance-check: ## CI governance checks wrapper
	@./bin/atlasctl dev ci governance-check

ci-workflows-make-only: ## Guardrail: enforce workflow make/atlasctl entrypoints
	@./bin/atlasctl check forbidden-paths

.PHONY: fmt lint test audit coverage dev-fmt dev-lint dev-test dev-coverage ci internal/dev/fmt internal/dev/lint internal/dev/check internal/dev/test internal/dev/test-all internal/dev/audit internal/dev/coverage internal/dev/ci test-all test-contracts ci-core ci-fast ci-contracts ci-docs ci-ops ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify ci-security-advisory-render governance-check ci-workflows-make-only
