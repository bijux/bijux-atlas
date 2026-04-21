# Scope: release workflow adapter for artifact bundling.
# Public targets: install, build

install: ## Prepare Rust dependencies for release bundle builds
	@cargo fetch --locked

build: ## Build release bundle artifacts into artifacts/dist/release
	@$(MAKE) -f makes/root.mk -C . dist

.PHONY: install build
