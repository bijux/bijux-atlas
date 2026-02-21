# Scope: path contract checks and make safety internal targets.
# Public targets: none
SHELL := /bin/sh

CANONICAL_PATHS := artifacts crates docs makefiles ops scripts configs docker .github .cargo

path-contract-check: ## Validate canonical repository path contract and forbidden raw paths
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/root/check_forbidden_paths.py
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_safety.py

.PHONY: path-contract-check
