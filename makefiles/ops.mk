# Scope: ops area wrappers only.
# Public targets: ops, ops-help, ops-doctor, ops-validate, ops-render, ops-install-plan, ops-up, ops-down, ops-clean, ops-reset, ops-status, ops-kind-up, ops-kind-down, ops-tools-verify, ops-pins-check, ops-pins-update
SHELL := /bin/sh

BIJUX ?= bijux
BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas
PROFILE ?= kind

ops: ## Canonical ops gate
	@$(BIJUX_DEV_ATLAS) ops validate --profile $(PROFILE) --format text

ops-help: ## Show ops control-plane command surface
	@$(BIJUX_DEV_ATLAS) ops --help

ops-doctor: ## Run ops doctor checks
	@$(BIJUX_DEV_ATLAS) ops doctor --profile $(PROFILE) --format json

ops-validate: ## Run ops validation checks
	@$(BIJUX_DEV_ATLAS) ops validate --profile $(PROFILE) --format json

ops-render: ## Render ops manifests for selected profile
	@$(BIJUX_DEV_ATLAS) ops render --target helm --profile $(PROFILE) --allow-subprocess --format json

ops-install-plan: ## Print install plan without applying resources
	@$(BIJUX_DEV_ATLAS) ops install --kind --apply --plan --profile $(PROFILE) --allow-subprocess --allow-write --format json

ops-up: ## Bring up full local ops environment
	@$(BIJUX_DEV_ATLAS) ops install --kind --apply --profile $(PROFILE) --allow-subprocess --allow-write --format text

ops-down: ## Tear down full local ops environment
	@$(BIJUX_DEV_ATLAS) ops down --profile $(PROFILE) --allow-subprocess --format text

ops-clean: ## Clean generated ops outputs
	@$(BIJUX_DEV_ATLAS) ops clean --format text

ops-reset: ## Delete artifacts for RESET_RUN_ID
	@$(BIJUX_DEV_ATLAS) ops reset --reset-run-id $(RESET_RUN_ID) --format text

ops-kind-up: ## Ensure local kind cluster plan is valid
	@$(BIJUX_DEV_ATLAS) ops install --kind --plan --profile $(PROFILE) --allow-subprocess --allow-write --format text

ops-kind-down: ## Delete local kind cluster for selected profile
	@$(BIJUX_DEV_ATLAS) ops down --profile $(PROFILE) --allow-subprocess --format text

ops-status: ## Query local cluster pod status
	@$(BIJUX_DEV_ATLAS) ops status --target pods --profile $(PROFILE) --allow-subprocess --format json

ops-tools-verify: ## Verify required ops external tools
	@$(BIJUX_DEV_ATLAS) ops verify-tools --allow-subprocess --format json

ops-pins-check: ## Validate unified reproducibility pins
	@$(BIJUX_DEV_ATLAS) ops pins check --format text

ops-pins-update: ## Refresh unified reproducibility pins
	@$(BIJUX_DEV_ATLAS) ops pins update --i-know-what-im-doing --allow-subprocess --format text

.PHONY: ops ops-help ops-doctor ops-validate ops-render ops-install-plan ops-up ops-down ops-clean ops-reset ops-kind-up ops-kind-down ops-status ops-tools-verify ops-pins-check ops-pins-update
