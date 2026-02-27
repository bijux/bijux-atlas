# Scope: top-level make entrypoints delegated to Rust control-plane wrappers.
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)
ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local

include makefiles/_cargo.mk
include makefiles/_configs.mk
include makefiles/_docs.mk
include makefiles/docker.mk
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
	help doctor fmt lint test build docker docker-contracts docker-contracts-effect docker-gate \
	k8s-render k8s-validate stack-up stack-down \
	ops-fast ops-pr ops-nightly make-target-list

help: ## Show curated make targets owned by Rust control-plane wrappers
	@printf '%s\n' "Curated make targets (Rust control plane):"; \
	for t in $(CURATED_TARGETS); do printf '  %s\n' "$$t"; done

_internal-list: ## Print curated make target names
	@for t in $(CURATED_TARGETS); do printf '%s\n' "$$t"; done

_internal-explain: ## Explain curated target ownership (TARGET=<name>)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make explain TARGET=<name>" >&2; exit 2; }
	@case " $(CURATED_TARGETS) " in \
	  *" $${TARGET} "*) echo "$${TARGET}: delegated via makefiles/*.mk wrappers to bijux dev atlas or cargo" ;; \
	  *) echo "$${TARGET}: not available (unsupported target removed during Rust control-plane cutover)"; exit 2 ;; \
	esac

_internal-surface: ## Print make surface and docs pointers for Rust control plane
	@$(MAKE) -s help
	@printf '%s\n' "Docs: docs/development/tooling/dev-atlas-ops.md docs/development/tooling/dev-atlas-docs.md"

doctor: ## Run Rust control-plane doctor suite as JSON
	@printf '%s\n' "run: $(DEV_ATLAS) check repo-doctor --format json"
	@$(MAKE) -s make-contract-check
	@$(MAKE) -s make-target-governance-check
	@$(MAKE) -s make-ci-surface-check
	@$(MAKE) -s make-public-surface-sync-check
	@$(MAKE) -s make-size-budget-check
	@$(MAKE) -s make-include-cycle-check
	@mkdir -p $(ARTIFACT_ROOT)/doctor/$(RUN_ID)
	@$(DEV_ATLAS) check tree-budgets --format json | tee $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/tree-budgets.json >/dev/null
	@$(DEV_ATLAS) check repo-doctor --format json | tee $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/report.json >/dev/null

_internal-lint-make: ## Run make domain checks via control-plane registry
	@$(DEV_ATLAS) check run --domain make --format json

_internal-make-drift-report: ## Generate make drift report artifact from make-domain checks
	@mkdir -p $(ARTIFACT_ROOT)/make-drift/$(RUN_ID)
	@$(DEV_ATLAS) check run --domain make --format json | tee $(ARTIFACT_ROOT)/make-drift/$(RUN_ID)/report.json >/dev/null

k8s-render: ## Render Kubernetes manifests through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) ops k8s render --profile $(PROFILE) --format json"
	@mkdir -p $(ARTIFACT_ROOT)/k8s-render/$(RUN_ID)
	@$(DEV_ATLAS) ops k8s render --profile $(PROFILE) --format json | tee $(ARTIFACT_ROOT)/k8s-render/$(RUN_ID)/report.json >/dev/null

k8s-validate: ## Validate Kubernetes manifests through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) ops k8s validate --profile $(PROFILE) --format json"
	@mkdir -p $(ARTIFACT_ROOT)/k8s-validate/$(RUN_ID)
	@$(DEV_ATLAS) ops k8s validate --profile $(PROFILE) --format json | tee $(ARTIFACT_ROOT)/k8s-validate/$(RUN_ID)/report.json >/dev/null

stack-up: ## Start local ops stack through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) ops stack up --profile $(PROFILE) --allow-subprocess --allow-write --format text"
	@mkdir -p $(ARTIFACT_ROOT)/stack-up/$(RUN_ID)
	@$(DEV_ATLAS) ops stack up --profile $(PROFILE) --allow-subprocess --allow-write --format text | tee $(ARTIFACT_ROOT)/stack-up/$(RUN_ID)/report.txt >/dev/null

stack-down: ## Stop local ops stack through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) ops stack down --profile $(PROFILE) --allow-subprocess --format text"
	@mkdir -p $(ARTIFACT_ROOT)/stack-down/$(RUN_ID)
	@$(DEV_ATLAS) ops stack down --profile $(PROFILE) --allow-subprocess --format text | tee $(ARTIFACT_ROOT)/stack-down/$(RUN_ID)/report.txt >/dev/null

ops-fast: ## Run fast ops check suite through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite ci_fast --format json"
	@mkdir -p $(ARTIFACT_ROOT)/ops-fast/$(RUN_ID)
	@$(DEV_ATLAS) check run --suite ci_fast --format json | tee $(ARTIFACT_ROOT)/ops-fast/$(RUN_ID)/report.json >/dev/null

ops-pr: ## Run PR ops check suite through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite ci_pr --format json"
	@mkdir -p $(ARTIFACT_ROOT)/ops-pr/$(RUN_ID)
	@$(DEV_ATLAS) check run --suite ci_pr --format json | tee $(ARTIFACT_ROOT)/ops-pr/$(RUN_ID)/report.json >/dev/null

ops-nightly: ## Run nightly ops check suite through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite ci_nightly --include-internal --include-slow --format json"
	@mkdir -p $(ARTIFACT_ROOT)/ops-nightly/$(RUN_ID)
	@$(DEV_ATLAS) check run --suite ci_nightly --include-internal --include-slow --format json | tee $(ARTIFACT_ROOT)/ops-nightly/$(RUN_ID)/report.json >/dev/null

.PHONY: help _internal-list _internal-explain _internal-surface _internal-lint-make _internal-make-drift-report doctor k8s-render k8s-validate stack-up stack-down ops-fast ops-pr ops-nightly
