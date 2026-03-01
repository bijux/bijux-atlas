# Scope: k8s wrapper targets delegated to bijux dev atlas ops k8s surfaces.
# Public targets: ops-k8s-tests, ops-k8s-suite, ops-k8s-template-tests, ops-k8s-contracts, ops-k8s-smoke
SHELL := /bin/sh
PROFILE ?= kind

ops-k8s-tests: ## Run canonical k8s test suite
	@$(DEV_ATLAS) ops k8s test --profile $(PROFILE) --allow-subprocess --format json

ops-k8s-suite: ## Run canonical k8s suite wrapper
	@$(DEV_ATLAS) ops k8s test --profile $(PROFILE) --allow-subprocess --format json

ops-k8s-template-tests: ## Validate k8s render contracts through control plane
	@$(DEV_ATLAS) ops k8s render --target kind --check --profile $(PROFILE) --allow-subprocess --format json

ops-k8s-contracts: ## Validate k8s contracts through control plane
	@$(DEV_ATLAS) ops k8s conformance --profile $(PROFILE) --allow-subprocess --format json

ops-k8s-smoke: ## Run k8s smoke checks through control plane
	@$(DEV_ATLAS) ops k8s smoke --profile $(PROFILE) --allow-subprocess --format json

.PHONY: ops-k8s-tests ops-k8s-suite ops-k8s-template-tests ops-k8s-contracts ops-k8s-smoke
