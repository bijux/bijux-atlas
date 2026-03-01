# Scope: top-level make entrypoints delegated to Rust control-plane wrappers.
# Public targets: help and curated includes from child modules
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)
ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local

include make/cargo.mk
include make/configs.mk
include make/docs.mk
include make/docker.mk
include make/ops.mk
include make/policies.mk
include make/build.mk
include make/ci.mk
include make/dev.mk
include make/runenv.mk
include make/gates.mk
include make/k8s.mk
include make/verification.mk

CURATED_TARGETS := \
	artifacts-clean build clean contracts contracts-all contracts-changed contracts-ci contracts-configs contracts-docker contracts-docs contracts-fast contracts-help contracts-json contracts-make contracts-merge contracts-ops contracts-pr contracts-release contracts-root \
	docker docker-contracts docker-contracts-effect docker-gate doctor \
	fmt help \
	k8s-render k8s-validate \
	lint lint-make make-target-list \
	ops-contracts ops-contracts-effect ops-fast ops-nightly ops-pr \
	stack-down stack-up \
	test test-all

help: ## Show curated make targets owned by Rust control-plane wrappers
	@$(DEV_ATLAS) make surface --format text
	@printf '%s\n' "docs: docs/development/tooling/make.md" "contracts: make/CONTRACT.md"

_internal-list: ## Print curated make target names
	@$(DEV_ATLAS) make surface --format text

_internal-explain: ## Explain curated target ownership (TARGET=<name>)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make explain TARGET=<name>" >&2; exit 2; }
	@case " $(CURATED_TARGETS) " in \
	  *" $${TARGET} "*) echo "$${TARGET}: delegated via make/*.mk wrappers to bijux dev atlas or cargo" ;; \
	  *) echo "$${TARGET}: not available (unsupported target removed during Rust control-plane cutover)"; exit 2 ;; \
	esac

_internal-surface: ## Print make surface and docs pointers for Rust control plane
	@$(MAKE) -s help
	@printf '%s\n' "Docs: docs/development/tooling/dev-atlas-ops.md docs/development/tooling/dev-atlas-docs.md"

doctor: ## Run Rust control-plane doctor suite as JSON
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite repo_required --include-internal --include-slow --allow-git --format json"
	@$(MAKE) -s make-contract-check
	@mkdir -p $(ARTIFACT_ROOT)/doctor/$(RUN_ID)
	@$(DEV_ATLAS) check run --suite repo_required --include-internal --include-slow --allow-git --format json --out $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/repo-hygiene-report.json >/dev/null
	@$(DEV_ATLAS) check tree-budgets --format json | tee $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/tree-budgets.json >/dev/null
	@$(DEV_ATLAS) check repo-doctor --format json | tee $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/report.json >/dev/null
	@printf '{\n  "schema_version": 1,\n  "text": "repo touched paths report",\n  "touched_paths": [\n' > $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/touched-paths-report.json
	@git status --porcelain | awk '{print $$2}' | LC_ALL=C sort -u | awk 'BEGIN{first=1} { if (!first) printf(",\\n"); first=0; printf("    \"%s\"", $$0)} END{printf("\\n") }' >> $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/touched-paths-report.json
	@printf '  ]\n}\n' >> $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/touched-paths-report.json

clean: ## Clean ephemeral artifacts through the control plane
	@$(DEV_ATLAS) artifacts clean --allow-write --format text

artifacts-clean: ## Clean ephemeral artifacts through the control plane
	@$(DEV_ATLAS) artifacts clean --allow-write --format text

lint-make: ## Run make contracts through the control plane
	@$(DEV_ATLAS) contracts make --mode static --format text

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

.PHONY: help _internal-list _internal-explain _internal-surface _internal-lint-make _internal-make-drift-report artifacts-clean clean doctor k8s-render k8s-validate lint-make stack-up stack-down ops-fast ops-pr ops-nightly
