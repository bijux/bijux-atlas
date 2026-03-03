# Scope: compatibility check wrappers delegated to the Rust control plane.
# Public targets: make-contract-check, make-target-governance-check
make-contract-check: ## Enforce make contract constraints through the Rust contracts runner
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain make --format text"
	@$(DEV_ATLAS) contract run --mode static --domain make --format text

make-target-governance-check: ## Compatibility wrapper for make target governance contracts
	@$(DEV_ATLAS) contract run --mode static --domain make --format text \
		--include MAKE-SURFACE-001 \
		--include MAKE-SURFACE-002 \
		--include MAKE-SURFACE-003 \
		--include MAKE-SURFACE-005 \
		--include MAKE-INTERNAL-001 \
		--include MAKE-DRIFT-001

make-ci-surface-check: ## Compatibility wrapper for workflow surface contracts
	@$(DEV_ATLAS) contract run --mode static --domain make --format text --include MAKE-CI-001

make-public-surface-sync-check: ## Compatibility wrapper for curated target registry sync
	@$(DEV_ATLAS) contract run --mode static --domain make --format text \
		--include MAKE-SURFACE-003 \
		--include MAKE-DRIFT-001

make-size-budget-check: ## Compatibility wrapper for bounded make structure contracts
	@$(DEV_ATLAS) contract run --mode static --domain make --format text \
		--include MAKE-DIR-003 \
		--include MAKE-STRUCT-002

make-include-cycle-check: ## Compatibility wrapper for include graph contracts
	@$(DEV_ATLAS) contract run --mode static --domain make --format text --include MAKE-INCLUDE-003
