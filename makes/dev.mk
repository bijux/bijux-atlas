# Scope: canonical developer wrappers and invocation SSOT for the Rust control plane.
# Public targets: dev-doctor, dev-check-ci, install-local
SHELL := /bin/sh

BIJUX ?= bijux
DEV_ATLAS ?= cargo run -q -p bijux-dev-atlas --

# Compatibility alias during makefile cutover; wrappers should use DEV_ATLAS directly.
BIJUX_DEV_ATLAS ?= $(DEV_ATLAS)

export BIJUX DEV_ATLAS BIJUX_DEV_ATLAS

dev-doctor: ## Run dev control-plane doctor suite
	@$(DEV_ATLAS) registry doctor --format $(FORMAT)

dev-check-ci: ## Run dev control-plane ci suite
	@$(DEV_ATLAS) check run --suite ci_fast --include-internal --include-slow --format $(FORMAT)

install-local: ## Build and install bijux-atlas + bijux-dev-atlas into artifacts/bin
	@$(DEV_ATLAS) build install-local --allow-subprocess --allow-write --format $(FORMAT)

.PHONY: dev-doctor dev-check-ci install-local
