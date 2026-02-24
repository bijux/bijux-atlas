DOCKER_RUN_ID ?= docker_run

docker: ## Canonical docker gate
	@$(DEV_ATLAS) docker check --allow-subprocess --format json

docker-build: ## Build docker images via dev-atlas wrapper
	@$(DEV_ATLAS) docker build --allow-subprocess --run-id $(DOCKER_RUN_ID) --format json

docker-check: ## Run docker smoke checks via dev-atlas wrapper
	@$(DEV_ATLAS) docker check --allow-subprocess --run-id $(DOCKER_RUN_ID) --format json

docker-push: ## Push docker images (explicit release gate)
	@$(DEV_ATLAS) docker push --allow-subprocess --i-know-what-im-doing --run-id $(DOCKER_RUN_ID) --format json

docker-release: ## Release docker artifacts (explicit release gate)
	@$(DEV_ATLAS) docker release --allow-subprocess --i-know-what-im-doing --run-id $(DOCKER_RUN_ID) --format json

.PHONY: docker docker-build docker-check docker-push docker-release
