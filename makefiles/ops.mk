# Scope: ops area wrappers only.
# Public targets: ops, ops-up, ops-down, ops-clean, ops-pins-check, ops-pins-update
SHELL := /bin/sh

define run_dev_atlas
	@if command -v bijux >/dev/null 2>&1; then \
		bijux dev atlas $(1); \
	else \
		cargo run -p bijux-dev-atlas -- $(1); \
	fi
endef

ops: ## Canonical ops gate
	@$(call run_dev_atlas,run --suite ops_fast --format text --durations 10)

ops-up: ## Bring up full local ops environment
	@ATLAS_E2E_NAMESPACE=atlas-e2e ATLAS_NS=atlas-e2e $(call run_dev_atlas,ops up)

ops-down: ## Tear down full local ops environment
	@ATLAS_E2E_NAMESPACE=atlas-e2e ATLAS_NS=atlas-e2e $(call run_dev_atlas,ops down)

ops-clean: ## Clean generated ops outputs
	@$(call run_dev_atlas,ops clean)

ops-pins-check: ## Validate unified reproducibility pins
	@$(call run_dev_atlas,run --id 'ops_*pins*' --format text --durations 10)

ops-pins-update: ## Refresh unified reproducibility pins
	@$(call run_dev_atlas,ops pins update --i-know-what-im-doing)

.PHONY: ops ops-up ops-down ops-clean ops-pins-check ops-pins-update
