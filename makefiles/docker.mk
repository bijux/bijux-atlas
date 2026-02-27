DOCKER_RUN_ID ?= $(RUN_ID)
DOCKER_CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/docker-contracts/$(DOCKER_RUN_ID)

docker: docker-contracts

docker-contracts:
	@$(DEV_ATLAS) contracts docker --mode static --artifacts-root $(DOCKER_CONTRACTS_ARTIFACT_ROOT) --format json

docker-contracts-effect:
	@$(DEV_ATLAS) contracts docker --mode effect --allow-subprocess --allow-network --artifacts-root $(DOCKER_CONTRACTS_ARTIFACT_ROOT) --format json

docker-gate: docker-contracts

.PHONY: docker docker-contracts docker-contracts-effect docker-gate
