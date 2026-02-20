# Scope: legacy aliases and historical targets, opt-in only.
# Public targets: none
SHELL := /bin/sh

legacy/config-validate-core: ## Legacy config schema/contracts validation implementation
	@python3 ./scripts/public/generate-config-key-registry.py
	@python3 ./scripts/public/config-validate.py
	@python3 ./scripts/public/config-drift-check.py

legacy/root-fast: ## Legacy preserved deterministic lane
	@$(call with_iso,root,$(MAKE) -s gates-check configs/all lane-cargo ci-deny ops-contracts-check docs-lint-names)

legacy/root-local-full: ## Legacy local superset gate with 5 parallel isolated lanes + unified summary
	@PARALLEL=1 MODE=root-local RUN_ID="$${RUN_ID:-legacy-root-local-$(MAKE_RUN_TS)}" ./ops/run/root-lanes.sh

legacy/root-local-fast: ## Legacy local superset fast mode (skip stack-smoke lane)
	@PARALLEL=0 MODE=root RUN_ID="$${RUN_ID:-legacy-root-fast-$(MAKE_RUN_TS)}" ./ops/run/root-lanes.sh

legacy/ci: ## Legacy root + CI-only packaging/publish checks
	@$(call with_iso,ci,$(MAKE) -s root ci-release-binaries ci-docs-build ci-release-compat-matrix-verify)

legacy/nightly: ## Legacy nightly superset (ci + nightly ops suites)
	@$(call with_iso,nightly,$(MAKE) -s ci ops-load-nightly ops-drill-suite ops-realdata)

legacy/local-fast-loop: ## Legacy fast local loop
	@mkdir -p "$(LOCAL_ISO_ROOT)/target" "$(LOCAL_ISO_ROOT)/cargo-home" "$(LOCAL_ISO_ROOT)/tmp"
	@$(LOCAL_ENV) $(MAKE) fmt
	@$(LOCAL_ENV) $(MAKE) lint
	@$(LOCAL_ENV) $(MAKE) test

legacy/local-full-loop: ## Legacy full local loop
	@mkdir -p "$(LOCAL_FULL_ISO_ROOT)/target" "$(LOCAL_FULL_ISO_ROOT)/cargo-home" "$(LOCAL_FULL_ISO_ROOT)/tmp"
	@$(LOCAL_FULL_ENV) $(MAKE) fmt
	@$(LOCAL_FULL_ENV) $(MAKE) lint
	@$(LOCAL_FULL_ENV) $(MAKE) audit
	@$(LOCAL_FULL_ENV) $(MAKE) test
	@$(LOCAL_FULL_ENV) $(MAKE) coverage
	@$(LOCAL_FULL_ENV) $(MAKE) docs
	@$(LOCAL_FULL_ENV) $(MAKE) docs-freeze

legacy/contracts: ## Legacy contracts meta pipeline
	@ISO_ROOT=artifacts/isolate/contracts $(MAKE) ssot-check
	@ISO_ROOT=artifacts/isolate/contracts $(MAKE) policy-lint
	@ISO_ROOT=artifacts/isolate/contracts $(MAKE) policy-schema-drift
	@ISO_ROOT=artifacts/isolate/contracts $(MAKE) openapi-drift
	@ISO_ROOT=artifacts/isolate/contracts $(MAKE) docs-freeze

legacy/hygiene: ## Legacy repo hygiene checks
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) layout-check
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) scripts-audit
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) ci-workflows-make-only
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) ci-make-help-drift
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) path-contract-check

.PHONY: legacy/config-validate-core legacy/root-fast legacy/root-local-full legacy/root-local-fast legacy/ci legacy/nightly legacy/local-fast-loop legacy/local-full-loop legacy/contracts legacy/hygiene
