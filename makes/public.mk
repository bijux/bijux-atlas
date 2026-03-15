# Scope: public include surface for the repository makes entrypoint.
# Public targets: makes-target-list
include makes/vars.mk
include makes/paths.mk
include makes/macros.mk
include makes/checks.mk
include makes/contracts.mk

makes-target-list: ## Regenerate the makes public target list artifact
	@$(DEV_ATLAS) makes target-list --allow-write --format $(FORMAT) >/dev/null
	@printf '%s\n' "wrote: makes/target-list.json"

.PHONY: makes-target-list make-contract-check
