# Scope: canonical developer wrappers delegated to stable atlasctl entrypoints.
# Public targets are surfaced via root help/catalog.
SHELL := /bin/sh

fmt: ## Rust formatter check
	@./bin/atlasctl dev fmt

fmt-all: ## Rust formatter full variant
	@./bin/atlasctl dev fmt --all

fmt-and-slow: ## Rust formatter full variant including slow repo checks
	@./bin/atlasctl dev fmt --all --and-slow

lint: ## Rust lint/policy lane
	@./bin/atlasctl dev lint

lint-all: ## Rust lint full variant
	@./bin/atlasctl dev lint --all

lint-and-slow: ## Rust lint full variant including slow repo checks
	@./bin/atlasctl dev lint --all --and-slow

test: ## Rust tests lane
	@./bin/atlasctl dev test

test-all: ## Rust tests full variant (includes ignored)
	@./bin/atlasctl dev test --all

test-and-slow: ## Rust tests full variant including slow repo checks
	@./bin/atlasctl dev test --all --and-slow

audit: ## Rust security/audit lane
	@./bin/atlasctl dev audit

audit-all: ## Rust audit full variant
	@./bin/atlasctl dev audit --all

audit-and-slow: ## Rust audit full variant including slow repo checks
	@./bin/atlasctl dev audit --all --and-slow

coverage: ## Rust coverage lane
	@./bin/atlasctl dev coverage

coverage-all: ## Rust coverage full variant
	@./bin/atlasctl dev coverage --all

coverage-and-slow: ## Rust coverage full variant including slow repo checks
	@./bin/atlasctl dev coverage --all --and-slow

check: ## Rust check lane
	@./bin/atlasctl dev check

check-all: ## Rust check full variant
	@./bin/atlasctl dev check --all

check-and-slow: ## Rust check full variant including slow repo checks
	@./bin/atlasctl dev check --all --and-slow

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

all-and-slow: ## Full dev gate set including slow repo checks
	@./bin/atlasctl dev fmt --all --and-slow
	@./bin/atlasctl dev lint --all --and-slow
	@./bin/atlasctl dev test --all --and-slow
	@./bin/atlasctl dev audit --all --and-slow
	@./bin/atlasctl dev check --all --and-slow

internal/dev/check: ## Internal DEV wrapper: check
	@./bin/atlasctl dev check

.PHONY: fmt fmt-all fmt-and-slow lint lint-all lint-and-slow test test-all test-and-slow audit audit-all audit-and-slow coverage coverage-all coverage-and-slow check check-all check-and-slow all all-all all-and-slow internal/dev/check
