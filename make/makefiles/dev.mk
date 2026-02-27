# Scope: canonical developer wrappers and invocation SSOT for the Rust control plane.
# Keep one target per gate with deterministic control-plane commands.
SHELL := /bin/sh

BIJUX ?= bijux
DEV_ATLAS ?= cargo run -q -p bijux-dev-atlas --

# Compatibility alias during makefile cutover; wrappers should use DEV_ATLAS directly.
BIJUX_DEV_ATLAS ?= $(DEV_ATLAS)

export BIJUX DEV_ATLAS BIJUX_DEV_ATLAS

dev-doctor: ## Run dev control-plane doctor suite
	@$(DEV_ATLAS) check doctor --format text

dev-check-ci: ## Run dev control-plane ci suite
	@$(DEV_ATLAS) check run --suite ci --format text

dev-ci: ## Alias for dev-check-ci
	@$(MAKE) -s dev-check-ci

install-local: ## Build and install bijux-atlas + bijux-dev-atlas into artifacts/bin
	@mkdir -p artifacts/bin
	@cargo build -p bijux-atlas-cli -p bijux-dev-atlas
	@cp artifacts/target/debug/bijux-atlas artifacts/bin/bijux-atlas
	@cp artifacts/target/debug/bijux-dev-atlas artifacts/bin/bijux-dev-atlas
	@chmod +x artifacts/bin/bijux-atlas artifacts/bin/bijux-dev-atlas
	@echo "installed artifacts/bin/bijux-atlas"
	@echo "installed artifacts/bin/bijux-dev-atlas"

.PHONY: dev-doctor dev-check-ci dev-ci install-local
