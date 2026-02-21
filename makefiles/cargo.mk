# Scope: cargo wrappers delegated to atlasctl DEV/CI entrypoints.
# Public targets: none
SHELL := /bin/sh

fmt:
	@./bin/atlasctl dev fmt

lint:
	@./bin/atlasctl dev lint

check:
	@./bin/atlasctl dev check

test:
	@./bin/atlasctl dev test

test-all:
	@./bin/atlasctl dev test --all

test-contracts:
	@./bin/atlasctl dev test --contracts

coverage:
	@./bin/atlasctl dev coverage

audit:
	@./bin/atlasctl dev audit

internal/cargo/fmt:
	@./bin/atlasctl dev fmt

internal/cargo/lint:
	@./bin/atlasctl dev lint

internal/cargo/check:
	@./bin/atlasctl dev check

internal/cargo/test:
	@./bin/atlasctl dev test

internal/cargo/audit:
	@./bin/atlasctl dev audit

_fmt:
	@./bin/atlasctl dev fmt

_lint:
	@./bin/atlasctl dev lint

_check:
	@./bin/atlasctl dev check

_test:
	@./bin/atlasctl dev test

_test-all:
	@./bin/atlasctl dev test --all

_test-contracts:
	@./bin/atlasctl dev test --contracts

_coverage:
	@./bin/atlasctl dev coverage

_audit:
	@./bin/atlasctl dev audit

ci-core:
	@./bin/atlasctl dev ci run --json

openapi-drift:
	@./bin/atlasctl contracts check --checks drift

api-contract-check:
	@./bin/atlasctl dev ci contracts

compat-matrix-validate:
	@./bin/atlasctl compat validate-matrix

query-plan-gate:
	@./bin/atlasctl dev ci contracts

critical-query-check:
	@./bin/atlasctl dev ci contracts

bench-smoke:
	@./bin/atlasctl dev test

.PHONY: fmt lint check test test-all test-contracts coverage audit internal/cargo/fmt internal/cargo/lint internal/cargo/check internal/cargo/test internal/cargo/audit _fmt _lint _check _test _test-all _test-contracts _coverage _audit ci-core openapi-drift api-contract-check compat-matrix-validate query-plan-gate critical-query-check bench-smoke
