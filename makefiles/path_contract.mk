SHELL := /bin/sh

CANONICAL_PATHS := artifacts crates docs makefiles ops scripts configs docker xtask .github .cargo

path-contract-check: ## Validate canonical repository path contract and forbidden raw paths
	@./scripts/layout/check_no_forbidden_paths.sh
	@python3 ./scripts/layout/check_make_safety.py

.PHONY: path-contract-check
