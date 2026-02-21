# Scope: CI wrappers that delegate to stable atlasctl entrypoints only.
# Public targets: none
SHELL := /bin/sh

ci:
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

.PHONY: ci ci-fast ci-contracts ci-docs ci-ops ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify ci-security-advisory-render governance-check ci-workflows-make-only
