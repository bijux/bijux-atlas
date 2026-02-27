help-contract: ## Show make contract and public target documentation pointers
	@printf '%s\n' "contract: $(MAKE_CONTRACT_PATH)" "public-targets: $(MAKE_HELP_PATH)"

.PHONY: help-contract
