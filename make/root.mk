# Scope: top-level make entrypoints delegated to Rust control-plane wrappers.
# Public targets: help and curated includes from child modules
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS ?= auto
FAIL_FAST ?= 0
ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local
SUITE_FAIL_FAST_FLAG := $(if $(filter 1 true yes,$(FAIL_FAST)),--fail-fast,--no-fail-fast)

include make/build.mk
include make/cargo.mk
include make/ci.mk
include make/configs.mk
include make/dev.mk
include make/docker.mk
include make/docs.mk
include make/k8s.mk
include make/ops.mk
include make/policies.mk
include make/runenv.mk
include make/verification.mk

CURATED_TARGETS := \
	artifacts-clean build clean docker doctor help k8s-render k8s-validate kind-down kind-reset kind-status kind-up lint-make make-fast make-target-list openapi-generate ops-contracts ops-contracts-effect ops-fast ops-nightly ops-pr registry-doctor release-plan release-verify root-surface-explain stack-down stack-up suites-all suites-list tests-all

help: ## Show curated make targets owned by Rust control-plane wrappers
	@$(DEV_ATLAS) make surface --format $(FORMAT)
	@printf '%s\n' "guide: docs/06-development/automation-control-plane.md" "reference: docs/07-reference/automation-command-surface.md" "make: make/README.md"

_internal-list: ## Print curated make target names
	@$(DEV_ATLAS) make list --format $(FORMAT)

_internal-explain: ## Explain curated target ownership (TARGET=<name>)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make explain TARGET=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) make explain "$${TARGET}" --format $(FORMAT)

_internal-surface: ## Print make surface and docs pointers for Rust control plane
	@$(MAKE) -s help
	@printf '%s\n' "Docs: docs/06-development/automation-control-plane.md docs/07-reference/automation-command-surface.md make/README.md"

doctor: ## Run Rust control-plane doctor suite as JSON
	@printf '%s\n' "run: $(DEV_ATLAS) registry doctor --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/doctor/$(RUN_ID)
	@$(DEV_ATLAS) registry doctor --format $(FORMAT) --out $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/report.json >/dev/null
	@printf '{\n  "schema_version": 1,\n  "text": "repo touched paths report",\n  "touched_paths": [\n' > $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/touched-paths-report.json
	@git status --porcelain | awk '{print $$2}' | LC_ALL=C sort -u | awk 'BEGIN{first=1} { if (!first) printf(",\\n"); first=0; printf("    \"%s\"", $$0)} END{printf("\\n") }' >> $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/touched-paths-report.json
	@printf '  ]\n}\n' >> $(ARTIFACT_ROOT)/doctor/$(RUN_ID)/touched-paths-report.json

root-surface-explain: ## Explain why each root file and directory exists
	@$(DEV_ATLAS) run CHECK-ROOT-SURFACE-EXPLAIN-001 --format $(FORMAT)

clean: ## Clean ephemeral artifacts through the control plane
	@$(DEV_ATLAS) artifacts clean --allow-write --format $(FORMAT)

artifacts-clean: ## Clean ephemeral artifacts through the control plane
	@$(DEV_ATLAS) artifacts clean --allow-write --format $(FORMAT)

lint-make: ## Run make contracts through the control plane
	@$(DEV_ATLAS) suites run --suite make_required --mode all --format $(FORMAT)

make-fast: ## Run the fastest make-focused contract lane
	@printf '%s\n' "run: $(DEV_ATLAS) suites run --suite make_required --mode all --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/make-fast/$(RUN_ID)
	@$(DEV_ATLAS) suites run --suite make_required --mode all --format $(FORMAT) --out $(ARTIFACT_ROOT)/make-fast/$(RUN_ID)/report.json >/dev/null

_internal-lint-make: ## Run make domain checks via control-plane registry
	@$(DEV_ATLAS) suites run --suite make_required --mode all --format $(FORMAT)

_internal-make-drift-report: ## Generate make drift report artifact from make-domain checks
	@mkdir -p $(ARTIFACT_ROOT)/make-drift/$(RUN_ID)
	@$(DEV_ATLAS) suites run --suite make_required --mode all --format $(FORMAT) --out $(ARTIFACT_ROOT)/make-drift/$(RUN_ID)/report.json >/dev/null

