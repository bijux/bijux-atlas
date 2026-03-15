# Scope: top-level make entrypoints delegated to Rust control-plane wrappers.
# Public targets: help and curated includes from child modules
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS ?= auto
FAIL_FAST ?= 0
ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local
SUITE_FAIL_FAST_FLAG := $(if $(filter 1 true yes,$(FAIL_FAST)),--fail-fast,--no-fail-fast)

include makes/build.mk
include makes/cargo.mk
include makes/ci.mk
include makes/configs.mk
include makes/dev.mk
include makes/docker.mk
include makes/docs.mk
include makes/k8s.mk
include makes/ops.mk
include makes/policies.mk
include makes/runenv.mk
include makes/verification.mk

CURATED_TARGETS := \
	build ci-fast ci-nightly ci-pr clean docker doctor help k8s-render k8s-validate kind-down kind-reset kind-status kind-up lint-make openapi-generate ops-contracts ops-contracts-effect registry-doctor release-plan release-verify root-surface-explain stack-down stack-up suites-list tests-all

help: ## Show curated makes targets owned by Rust control-plane wrappers
	@$(DEV_ATLAS) makes surface --format $(FORMAT)
	@printf '%s\n' "guide: docs/06-development/automation-control-plane.md" "reference: docs/07-reference/automation-command-surface.md" "makes: makes/README.md"

_internal-list: ## Print curated makes target names
	@$(DEV_ATLAS) makes list --format $(FORMAT)

_internal-explain: ## Explain curated target ownership (TARGET=<name>)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make explain TARGET=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) makes explain "$${TARGET}" --format $(FORMAT)

_internal-surface: ## Print the makes surface and docs pointers for Rust control plane
	@$(MAKE) -s help
	@printf '%s\n' "Docs: docs/06-development/automation-control-plane.md docs/07-reference/automation-command-surface.md makes/README.md"

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

lint-make: ## Run the governed make-required check suite
	@$(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format $(FORMAT)

_internal-lint-make: ## Run make domain checks via control-plane registry
	@$(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format $(FORMAT)

_internal-make-drift-report: ## Generate make drift report artifact from make-domain checks
	@mkdir -p $(ARTIFACT_ROOT)/make-drift/$(RUN_ID)
	@$(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format $(FORMAT) --out $(ARTIFACT_ROOT)/make-drift/$(RUN_ID)/report.json >/dev/null

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

release-verify: ## Verify a release evidence tarball through bijux-dev-atlas
	@[ -n "$${EVIDENCE:-}" ] || { echo "usage: make release-verify EVIDENCE=<tarball>" >&2; exit 2; }
	@$(DEV_ATLAS) release verify --evidence "$${EVIDENCE}" --format $(FORMAT)

.PHONY: help _internal-list _internal-explain _internal-surface _internal-lint-make _internal-make-drift-report checks-effect checks-group checks-pure checks-tag clean doctor kind-down kind-reset kind-status kind-up openapi-generate registry-doctor release-plan release-verify root-surface-explain k8s-render k8s-validate lint-make stack-up stack-down suites-list tests-all
