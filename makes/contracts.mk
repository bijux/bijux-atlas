# Scope: contracts wrapper targets delegated to bijux-dev-atlas checks and suite runners.
# Public targets: contract, contract-effect, contract-all, contracts-pr, contracts-merge, contracts-release, contracts-configs-required, contracts-docs-required, contracts-make-required, contracts-help, contracts-group, contracts-tag, contracts-pure
CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/contracts/$(RUN_ID)
CONTRACT_FAIL_FAST_FLAG := $(if $(filter 1 true yes,$(FAIL_FAST)),--fail-fast,--no-fail-fast)

contracts-help: ## Show contracts gate targets
	@printf '%s\n' "docs: $(MAKE_README_PATH)" "target-list: makes/target-list.json"

contract: ## Run pure contract execution through the canonical suite runner
	@$(DEV_ATLAS) suites run --suite contracts --jobs $(JOBS) --mode pure $(CONTRACT_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contract-effect: ## Run effect contract execution through the canonical suite runner
	@$(DEV_ATLAS) suites run --suite contracts --jobs $(JOBS) --mode effect $(CONTRACT_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contract-all: ## Run the complete contract execution set through the canonical suite runner
	@$(DEV_ATLAS) suites run --suite contracts --jobs $(JOBS) --mode all $(CONTRACT_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contracts-pr: ## Run pure contracts for the pull-request lane
	@$(DEV_ATLAS) suites run --suite contracts --jobs $(JOBS) --mode pure $(CONTRACT_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contracts-merge: ## Run all contracts for merge gating
	@$(DEV_ATLAS) suites run --suite contracts --jobs $(JOBS) --mode all $(CONTRACT_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contracts-release: ## Run all contracts for release verification
	@$(DEV_ATLAS) suites run --suite contracts --jobs $(JOBS) --mode all $(CONTRACT_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contracts-group: ## Run one contracts suite group (GROUP=<name>)
	@[ -n "$${GROUP:-}" ] || { echo "usage: make contracts-group GROUP=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) suites run --suite contracts --group "$${GROUP}" --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contracts-tag: ## Run contracts suite entries with a shared tag (TAG=<name>)
	@[ -n "$${TAG:-}" ] || { echo "usage: make contracts-tag TAG=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) suites run --suite contracts --tag "$${TAG}" --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contracts-pure: ## Run only pure contracts suite entries
	@$(DEV_ATLAS) suites run --suite contracts --mode pure --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format $(CONTRACTS_FORMAT)

contracts-configs-required: ## Run PR-required configs suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) checks run --suite configs_required --include-internal --include-slow --format $(FORMAT)"
	@$(DEV_ATLAS) checks run --suite configs_required --include-internal --include-slow --format $(FORMAT)

contracts-docs-required: ## Run PR-required docs suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) checks run --suite docs_required --include-internal --include-slow --format $(FORMAT)"
	@$(DEV_ATLAS) checks run --suite docs_required --include-internal --include-slow --format $(FORMAT)

contracts-make-required: ## Run PR-required makes suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) checks run --suite make_required --include-internal --include-slow --format $(FORMAT)"
	@$(DEV_ATLAS) checks run --suite make_required --include-internal --include-slow --format $(FORMAT)

.PHONY: contract contract-effect contract-all contracts-help contracts-pr contracts-merge contracts-release contracts-configs-required contracts-docs-required contracts-make-required contracts-group contracts-tag contracts-pure
