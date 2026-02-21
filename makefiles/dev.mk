# Scope: canonical developer + CI wrappers delegated to stable atlasctl entrypoints.
# Public targets are surfaced via root help/catalog.
SHELL := /bin/sh

fmt: ## Rust formatter check
	@./bin/atlasctl dev fmt

fmt-all: ## Rust formatter full variant
	@./bin/atlasctl dev fmt --all

lint: ## Rust lint/policy lane
	@./bin/atlasctl dev lint

lint-all: ## Rust lint full variant
	@./bin/atlasctl dev lint --all

test: ## Rust tests lane
	@./bin/atlasctl dev test

test-all: ## Rust tests full variant (includes ignored)
	@./bin/atlasctl dev test --all

test-contracts: ## Rust contract-focused tests
	@./bin/atlasctl dev test --contracts

audit: ## Rust security/audit lane
	@./bin/atlasctl dev audit

audit-all: ## Rust audit full variant
	@./bin/atlasctl dev audit --all

coverage: ## Rust coverage lane
	@./bin/atlasctl dev coverage

coverage-all: ## Rust coverage full variant
	@./bin/atlasctl dev coverage --all

check-all: ## Rust check full variant
	@./bin/atlasctl dev check --all

ci: ## Canonical CI entrypoint
	@./bin/atlasctl ci run --json --out-dir artifacts/reports/atlasctl/suite-ci

internal/dev/check: ## Internal DEV wrapper: check
	@./bin/atlasctl dev check

internal/ci/run: ## Internal CI wrapper: canonical run
	@./bin/atlasctl ci run --json

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

.PHONY: fmt fmt-all lint lint-all test test-all test-contracts audit audit-all coverage coverage-all check-all ci internal/dev/check internal/ci/run ci-fast ci-contracts ci-docs ci-ops ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify ci-security-advisory-render governance-check ci-workflows-make-only
