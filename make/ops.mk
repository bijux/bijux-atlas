# Scope: ops area wrappers only.
# Public targets: ops, ops-contracts, ops-contracts-effect, ops-help, ops-doctor, ops-validate, ops-artifact-root-check, ops-render, ops-install-plan, ops-up, ops-down, ops-clean, ops-reset, ops-status, ops-kind-up, ops-kind-down, ops-tools-verify, ops-pins-check, ops-pins-update, ops-stack, ops-k8s, ops-e2e, ops-load, ops-load-plan, ops-load-run, ops-observability
# All external tools are invoked through bijux dev atlas command surfaces.
SHELL := /bin/sh
BIJUX ?= bijux
BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas
PROFILE ?= kind
OPS_RESET_RUN_ID ?= ops_reset
OPS_CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/ops-contracts/$(RUN_ID)

ops: ## Canonical ops gate
	@$(DEV_ATLAS) ops doctor --profile $(PROFILE) --format $(FORMAT)

ops-contracts: ## Run static ops contracts via dev-atlas contracts runner
	@mkdir -p $(OPS_CONTRACTS_ARTIFACT_ROOT) && printf '%s\n' "run: $(DEV_ATLAS) contracts ops --mode static --artifacts-root $(OPS_CONTRACTS_ARTIFACT_ROOT)" && $(DEV_ATLAS) contracts ops --mode static --artifacts-root $(OPS_CONTRACTS_ARTIFACT_ROOT)

ops-contracts-effect: ## Run effect ops contracts via dev-atlas contracts runner
	@mkdir -p $(OPS_CONTRACTS_ARTIFACT_ROOT) && printf '%s\n' "run: $(DEV_ATLAS) contracts ops --mode effect --allow-subprocess --allow-network --artifacts-root $(OPS_CONTRACTS_ARTIFACT_ROOT)" && $(DEV_ATLAS) contracts ops --mode effect --allow-subprocess --allow-network --artifacts-root $(OPS_CONTRACTS_ARTIFACT_ROOT)

ops-help: ## Show ops control-plane command surface
	@$(DEV_ATLAS) ops --help

ops-doctor: ## Run ops doctor checks
	@$(DEV_ATLAS) ops doctor --profile $(PROFILE) --format $(FORMAT)

ops-validate: ## Run ops validation checks
	@$(DEV_ATLAS) ops validate --profile $(PROFILE) --format $(FORMAT)

ops-artifact-root-check: ## Fail fast on retired ops artifact path drift
	@$(DEV_ATLAS) check run --id 'checks_ops_*artifact*' --format $(FORMAT) && $(DEV_ATLAS) check run --id 'checks_ops_runtime_output_roots_under_ops_absent' --format $(FORMAT)

ops-render: ## Render ops manifests for selected profile
	@$(DEV_ATLAS) ops render --target kind --check --profile $(PROFILE) --format $(FORMAT)

ops-install-plan: ## Print install plan without applying resources
	@$(DEV_ATLAS) ops install --kind --apply --plan --profile $(PROFILE) --allow-subprocess --allow-write --format $(FORMAT)

ops-up: ## Bring up full local ops environment
	@if command -v kind >/dev/null 2>&1 && kind get clusters 2>/dev/null | grep -Eq '^(normal|bijux-atlas-e2e)$$'; then echo "ops-up: local kind cluster already exists"; else $(DEV_ATLAS) ops install --kind --apply --profile $(PROFILE) --allow-subprocess --allow-write --format $(FORMAT); fi

ops-down: ## Tear down full local ops environment
	@$(DEV_ATLAS) ops down --profile $(PROFILE) --allow-subprocess --format $(FORMAT)

ops-clean: ## Clean generated ops outputs
	@$(DEV_ATLAS) ops clean --format $(FORMAT)

ops-reset: ## Delete artifacts for RESET_RUN_ID
	@$(DEV_ATLAS) ops reset --reset-run-id $(or $(RESET_RUN_ID),$(OPS_RESET_RUN_ID)) --format $(FORMAT)

ops-kind-up: ## Ensure local kind cluster plan is valid
	@$(DEV_ATLAS) ops install --kind --plan --profile $(PROFILE) --allow-subprocess --allow-write --format $(FORMAT)

ops-kind-down: ## Delete local kind cluster for selected profile
	@$(DEV_ATLAS) ops down --profile $(PROFILE) --allow-subprocess --format $(FORMAT)

ops-status: ## Query local cluster pod status
	@$(DEV_ATLAS) ops status --target pods --profile $(PROFILE) --allow-subprocess --format $(FORMAT)

ops-stack: ## Show stack status through grouped ops namespace
	@$(DEV_ATLAS) ops stack status --target pods --profile $(PROFILE) --allow-subprocess --format $(FORMAT)

ops-k8s: ## Run k8s conformance wrapper through grouped ops namespace
	@$(DEV_ATLAS) ops k8s test --profile $(PROFILE) --allow-subprocess --format $(FORMAT)

ops-e2e: ## Run e2e scenario wrapper through grouped ops namespace
	@$(DEV_ATLAS) ops e2e run --profile $(PROFILE) --format $(FORMAT)

ops-load: ## Run load wrapper through grouped ops namespace
	@$(DEV_ATLAS) ops load plan mixed --profile $(PROFILE) --format $(FORMAT)

ops-load-plan: ## Resolve load suite into script and thresholds
	@$(DEV_ATLAS) ops load plan $(or $(SUITE),mixed) --profile $(PROFILE) --format $(FORMAT)

ops-load-run: ## Execute load suite through canonical control plane
	@$(DEV_ATLAS) ops load run $(or $(SUITE),mixed) --profile $(PROFILE) --allow-subprocess --allow-network --allow-write --format $(FORMAT)

ops-observability: ## Run observability verification wrapper through grouped ops namespace
	@$(DEV_ATLAS) ops observe verify --profile $(PROFILE) --format $(FORMAT)

ops-tools-verify: ## Verify required ops external tools
	@$(DEV_ATLAS) ops verify-tools --allow-subprocess --format $(FORMAT)

ops-pins-check: ## Validate unified reproducibility pins
	@$(DEV_ATLAS) ops pins check --format $(FORMAT)

ops-pins-update: ## Refresh unified reproducibility pins
	@$(DEV_ATLAS) ops pins update --i-know-what-im-doing --allow-subprocess --format $(FORMAT)

.PHONY: ops ops-contracts ops-contracts-effect ops-help ops-doctor ops-validate ops-artifact-root-check ops-render ops-install-plan ops-up ops-down ops-clean ops-reset ops-kind-up ops-kind-down ops-status ops-stack ops-k8s ops-e2e ops-load ops-load-plan ops-load-run ops-observability ops-tools-verify ops-pins-check ops-pins-update
