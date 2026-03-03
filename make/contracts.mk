# Scope: contracts wrapper targets delegated to bijux-dev-atlas suites and contract runners.
# Public targets: contracts, contracts-pr, contracts-merge, contracts-release, contracts-all, contracts-fast, contracts-changed, contracts-json, contracts-ci, contracts-root, contracts-repo, contracts-crates, contracts-runtime, contracts-configs, contracts-configs-required, contracts-docs, contracts-docs-required, contracts-docker, contracts-make, contracts-make-required, contracts-ops, contracts-help
CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/contracts/$(RUN_ID)
CONTRACTS_DEV_ATLAS_TARGET_DIR ?= $(WORKSPACE_ROOT)/artifacts/target
CONTRACTS_DEV_ATLAS_BIN ?= $(CONTRACTS_DEV_ATLAS_TARGET_DIR)/debug/bijux-dev-atlas
CONTRACTS_EFFECT_FLAGS := --mode effect --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon

_contracts_guard:
	@if [ ! -x "$(CONTRACTS_DEV_ATLAS_BIN)" ]; then \
		printf '%s\n' "build: cargo build -p bijux-dev-atlas"; \
		CARGO_TARGET_DIR="$(CONTRACTS_DEV_ATLAS_TARGET_DIR)" cargo build -q -p bijux-dev-atlas; \
	fi
	@command -v "$(CONTRACTS_DEV_ATLAS_BIN)" >/dev/null 2>&1 || { \
		printf '%s\n' "missing $(CONTRACTS_DEV_ATLAS_BIN); run: cargo build -p bijux-dev-atlas"; \
		exit 2; \
	}

contracts-help: ## Show contracts gate targets
	@$(MAKE) -s help-contract

contracts: _contracts_guard ## Run the fast static contract lane
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-pr: _contracts_guard ## Run required and static contracts for pull requests
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane pr --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane pr --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-merge: _contracts_guard ## Run required and effect contracts for merge gating
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane merge --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane merge --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-release: _contracts_guard ## Run full release contracts matrix
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane release --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane release --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-all: _contracts_guard ## Run the full contract suite without static skips
	@$(DEV_ATLAS) suites run --suite contracts --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format json

contracts-group: _contracts_guard ## Run one contracts suite group (GROUP=<name>)
	@[ -n "$${GROUP:-}" ] || { echo "usage: make contracts-group GROUP=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) suites run --suite contracts --group "$${GROUP}" --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format json

contracts-tag: _contracts_guard ## Run contracts suite entries with a shared tag (TAG=<name>)
	@[ -n "$${TAG:-}" ] || { echo "usage: make contracts-tag TAG=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) suites run --suite contracts --tag "$${TAG}" --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format json

contracts-pure: _contracts_guard ## Run only pure contracts suite entries
	@$(DEV_ATLAS) suites run --suite contracts --mode pure --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format json

contracts-effect: _contracts_guard ## Run only effectful contracts suite entries
	@$(DEV_ATLAS) suites run --suite contracts --mode effect --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format json

contracts-fast: _contracts_guard ## Run static-only contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-changed: _contracts_guard ## Run changed-only contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --changed-only --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --changed-only --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-json: _contracts_guard ## Run all contracts and emit json
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-ci: _contracts_guard ## Run strict CI contracts lane
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all $(CONTRACTS_EFFECT_FLAGS) --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all $(CONTRACTS_EFFECT_FLAGS) --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-root: _contracts_guard ## Run root contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts root --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts root --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-repo: _contracts_guard ## Run repository contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts repo --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts repo --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-crates: _contracts_guard ## Run crate contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts crates --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts crates --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-runtime: _contracts_guard ## Run runtime contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts runtime --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts runtime --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-configs: _contracts_guard ## Run configs contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts configs --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts configs --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-configs-required: _contracts_guard ## Run PR-required configs suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite configs_required --include-internal --include-slow --format json"
	@$(DEV_ATLAS) check run --suite configs_required --include-internal --include-slow --format json

contracts-docs: _contracts_guard ## Run docs contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts docs --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts docs --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-docs-required: _contracts_guard ## Run PR-required docs suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite docs_required --include-internal --include-slow --format json"
	@$(DEV_ATLAS) check run --suite docs_required --include-internal --include-slow --format json

contracts-docker: _contracts_guard ## Run docker contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts docker --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts docker --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-make: _contracts_guard ## Run make contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts make --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts make --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-make-required: _contracts_guard ## Run PR-required make suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format json"
	@$(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format json

contracts-ops: _contracts_guard ## Run ops contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts ops --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts ops --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

.PHONY: _contracts_guard contracts-help contracts contracts-pr contracts-merge contracts-release contracts-all contracts-changed contracts-ci contracts-configs contracts-crates contracts-docker contracts-docs contracts-effect contracts-fast contracts-group contracts-json contracts-make contracts-make-required contracts-merge contracts-ops contracts-pr contracts-pure contracts-release contracts-repo contracts-root contracts-runtime contracts-tag
