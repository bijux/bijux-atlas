# Scope: product-facing release artifact wrappers only.
# Public targets: product, bootstrap, docker-*, chart-package/chart-validate, rename-lint, docs-lint-names
SHELL := /bin/sh

product: ## Run canonical product verification lane
	@./bin/atlasctl product check

bootstrap:
	@./bin/atlasctl product bootstrap


docker-build:
	@./bin/atlasctl product docker build


docker-check: ## Docker fast checks: contracts + build + runtime smoke
	@./bin/atlasctl product docker check


docker-push:
	@./bin/atlasctl product docker push


docker-release: ## CI-only docker release lane (build + contracts + push)
	@./bin/atlasctl product docker release


chart-package:
	@./bin/atlasctl product chart package


chart-validate: ## Validate chart via lint/template and values contract schema checks
	@./bin/atlasctl product chart validate


rename-lint: ## Enforce durable naming rules for docs/scripts and concept ownership
	@./bin/atlasctl product naming lint


docs-lint-names: ## Enforce durable naming contracts, registries, and inventory
	@./bin/atlasctl product docs naming-lint

.PHONY: product bootstrap docker-build docker-check docker-push docker-release chart-package chart-validate rename-lint docs-lint-names
