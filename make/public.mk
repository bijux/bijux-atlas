# Scope: public include surface for the repository make entrypoint.
# Public targets: make-target-list
include make/vars.mk
include make/paths.mk
include make/macros.mk
include make/_internal.mk
include make/checks.mk
include make/contracts.mk

make-target-list: ## Regenerate make public target list artifact
	@$(DEV_ATLAS) make target-list --allow-write --format $(FORMAT) >/dev/null
	@printf '%s\n' "wrote: make/target-list.json"

.PHONY: make-target-list make-contract-check
