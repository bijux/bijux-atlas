# Scope: local developer convenience cargo targets (dev-only).
# Public targets: none
SHELL := /bin/sh

DEV_ISO_TAG ?= dev-ci-local
ROOT_MAKE ?= Makefile
ISO_DEV = ISO_TAG="$(DEV_ISO_TAG)" ./bin/atlasctl env isolate --tag "$(DEV_ISO_TAG)" --reuse

dev-fmt: ## DEV_ONLY=1
	@$(ISO_DEV) ./bin/atlasctl dev fmt

dev-lint: ## DEV_ONLY=1
	@$(ISO_DEV) ./bin/atlasctl dev lint

dev-check: ## DEV_ONLY=1
	@$(ISO_DEV) ./bin/atlasctl dev check

dev-test: ## DEV_ONLY=1
	@$(ISO_DEV) ./bin/atlasctl dev test

dev-test-all: ## DEV_ONLY=1
	@$(ISO_DEV) ./bin/atlasctl dev test --all

dev-audit: ## DEV_ONLY=1
	@$(ISO_DEV) ./bin/atlasctl dev audit

dev-coverage: ## DEV_ONLY=1
	@$(ISO_DEV) ./bin/atlasctl dev coverage

dev-ci: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) ci

dev-clean: ## DEV_ONLY=1
	@rm -rf "artifacts/isolate/$(DEV_ISO_TAG)"
	@echo "removed artifacts/isolate/$(DEV_ISO_TAG)"

.PHONY: dev-fmt dev-lint dev-check dev-test dev-test-all dev-audit dev-coverage dev-ci dev-clean
