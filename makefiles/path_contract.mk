# Scope: path contract checks and make safety internal targets.
# Public targets: none
SHELL := /bin/sh

CANONICAL_PATHS := artifacts crates docs makefiles ops scripts configs docker .github .cargo

path-contract-check: ## Validate canonical repository path contract and forbidden raw paths
	@./packages/atlasctl/src/atlasctl/checks/layout/check_no_forbidden_paths.sh
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/check_make_safety.py

.PHONY: path-contract-check
