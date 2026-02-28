CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/contracts/$(RUN_ID)

_contracts_guard:
	@command -v $(DEV_ATLAS) >/dev/null 2>&1 || { \
		printf '%s\n' "missing $(DEV_ATLAS); run: cargo build -p bijux-dev-atlas"; \
		exit 2; \
	}

contracts-help: ## Show contracts gate targets
	@$(MAKE) -s help-contract

contracts: _contracts_guard ## Run all contracts
	@$(DEV_ATLAS) contracts all --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-fast: _contracts_guard ## Run static-only contracts
	@$(DEV_ATLAS) contracts all --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-changed: _contracts_guard ## Run changed-only contracts
	@$(DEV_ATLAS) contracts all --mode static --changed-only --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-json: _contracts_guard ## Run all contracts and emit json
	@$(DEV_ATLAS) contracts all --mode static --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-ci: _contracts_guard ## Run strict CI contracts lane
	@CI=1 $(DEV_ATLAS) contracts all --mode effect --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-root: _contracts_guard ## Run root contracts
	@$(DEV_ATLAS) contracts root --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-configs: _contracts_guard ## Run configs contracts
	@$(DEV_ATLAS) contracts configs --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-docs: _contracts_guard ## Run docs contracts
	@$(DEV_ATLAS) contracts docs --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-docker: _contracts_guard ## Run docker contracts
	@$(DEV_ATLAS) contracts docker --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-make: _contracts_guard ## Run make contracts
	@$(DEV_ATLAS) contracts make --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-ops: _contracts_guard ## Run ops contracts
	@$(DEV_ATLAS) contracts ops --mode static --format human --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

.PHONY: _contracts_guard contracts-help contracts contracts-fast contracts-changed contracts-json contracts-ci contracts-root contracts-configs contracts-docs contracts-docker contracts-make contracts-ops
