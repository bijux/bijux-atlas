# Scope: path contract checks and make safety internal targets.
# Public targets: none
SHELL := /bin/sh

CANONICAL_PATHS := artifacts crates docs makefiles ops configs docker .github .cargo

path-contract-check: ## Validate canonical repository path contract and forbidden raw paths
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/checks/layout/root/check_forbidden_paths.py
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_safety.py

.PHONY: path-contract-check
