# Scope: top-level make entrypoints delegated to Rust control-plane wrappers.
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/_cargo.mk
include makefiles/_configs.mk
include makefiles/_docs.mk
include makefiles/_docker.mk
include makefiles/_ops.mk
include makefiles/_policies.mk
include makefiles/build.mk
include makefiles/ci.mk
include makefiles/dev.mk
include makefiles/env.mk
include makefiles/gates.mk
include makefiles/k8s.mk
include makefiles/verification.mk

CURATED_TARGETS := \
	help list explain surface \
	dev-atlas doctor ci-local \
	dev-doctor dev-ci dev-check-ci \
	build build-release build-ci build-meta dist dist-verify clean-build clean-dist build-doctor \
	check check-gates check-list gates \
	gate-10 gate-20 \
	ci ci-fast ci-pr ci-nightly ci-docs \
	lanes verify \
	lint-makefiles lint-root lint-policies lint-docker lint-ops lint-configs lint-docs \
	policies \
	docs docs-doctor docs-validate docs-build docs-serve docs-clean docs-lock \
	configs configs-doctor configs-validate configs-lint \
	ops ops-help ops-doctor ops-validate ops-artifact-root-check ops-render ops-install-plan ops-up ops-down ops-clean ops-reset ops-status ops-stack ops-k8s ops-e2e ops-load ops-observability ops-tools-verify ops-pins-check ops-pins-update \
	ops-k8s-tests ops-k8s-suite ops-k8s-template-tests ops-k8s-contracts \
	make-gate-no-retired-cli-refs make-gate-no-retired-cli-shim

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

doctor: ## Run Rust control-plane doctor suite as JSON
	@$(DEV_ATLAS) check doctor --format json

check-gates: ## Run Rust control-plane CI-fast check suite
	@$(DEV_ATLAS) check run --suite ci_fast --format text

gates: ## Run governance gates via dev-atlas CI-fast suite
	@$(MAKE) -s check-gates

check-list: ## List checks from the Rust control-plane registry
	@$(DEV_ATLAS) check list --format text

clean: ## Clean scoped generated outputs via control-plane wrappers
	@$(MAKE) -s ops-clean

verify: ## Run repo verification orchestration via dev-atlas checks
	@$(DEV_ATLAS) check run --suite ci --format json

lanes: ## Print CI lane mapping to dev-atlas suites
	@printf '%s\n' "ci-pr -> check run --suite ci_fast"; \
	printf '%s\n' "ci-nightly -> check run --suite deep --include-internal --include-slow"; \
	printf '%s\n' "ci-docs -> check run --domain docs"

lint-makefiles: ## Lint make wrappers via dev-atlas make checks
	@$(DEV_ATLAS) check run --domain make --format json

lint-root: ## Lint root repo contracts via dev-atlas root checks
	@$(DEV_ATLAS) check run --domain root --format json

lint-policies: ## Lint control-plane policies via dev-atlas policies validate
	@$(DEV_ATLAS) policies validate --format json

lint-docker: ## Lint docker contracts via dev-atlas docker checks
	@$(DEV_ATLAS) check run --domain docker --format json

lint-ops: ## Lint ops contracts via dev-atlas ops checks
	@$(DEV_ATLAS) check run --domain ops --format json

lint-configs: ## Lint configs contracts via dev-atlas configs checks
	@$(DEV_ATLAS) check run --domain configs --format json

lint-docs: ## Lint docs contracts via dev-atlas docs checks
	@$(DEV_ATLAS) check run --domain docs --format json

make-gate-no-retired-cli-refs: ## Fail if retired Python control-plane token appears in makefiles
	@retired_cli_token='atlas''ctl'; ! rg -n "$$retired_cli_token" makefiles -g'*.mk'

make-gate-no-retired-cli-shim: ## Fail if retired root control-plane shim exists
	@retired_cli_path=bin/atlas''ctl; test ! -e "$$retired_cli_path"
.PHONY: help list explain surface ci-local dev-atlas doctor check check-list gates gate-10 gate-20 clean verify lanes lint-makefiles lint-root lint-policies lint-docker lint-ops lint-configs lint-docs make-gate-no-retired-cli-refs make-gate-no-retired-cli-shim
