policies: ## Run control-plane policies validation
	@$(DEV_ATLAS) policies validate --format json

.PHONY: policies
