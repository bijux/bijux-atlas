DOCKER_RUN_ID ?= $(RUN_ID)
DOCKER_CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/docker-contracts/$(DOCKER_RUN_ID)

docker: docker-contracts

docker-contracts: ## Run static docker contracts via dev-atlas contracts runner
	@$(DEV_ATLAS) contracts docker --mode static --artifacts-root $(DOCKER_CONTRACTS_ARTIFACT_ROOT)

docker-contracts-effect: ## Run effect docker contracts via dev-atlas contracts runner
	@$(DEV_ATLAS) contracts docker --mode effect --allow-subprocess --allow-network --artifacts-root $(DOCKER_CONTRACTS_ARTIFACT_ROOT)

docker-gate: docker-contracts ## Compatibility alias for static docker contracts

.PHONY: docker docker-contracts docker-contracts-effect docker-gate
