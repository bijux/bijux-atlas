# Scope: canonical internal developer wrappers delegated to stable atlasctl DEV entrypoints.
# Public targets: none
SHELL := /bin/sh

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

.PHONY: internal/dev/fmt internal/dev/lint internal/dev/check internal/dev/test internal/dev/test-all internal/dev/audit internal/dev/coverage internal/dev/ci
