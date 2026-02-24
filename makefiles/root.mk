# Scope: top-level make entrypoints delegated to Rust control-plane wrappers.
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/env.mk
include makefiles/build.mk
include makefiles/_cargo.mk
include makefiles/dev.mk
include makefiles/ci.mk
include makefiles/_docker.mk
include makefiles/_policies.mk
include makefiles/_configs.mk
include makefiles/_docs.mk
include makefiles/_ops.mk
include makefiles/verification.mk

CURATED_TARGETS := \
	help list explain surface \
	dev-atlas doctor ci-local \
	dev-doctor dev-ci dev-check-ci \
	build dist clean-build build-doctor \
	check check-gates check-list gates \
	ci ci-fast ci-pr ci-nightly ci-docs \
	policies \
	docs docs-doctor docs-validate docs-build docs-serve docs-clean docs-lock \
	configs configs-doctor configs-validate configs-lint \
	ops ops-help ops-doctor ops-validate ops-render ops-install-plan ops-up ops-down ops-clean ops-reset ops-status ops-tools-verify ops-pins-check ops-pins-update \
	make-gate-no-legacy-cli-refs make-gate-no-legacy-cli-shim

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

dev-atlas: ## Print canonical dev-atlas invocation and examples
	@printf '%s\n' "Local: cargo run -q -p bijux-dev-atlas -- <args>"
	@printf '%s\n' "Installed umbrella: bijux dev atlas <args>"
	@printf '%s\n' "Examples:"
	@printf '%s\n' "  $(DEV_ATLAS) check doctor --format json"
	@printf '%s\n' "  $(DEV_ATLAS) check list --format text"
	@printf '%s\n' "  $(DEV_ATLAS) ops validate --profile kind --format json"

doctor: ## Run Rust control-plane doctor suite
	@$(MAKE) -s dev-doctor

check-gates: ## Run Rust control-plane CI-fast check suite
	@$(DEV_ATLAS) check run --suite ci_fast --format text

gates: ## Run governance gates via dev-atlas CI-fast suite
	@$(MAKE) -s check-gates

check-list: ## List checks from the Rust control-plane registry
	@$(DEV_ATLAS) check list --format text

configs-check: ## Back-compat alias to configs validation wrapper
	@$(MAKE) -s configs-validate

clean: ## Clean scoped generated outputs via control-plane wrappers
	@$(MAKE) -s ops-clean

make-gate-no-legacy-cli-refs: ## Fail if legacy Python control-plane token appears in makefiles
	@legacy_cli_token='atlas''ctl'; ! rg -n "$$legacy_cli_token" makefiles -g'*.mk'

make-gate-no-legacy-cli-shim: ## Fail if legacy root control-plane shim exists
	@legacy_cli_path=bin/atlas''ctl; test ! -e "$$legacy_cli_path"
.PHONY: help list explain surface ci-local dev-atlas doctor check check-list gates configs-check clean make-gate-no-legacy-cli-refs make-gate-no-legacy-cli-shim
