# Scope: canonical developer wrappers delegated to stable atlasctl entrypoints.
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

audit: ## Rust security/audit lane
	@./bin/atlasctl dev audit

audit-all: ## Rust audit full variant
	@./bin/atlasctl dev audit --all

coverage: ## Rust coverage lane
	@./bin/atlasctl dev coverage

coverage-all: ## Rust coverage full variant
	@./bin/atlasctl dev coverage --all

check: ## Rust check lane
	@./bin/atlasctl dev check

check-all: ## Rust check full variant
	@./bin/atlasctl dev check --all

all: ## Default dev gate set (fmt+lint+test)
	@./bin/atlasctl dev fmt
	@./bin/atlasctl dev lint
	@./bin/atlasctl dev test

all-all: ## Full dev gate set (*-all variants)
	@./bin/atlasctl dev fmt --all
	@./bin/atlasctl dev lint --all
	@./bin/atlasctl dev test --all
	@./bin/atlasctl dev audit --all
	@./bin/atlasctl dev check --all

internal/dev/check: ## Internal DEV wrapper: check
	@./bin/atlasctl dev check

.PHONY: fmt fmt-all lint lint-all test test-all audit audit-all coverage coverage-all check check-all all all-all internal/dev/check