k8s-render: ## Render Kubernetes manifests through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) ops k8s render --profile $(PROFILE) --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/k8s-render/$(RUN_ID)
	@$(DEV_ATLAS) ops k8s render --profile $(PROFILE) --format $(FORMAT) --out $(ARTIFACT_ROOT)/k8s-render/$(RUN_ID)/report.json >/dev/null

k8s-validate: ## Validate Kubernetes manifests through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) ops k8s validate --profile $(PROFILE) --allow-subprocess --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/k8s-validate/$(RUN_ID)
	@$(DEV_ATLAS) ops k8s validate --profile $(PROFILE) --allow-subprocess --format $(FORMAT) --out $(ARTIFACT_ROOT)/k8s-validate/$(RUN_ID)/report.json >/dev/null

stack-up: ## Start local ops stack through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) ops stack up --profile $(PROFILE) --allow-subprocess --allow-write --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/stack-up/$(RUN_ID)
	@$(DEV_ATLAS) ops stack up --profile $(PROFILE) --allow-subprocess --allow-write --format $(FORMAT) --out $(ARTIFACT_ROOT)/stack-up/$(RUN_ID)/report.txt >/dev/null

stack-down: ## Stop local ops stack through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) ops stack down --profile $(PROFILE) --allow-subprocess --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/stack-down/$(RUN_ID)
	@$(DEV_ATLAS) ops stack down --profile $(PROFILE) --allow-subprocess --format $(FORMAT) --out $(ARTIFACT_ROOT)/stack-down/$(RUN_ID)/report.txt >/dev/null

ops-fast: ## Run fast ops check suite through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) suites run --suite ci_fast --mode all --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/ops-fast/$(RUN_ID)
	@$(DEV_ATLAS) suites run --suite ci_fast --mode all --format $(FORMAT) --out $(ARTIFACT_ROOT)/ops-fast/$(RUN_ID)/report.json >/dev/null

ops-pr: ## Run PR ops check suite through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) suites run --suite ci_pr --mode all --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/ops-pr/$(RUN_ID)
	@$(DEV_ATLAS) suites run --suite ci_pr --mode all --format $(FORMAT) --out $(ARTIFACT_ROOT)/ops-pr/$(RUN_ID)/report.json >/dev/null

ops-nightly: ## Run nightly ops check suite through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) suites run --suite ci_nightly --mode all --format $(FORMAT)"
	@mkdir -p $(ARTIFACT_ROOT)/ops-nightly/$(RUN_ID)
	@$(DEV_ATLAS) suites run --suite ci_nightly --mode all --format $(FORMAT) --out $(ARTIFACT_ROOT)/ops-nightly/$(RUN_ID)/report.json >/dev/null

kind-up: ## Create or verify the deterministic kind simulation cluster
	@$(DEV_ATLAS) ops kind up --allow-subprocess --allow-write --format $(FORMAT)

kind-down: ## Delete the deterministic kind simulation cluster
	@$(DEV_ATLAS) ops kind down --allow-subprocess --allow-write --format $(FORMAT)

kind-reset: ## Recreate the deterministic kind simulation cluster
	-@$(MAKE) -s kind-down
	@$(MAKE) -s kind-up

kind-status: ## Report kind simulation cluster node readiness
	@$(DEV_ATLAS) ops kind status --allow-subprocess --allow-write --format $(FORMAT)

release-plan: ## Generate release readiness plan through dev-atlas
	@$(DEV_ATLAS) release plan --format $(FORMAT)

openapi-generate: ## Regenerate OpenAPI contract artifacts through dev-atlas
	@$(DEV_ATLAS) api contract --format $(FORMAT)

suites-list: ## List suite ids exposed through the control plane
	@$(DEV_ATLAS) suites list --format $(FORMAT)

suites-all: ## Run the governed validation suites sequentially
	@$(DEV_ATLAS) suites run --suite deep --mode all $(SUITE_FAIL_FAST_FLAG) --format $(FORMAT)
	@$(DEV_ATLAS) suites run --suite contracts --mode all $(SUITE_FAIL_FAST_FLAG) --format $(FORMAT)

