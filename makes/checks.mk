# Scope: makes governance support wrappers delegated to the control plane.
# Public targets: make-contract-check
make-contract-check: ## Run the governed make-required check suite
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format $(FORMAT)"
	@$(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format $(FORMAT)
