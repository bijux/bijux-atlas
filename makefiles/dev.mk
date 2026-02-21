# Scope: canonical developer + CI wrappers delegated to stable atlasctl DEV entrypoints.
# Public targets: none
SHELL := /bin/sh

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

internal/dev/fmt:
	@./bin/atlasctl dev fmt

internal/dev/lint:
	@./bin/atlasctl dev lint

internal/dev/check:
	@./bin/atlasctl dev check

internal/dev/test:
	@./bin/atlasctl dev test

internal/dev/test-all:
	@./bin/atlasctl dev test --all

internal/dev/audit:
	@./bin/atlasctl dev audit

internal/dev/coverage:
	@./bin/atlasctl dev coverage

internal/dev/ci:
	@./bin/atlasctl dev ci run --json

test-all:
	@./bin/atlasctl dev test --all

test-contracts:
	@./bin/atlasctl dev test --contracts

ci-core:
	@./bin/atlasctl dev ci run --json

ci-fast:
	@./bin/atlasctl dev ci fast

ci-contracts:
	@./bin/atlasctl dev ci contracts

ci-docs:
	@./bin/atlasctl dev ci docs

ci-ops:
	@./bin/atlasctl dev ci ops

ci-init-iso-dirs:
	@./bin/atlasctl dev ci init-iso-dirs

ci-init-tmp:
	@./bin/atlasctl dev ci init-tmp

ci-dependency-lock-refresh:
	@./bin/atlasctl dev ci dependency-lock-refresh

ci-release-compat-matrix-verify:
	@./bin/atlasctl dev ci release-compat-matrix-verify

ci-release-build-artifacts:
	@./bin/atlasctl dev ci release-build-artifacts

ci-release-notes-render:
	@./bin/atlasctl dev ci release-notes-render

ci-release-publish-gh:
	@./bin/atlasctl dev ci release-publish-gh

ci-cosign-sign:
	@./bin/atlasctl dev ci cosign-sign

ci-cosign-verify:
	@./bin/atlasctl dev ci cosign-verify

ci-chart-package-release:
	@./bin/atlasctl dev ci chart-package-release

ci-reproducible-verify:
	@./bin/atlasctl dev ci reproducible-verify

ci-security-advisory-render:
	@./bin/atlasctl dev ci security-advisory-render

governance-check:
	@./bin/atlasctl dev ci governance-check

ci-workflows-make-only:
	@./bin/atlasctl check forbidden-paths

.PHONY: dev-fmt dev-lint dev-test dev-coverage ci internal/dev/fmt internal/dev/lint internal/dev/check internal/dev/test internal/dev/test-all internal/dev/audit internal/dev/coverage internal/dev/ci test-all test-contracts ci-core ci-fast ci-contracts ci-docs ci-ops ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify ci-security-advisory-render governance-check ci-workflows-make-only
