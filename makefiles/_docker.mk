DOCKER_RUN_ID ?= docker_run

docker: ## Canonical docker gate
	@$(DEV_ATLAS) docker validate --format json

docker-build: ## Build docker images via dev-atlas wrapper
	@$(DEV_ATLAS) docker build --allow-subprocess --run-id $(DOCKER_RUN_ID) --format json

docker-check: ## Run docker check wrapper via dev-atlas
	@$(DEV_ATLAS) docker validate --run-id $(DOCKER_RUN_ID) --format json

docker-smoke: ## Run docker smoke wrapper via dev-atlas
	@$(DEV_ATLAS) docker smoke --allow-subprocess --run-id $(DOCKER_RUN_ID) --format json

docker-scan: ## Run docker scanner wrapper via dev-atlas
	@$(DEV_ATLAS) docker scan --allow-subprocess --allow-network --run-id $(DOCKER_RUN_ID) --format json

docker-sbom: ## Emit docker sbom wrapper output via dev-atlas
	@$(DEV_ATLAS) docker sbom --allow-subprocess --run-id $(DOCKER_RUN_ID) --format json

docker-policy-check: ## Run docker tag policy checks via dev-atlas
	@$(DEV_ATLAS) docker policy check --run-id $(DOCKER_RUN_ID) --format json

docker-lock: ## Write docker image digest lockfile via dev-atlas
	@$(DEV_ATLAS) docker lock --allow-write --run-id $(DOCKER_RUN_ID) --format json

docker-push: ## Push docker images (explicit release gate)
	@$(DEV_ATLAS) docker push --allow-subprocess --allow-network --i-know-what-im-doing --run-id $(DOCKER_RUN_ID) --format json

docker-release: ## Release docker artifacts (explicit release gate)
	@$(DEV_ATLAS) docker release --allow-subprocess --allow-network --i-know-what-im-doing --run-id $(DOCKER_RUN_ID) --format json

.PHONY: docker docker-build docker-check docker-smoke docker-scan docker-sbom docker-policy-check docker-lock docker-push docker-release
