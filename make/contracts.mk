CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/contracts/$(RUN_ID)
CONTRACTS_DEV_ATLAS_TARGET_DIR ?= $(WORKSPACE_ROOT)/artifacts/target
CONTRACTS_DEV_ATLAS_BIN ?= $(CONTRACTS_DEV_ATLAS_TARGET_DIR)/debug/bijux-dev-atlas

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

contracts: _contracts_guard ## Run all contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-pr: _contracts_guard ## Run required and static contracts for pull requests
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane pr --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane pr --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-merge: _contracts_guard ## Run required and effect contracts for merge gating
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane merge --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane merge --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-release: _contracts_guard ## Run full release contracts matrix
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane release --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane release --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-all: _contracts_guard ## Run all contracts with effect lane (no static mode skips)
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --mode effect --profile ci --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --mode effect --profile ci --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-fast: _contracts_guard ## Run static-only contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-changed: _contracts_guard ## Run changed-only contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --changed-only --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --changed-only --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-json: _contracts_guard ## Run all contracts and emit json
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-ci: _contracts_guard ## Run strict CI contracts lane
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --mode effect --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --mode effect --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-root: _contracts_guard ## Run root contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts root --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts root --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-configs: _contracts_guard ## Run configs contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts configs --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts configs --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-docs: _contracts_guard ## Run docs contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts docs --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts docs --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-docker: _contracts_guard ## Run docker contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts docker --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts docker --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-make: _contracts_guard ## Run make contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts make --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts make --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-ops: _contracts_guard ## Run ops contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts ops --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts ops --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

.PHONY: _contracts_guard contracts-help contracts contracts-pr contracts-merge contracts-release contracts-all contracts-fast contracts-changed contracts-json contracts-ci contracts-root contracts-configs contracts-docs contracts-docker contracts-make contracts-ops