checks-group: ## Run one checks suite group (GROUP=<name>)
	@[ -n "$${GROUP:-}" ] || { echo "usage: make checks-group GROUP=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) checks run --group "$${GROUP}" --format $(FORMAT)

checks-tag: ## Run checks suite entries with a shared tag (TAG=<name>)
	@[ -n "$${TAG:-}" ] || { echo "usage: make checks-tag TAG=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) checks run --tag "$${TAG}" --format $(FORMAT)

checks-pure: ## Run only pure checks suite entries
	@$(DEV_ATLAS) checks run --mode static --format $(FORMAT)

checks-effect: ## Run only effectful checks suite entries
	@$(DEV_ATLAS) checks run --mode effect --format $(FORMAT)

registry-doctor: ## Validate governed suite registries and mappings
	@$(DEV_ATLAS) registry doctor --format $(FORMAT)

tests-all: ## Run the deterministic test suite without external network
	@$(DEV_ATLAS) tests run --mode all --artifacts-root $(ARTIFACT_ROOT) --run-id $(RUN_ID) $(if $(filter 1 true yes,$(INCLUDE_CLIENT_PYTHON)),--include-client-python,) --format $(FORMAT)

release-verify: ## Run release verification lane through bijux-dev-atlas only
	@mkdir -p $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)
	@echo "run: $(DEV_ATLAS) suites run --suite deep --mode all --format json"
	@$(DEV_ATLAS) suites run --suite deep --mode all --format json --out $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/checks-deep.json \
	|| { echo "release-verify failed at deep validation suite; next: inspect $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/checks-deep.json and rerun with --verbose"; exit 1; }
	@echo "run: $(DEV_ATLAS) suites run --suite contracts --mode all --format json"
	@$(DEV_ATLAS) suites run --suite contracts --mode all --format json --out $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/contracts-full.json \
	|| { echo "release-verify failed at contracts suite; next: inspect $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/contracts-full.json and rerun a focused domain with --suite contracts"; exit 1; }
	@echo "run: $(DEV_ATLAS) tests run --mode all --artifacts-root $(ARTIFACT_ROOT) --run-id $(RUN_ID) --format json"
	@$(DEV_ATLAS) tests run --mode all --artifacts-root $(ARTIFACT_ROOT) --run-id $(RUN_ID) --format json --out $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/tests-all.json \
	|| { echo "release-verify failed at test-all; next: inspect $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/tests-all.json and rerun the failing suite"; exit 1; }
	@echo "run: $(DEV_ATLAS) docs build --allow-subprocess --allow-write --format json"
	@$(DEV_ATLAS) docs build --allow-subprocess --allow-write --format json --out $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/docs-build.json \
	|| { echo "release-verify failed at docs build; next: inspect $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/docs-build.json and fix docs build errors"; exit 1; }
	@echo "run: $(DEV_ATLAS) release crates dry-run --format json"
	@$(DEV_ATLAS) release crates dry-run --format json --out $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/crates-dry-run.json \
	|| { echo "release-verify failed at crates packaging dry-run; next: inspect $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/crates-dry-run.json"; exit 1; }
	@echo "run: $(DEV_ATLAS) release ops publish-plan --format json"
	@$(DEV_ATLAS) release ops publish-plan --format json --out $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/ops-packaging-dry-run.json \
	|| { echo "release-verify failed at ops packaging dry-run; next: inspect $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/ops-packaging-dry-run.json"; exit 1; }
	@printf '{\n  "schema_version": 1,\n  "status": "ok",\n  "text": "release verify passed",\n  "artifacts": {\n    "checks": "%s",\n    "contracts": "%s",\n    "tests": "%s",\n    "docs_build": "%s",\n    "crates_dry_run": "%s",\n    "ops_packaging_dry_run": "%s"\n  }\n}\n' \
	"$(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/checks-deep.json" \
	"$(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/contracts-full.json" \
	"$(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/tests-all.json" \
	"$(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/docs-build.json" \
	"$(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/crates-dry-run.json" \
	"$(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/ops-packaging-dry-run.json" \
	> $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/summary.json
	@echo "release-verify summary: $(ARTIFACT_ROOT)/release-verify/$(RUN_ID)/summary.json"

.PHONY: help _internal-list _internal-explain _internal-surface _internal-lint-make _internal-make-drift-report artifacts-clean checks-effect checks-group checks-pure checks-tag clean doctor kind-down kind-reset kind-status kind-up openapi-generate registry-doctor release-plan release-verify root-surface-explain k8s-render k8s-validate lint-make make-fast stack-up stack-down ops-fast ops-pr ops-nightly suites-all suites-list tests-all
