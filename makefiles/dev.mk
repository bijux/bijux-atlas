# Scope: canonical developer wrappers delegated to stable atlasctl entrypoints.
# Keep one target per gate; atlasctl-check is the explicit repo checks gate.
SHELL := /bin/sh

audit: ## Rust audit lane
	@./bin/atlasctl dev audit

check: ## Rust cargo check lane
	@./bin/atlasctl dev check

coverage: ## Rust coverage lane
	@./bin/atlasctl dev coverage

fmt: ## Rust formatter check
	@./bin/atlasctl dev fmt

lint: ## Rust lint lane
	@./bin/atlasctl dev lint

test: ## Rust tests lane
	@./bin/atlasctl dev test

test-all: ## Rust tests full variant (includes ignored)
	@./bin/atlasctl dev test --all

.PHONY: audit check coverage fmt lint test test-all
