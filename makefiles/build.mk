# Scope: thin build wrappers delegating to the Rust control plane.

build: ## Build required binaries into artifacts/dist/bin
	@printf '%s\n' "run: $(DEV_ATLAS) build bin --allow-subprocess --allow-write --format json"
	@mkdir -p $(ARTIFACT_ROOT)/build/$(RUN_ID)
	@$(DEV_ATLAS) build bin --allow-subprocess --allow-write --format json | tee $(ARTIFACT_ROOT)/build/$(RUN_ID)/report.json >/dev/null

build-release: ## Build release bundle inputs (delegates to build bin contract)
	@$(DEV_ATLAS) build bin --allow-subprocess --allow-write --format json

build-ci: ## Build binaries with CI-oriented environment defaults
	@CARGO_INCREMENTAL=0 $(DEV_ATLAS) build bin --allow-subprocess --allow-write --format json

build-meta: ## Emit build metadata under artifacts/dist/build.json
	@$(DEV_ATLAS) build meta --allow-write --format json

dist: ## Build release bundles into artifacts/dist/release
	@$(DEV_ATLAS) build dist --allow-subprocess --allow-write --format json

dist-verify: ## Verify built binaries under artifacts/dist/bin
	@$(DEV_ATLAS) build verify --allow-subprocess --format json

build-sdist: ## Forbidden: source dist archives are not part of the Rust control-plane contract
	@printf '%s\n' 'build-sdist is forbidden; use make dist for release bundles under artifacts/dist/release' >&2; exit 2

clean-build: ## Clean build-scoped artifact directories
	@$(DEV_ATLAS) build clean --allow-write --format json

clean-dist: ## Clean dist outputs only (and build-scoped artifacts via build clean)
	@$(DEV_ATLAS) build clean --allow-write --format json

build-doctor: ## Validate build toolchain and build output contracts
	@$(DEV_ATLAS) build doctor --format json

.PHONY: build build-release build-ci build-meta dist dist-verify build-sdist clean-build clean-dist build-doctor
