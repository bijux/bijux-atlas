include make/vars.mk
include make/paths.mk
include make/macros.mk
include make/_internal.mk
include make/checks.mk
include make/contracts.mk

help-contract: ## Show make contract and public target documentation pointers
	@printf '%s\n' "contract: $(MAKE_CONTRACT_PATH)" "readme: $(MAKE_HELP_PATH)" "target-list: make/target-list.json"

make-target-list: ## Regenerate make public target list artifact
	@$(DEV_ATLAS) make target-list --allow-write --format text

.PHONY: help-contract make-target-list make-contract-check
