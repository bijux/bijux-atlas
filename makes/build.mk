# Scope: thin build wrappers delegating to the Rust control plane.
# Public targets: build, build-release, build-ci, build-meta, dist, dist-verify

build: ## Build required binaries into artifacts/dist/bin
	@mkdir -p "$(ARTIFACT_ROOT)/build/$(RUN_ID)" "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)" && printf '%s\n' "run: $(DEV_ATLAS) build bin --allow-subprocess --allow-write --format $(FORMAT)" && $(DEV_ATLAS) build bin --allow-subprocess --allow-write --format $(FORMAT) --out $(ARTIFACT_ROOT)/build/$(RUN_ID)/report.json >/dev/null

build-release: ## Build release bundle inputs (delegates to build bin contract)
	@mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@$(DEV_ATLAS) build bin --allow-subprocess --allow-write --format $(FORMAT)

build-ci: ## Build binaries with CI-oriented environment defaults
	@mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@CARGO_INCREMENTAL=0 $(DEV_ATLAS) build bin --allow-subprocess --allow-write --format $(FORMAT)

build-meta: ## Emit build metadata under artifacts/dist/build.json
	@mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@$(DEV_ATLAS) build meta --allow-write --format $(FORMAT)

dist: ## Build release bundles into artifacts/dist/release
	@mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@$(DEV_ATLAS) build dist --allow-subprocess --allow-write --format $(FORMAT)

dist-verify: ## Verify built binaries under artifacts/dist/bin
	@mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@$(DEV_ATLAS) build verify --allow-subprocess --format $(FORMAT)

build-sdist: ## Forbidden: source dist archives are not part of the Rust control-plane contract
	@printf '%s\n' 'build-sdist is forbidden; use make dist for release bundles under artifacts/dist/release' >&2; exit 2

clean-build: ## Clean build-scoped artifact directories
	@mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@$(DEV_ATLAS) build clean --allow-write --format $(FORMAT)

clean-dist: ## Clean dist outputs only (and build-scoped artifacts via build clean)
	@mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@$(DEV_ATLAS) build clean --allow-write --format $(FORMAT)

build-doctor: ## Validate build toolchain and build output contracts
	@mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@$(DEV_ATLAS) build doctor --format $(FORMAT)

.PHONY: build build-release build-ci build-meta dist dist-verify build-sdist clean-build clean-dist build-doctor
