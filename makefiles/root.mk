# Scope: top-level make entrypoints delegated to Rust control-plane wrappers.
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/env.mk
include makefiles/dev.mk
include makefiles/ci.mk
include makefiles/configs.mk
include makefiles/docs.mk
include makefiles/ops.mk
include makefiles/verification.mk

CURATED_TARGETS := \
	help list explain surface \
	doctor ci-local \
	dev-doctor dev-ci dev-check-ci \
	ci ci-fast ci-nightly ci-docs \
	docs docs-doctor docs-validate docs-build docs-serve docs-clean docs-lock \
	configs configs-doctor configs-validate configs-lint \
	ops ops-help ops-doctor ops-validate ops-render ops-install-plan ops-up ops-down ops-clean ops-reset ops-status ops-tools-verify ops-pins-check ops-pins-update

help: ## Show curated make targets owned by Rust control-plane wrappers
	@printf '%s\n' "Curated make targets (Rust control plane):"; \
	for t in $(CURATED_TARGETS); do printf '  %s\n' "$$t"; done

list: ## Print curated make target names
	@for t in $(CURATED_TARGETS); do printf '%s\n' "$$t"; done

explain: ## Explain curated target ownership (TARGET=<name>)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make explain TARGET=<name>" >&2; exit 2; }
	@case " $(CURATED_TARGETS) " in \
	  *" $${TARGET} "*) echo "$${TARGET}: delegated via makefiles/*.mk wrappers to bijux dev atlas or cargo" ;; \
	  *) echo "$${TARGET}: not available (unsupported target removed during Rust control-plane cutover)"; exit 2 ;; \
	esac

surface: ## Print make surface and docs pointers for Rust control plane
	@$(MAKE) -s help
	@printf '%s\n' "Docs: docs/development/tooling/dev-atlas-ops.md docs/development/tooling/dev-atlas-docs.md"

ci-local: ## Local runner mirroring CI control-plane entrypoints
	@$(MAKE) -s dev-ci

doctor: ## Run Rust control-plane doctor suite
	@$(MAKE) -s dev-doctor

configs-check: ## Back-compat alias to configs validation wrapper
	@$(MAKE) -s configs-validate

.PHONY: help list explain surface ci-local doctor configs-check
