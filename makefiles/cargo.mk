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

.PHONY: fmt lint check test test-all test-contracts coverage audit ci-core openapi-drift api-contract-check compat-matrix-validate query-plan-gate critical-query-check bench-smoke
