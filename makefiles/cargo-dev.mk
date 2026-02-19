SHELL := /bin/sh

DEV_ISO_TAG ?= dev-ci-local
ROOT_MAKE ?= Makefile
ISO_DEV = ./scripts/bin/isolate --tag "$(DEV_ISO_TAG)" --reuse

dev-fmt: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _fmt

dev-lint: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _lint

dev-check: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _check

dev-test: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _test

dev-test-all: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _test-all

dev-audit: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _audit

dev-coverage: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _coverage

dev-ci: ## DEV_ONLY=1
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) ci

dev-clean: ## DEV_ONLY=1
	@rm -rf "artifacts/isolate/$(DEV_ISO_TAG)"
	@echo "removed artifacts/isolate/$(DEV_ISO_TAG)"

.PHONY: dev-fmt dev-lint dev-check dev-test dev-test-all dev-audit dev-coverage dev-ci dev-clean
