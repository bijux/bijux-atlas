# Scope: ops area wrappers only.
# Public targets: ops, ops-up, ops-down, ops-clean, ops-pins-check, ops-pins-update
SHELL := /bin/sh
# verification lane must stay non-destructive and deterministic.
VERIFICATION_TARGETS_ops := ops-clean

ops: ## Canonical ops gate
	@./bin/atlasctl ops check --report text

ops-up: ## Bring up full local ops environment
	@./bin/atlasctl ops up --report text

ops-down: ## Tear down full local ops environment
	@./bin/atlasctl ops down --report text

ops-clean: ## Clean generated ops outputs
	@./bin/atlasctl ops clean --report text

ops-pins-check: ## Validate unified reproducibility pins
	@./bin/atlasctl ops pins --report text check

ops-pins-update: ## Refresh unified reproducibility pins
	@./bin/atlasctl ops pins --report text update

.PHONY: ops ops-up ops-down ops-clean ops-pins-check ops-pins-update
