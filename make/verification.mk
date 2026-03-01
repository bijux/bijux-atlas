# Scope: verification orchestrator for running all targets in a module makefile.
# Usage: make verification <module>  (example: make verification ops)
SHELL := /bin/sh

ifeq (verification,$(firstword $(MAKECMDGOALS)))
VERIFICATION_MODULE := $(word 2,$(MAKECMDGOALS))
ifneq ($(strip $(VERIFICATION_MODULE)),)
$(eval $(VERIFICATION_MODULE):;@:)
endif
endif

VERIFICATION_ACCEPT_CODES ?= 0
VERIFICATION_ACCEPT_CODES_configs ?= 0 2
VERIFICATION_ACCEPT_CODES__configs ?= 0 2
VERIFICATION_ACCEPT_CODES_docs ?= 0 2
VERIFICATION_ACCEPT_CODES__docs ?= 0 2

verification: ## Run every target declared in make/<module>.mk
	@module="$(VERIFICATION_MODULE)"; \
	if [ -z "$$module" ]; then \
		printf '%s\n' "usage: make verification <module>"; \
		exit 2; \
	fi; \
	$(DEV_ATLAS) make verify-module "$$module" --allow-subprocess --format text

.PHONY: verification
