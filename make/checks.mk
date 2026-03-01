# Scope: compatibility check wrappers delegated to the Rust control plane.
# Public targets: make-contract-check, make-target-governance-check
make-contract-check: ## Enforce make contract constraints through the Rust contracts runner
	@printf '%s\n' "run: $(DEV_ATLAS) contracts make --mode static --format text"
	@$(DEV_ATLAS) contracts make --mode static --format text

make-target-governance-check: ## Compatibility wrapper for make target governance contracts
	@$(DEV_ATLAS) contracts make --mode static --format text \
		--only MAKE-SURFACE-001 \
		--only MAKE-SURFACE-002 \
		--only MAKE-SURFACE-003 \
		--only MAKE-SURFACE-005 \
		--only MAKE-INTERNAL-001 \
		--only MAKE-DRIFT-001

make-ci-surface-check: ## Compatibility wrapper for workflow surface contracts
	@$(DEV_ATLAS) contracts make --mode static --format text --only MAKE-CI-001

make-public-surface-sync-check: ## Compatibility wrapper for curated target registry sync
	@$(DEV_ATLAS) contracts make --mode static --format text \
		--only MAKE-SURFACE-003 \
		--only MAKE-DRIFT-001

make-size-budget-check: ## Compatibility wrapper for bounded make structure contracts
	@$(DEV_ATLAS) contracts make --mode static --format text \
		--only MAKE-DIR-003 \
		--only MAKE-STRUCT-002

make-include-cycle-check: ## Compatibility wrapper for include graph contracts
	@$(DEV_ATLAS) contracts make --mode static --format text --only MAKE-INCLUDE-003
