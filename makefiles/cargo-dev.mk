SHELL := /bin/sh

DEV_ISO_TAG ?= dev-ci-local
ROOT_MAKE ?= Makefile
ISO_DEV = ./bin/isolate --tag "$(DEV_ISO_TAG)" --reuse

dev-fmt:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _fmt

dev-lint:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _lint

dev-check:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _check

dev-test:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _test

dev-test-all:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _test-all

dev-audit:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _audit

dev-coverage:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _coverage

dev-ci:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) ci

dev-clean:
	@rm -rf "artifacts/isolates/$(DEV_ISO_TAG)"
	@echo "removed artifacts/isolates/$(DEV_ISO_TAG)"

.PHONY: dev-fmt dev-lint dev-check dev-test dev-test-all dev-audit dev-coverage dev-ci dev-clean
