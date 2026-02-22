# Scope: product-facing docker/chart/contracts bootstrap targets split from root.
# Public targets: none
SHELL := /bin/sh

bootstrap:
	@./bin/atlasctl product bootstrap

k8s: ## Run canonical k8s verification lane
	@./bin/atlasctl k8s render

load: ## Run canonical load verification lane
	@./bin/atlasctl load smoke

obs: ## Run canonical observability verification lane
	@./bin/atlasctl obs verify

docker-build:
	@./bin/atlasctl product docker build

docker-check: ## Docker fast checks: contracts + build + runtime smoke
	@./bin/atlasctl product docker check

docker-smoke:
	@./bin/atlasctl docker smoke --image "$${DOCKER_IMAGE:-bijux-atlas:local}"

docker-scan:
	@./bin/atlasctl docker scan --image "$${DOCKER_IMAGE:-bijux-atlas:local}"

docker-push:
	@./bin/atlasctl product docker push

docker-release: ## CI-only docker release lane (build + contracts + push)
	@./bin/atlasctl product docker release

chart-package:
	@./bin/atlasctl product chart package

chart-verify:
	@./bin/atlasctl product chart verify

chart-validate: ## Validate chart via lint/template and values contract schema checks
	@./bin/atlasctl product chart validate

docker-contracts: ## Validate Docker layout/policy/no-latest contracts
	@./bin/atlasctl check domain docker

rename-lint: ## Enforce durable naming rules for docs/scripts and concept ownership
	@./bin/atlasctl product naming lint

docs-lint-names: ## Enforce durable naming contracts, registries, and inventory
	@./bin/atlasctl product docs naming-lint

internal/product/doctor: ## Print tool/env/path diagnostics and store doctor report
	@RUN_ID="$${RUN_ID:-doctor-$(MAKE_RUN_TS)}" ./bin/atlasctl make doctor

prereqs: ## Check required binaries and versions and store prereqs report
	@./bin/atlasctl make prereqs

dataset-id-lint: ## Validate DatasetId/DatasetKey contract usage across ops fixtures
	@./bin/atlasctl ops datasets lint-ids

.PHONY: bootstrap k8s load obs docker-build docker-check docker-smoke docker-scan docker-push docker-release chart-package chart-verify chart-validate docker-contracts rename-lint docs-lint-names prereqs dataset-id-lint
