# Scope: thin build wrappers delegating to the Rust control plane.

build: ## Build required binaries into artifacts/bin
	@$(DEV_ATLAS) build bin --allow-subprocess --allow-write --format json

dist: ## Build release bundles into artifacts/dist
	@$(DEV_ATLAS) build dist --allow-subprocess --allow-write --format json

clean-build: ## Clean build-scoped artifact directories
	@$(DEV_ATLAS) build clean --allow-write --format json

build-doctor: ## Validate build toolchain and build output contracts
	@$(DEV_ATLAS) build doctor --format json

.PHONY: build dist clean-build build-doctor
