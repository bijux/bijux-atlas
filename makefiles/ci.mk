# Scope: canonical CI wrappers delegated to stable atlasctl entrypoints.
# Public targets are surfaced via root help/catalog.
SHELL := /bin/sh

ci: ## Canonical CI entrypoint
	@./bin/atlasctl ci run --json --out-dir artifacts/reports/atlasctl/suite-ci

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

.PHONY: ci internal/ci/run ci-fast ci-contracts ci-docs ci-ops ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify ci-security-advisory-render governance-check ci-workflows-make-only
