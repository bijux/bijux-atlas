# Scope: policy validation wrapper targets delegated to the control plane.
# Public targets: policies
policies: ## Run control-plane policies validation
	@$(DEV_ATLAS) policies validate --format json

.PHONY: policies
