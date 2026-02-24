# Scope: ops area wrappers only.
# Public targets: ops, ops-help, ops-doctor, ops-validate, ops-render, ops-install-plan, ops-up, ops-down, ops-clean, ops-reset, ops-status, ops-kind-up, ops-kind-down, ops-tools-verify, ops-pins-check, ops-pins-update
SHELL := /bin/sh
PROFILE ?= kind
OPS_RESET_RUN_ID ?= ops_reset

ops: ## Canonical ops gate
	@$(DEV_ATLAS) ops validate --profile $(PROFILE) --format text

ops-help: ## Show ops control-plane command surface
	@$(DEV_ATLAS) ops --help

ops-doctor: ## Run ops doctor checks
	@$(DEV_ATLAS) ops doctor --profile $(PROFILE) --format json

ops-validate: ## Run ops validation checks
	@$(DEV_ATLAS) ops validate --profile $(PROFILE) --format json

ops-render: ## Render ops manifests for selected profile
	@$(DEV_ATLAS) ops render --target kind --check --profile $(PROFILE) --format json

ops-install-plan: ## Print install plan without applying resources
	@$(DEV_ATLAS) ops install --kind --apply --plan --profile $(PROFILE) --allow-subprocess --allow-write --format json

ops-up: ## Bring up full local ops environment
	@if command -v kind >/dev/null 2>&1 && kind get clusters 2>/dev/null | grep -Eq '^(normal|bijux-atlas-e2e)$$'; then echo "ops-up: local kind cluster already exists"; else $(DEV_ATLAS) ops install --kind --apply --profile $(PROFILE) --allow-subprocess --allow-write --format text; fi

ops-down: ## Tear down full local ops environment
	@$(DEV_ATLAS) ops down --profile $(PROFILE) --allow-subprocess --format text

ops-clean: ## Clean generated ops outputs
	@$(DEV_ATLAS) ops clean --format text

ops-reset: ## Delete artifacts for RESET_RUN_ID
	@$(DEV_ATLAS) ops reset --reset-run-id $(or $(RESET_RUN_ID),$(OPS_RESET_RUN_ID)) --format text

ops-kind-up: ## Ensure local kind cluster plan is valid
	@$(DEV_ATLAS) ops install --kind --plan --profile $(PROFILE) --allow-subprocess --allow-write --format text

ops-kind-down: ## Delete local kind cluster for selected profile
	@$(DEV_ATLAS) ops down --profile $(PROFILE) --allow-subprocess --format text

ops-status: ## Query local cluster pod status
	@$(DEV_ATLAS) ops status --target pods --profile $(PROFILE) --allow-subprocess --format json

ops-tools-verify: ## Verify required ops external tools
	@$(DEV_ATLAS) ops verify-tools --allow-subprocess --format json

ops-pins-check: ## Validate unified reproducibility pins
	@$(DEV_ATLAS) ops pins check --format text

ops-pins-update: ## Refresh unified reproducibility pins
	@$(DEV_ATLAS) ops pins update --i-know-what-im-doing --allow-subprocess --format text

.PHONY: ops ops-help ops-doctor ops-validate ops-render ops-install-plan ops-up ops-down ops-clean ops-reset ops-kind-up ops-kind-down ops-status ops-tools-verify ops-pins-check ops-pins-update
