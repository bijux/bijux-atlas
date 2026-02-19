# Scope: top-level publication surface and orchestration for public make targets.
# Public targets: declared here; all other makefiles expose internal-only targets.
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS  ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/env.mk
include makefiles/_macros.mk
include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/ci.mk
include makefiles/docs.mk
include makefiles/scripts.mk
include makefiles/path_contract.mk
include makefiles/registry.mk
include makefiles/help.mk
include makefiles/layout.mk
include makefiles/ops.mk
include makefiles/policies.mk

bootstrap:
	@python3 --version
	@command -v pip >/dev/null 2>&1 || { echo "missing pip" >&2; exit 1; }
	@python3 -m pip install -r configs/docs/requirements.txt >/dev/null
	@command -v k6 >/dev/null 2>&1 || echo "k6 not found (optional for non-perf workflows)"
	@command -v kind >/dev/null 2>&1 || echo "kind not found (required for k8s e2e)"
	@command -v kubectl >/dev/null 2>&1 || echo "kubectl not found (required for k8s e2e)"


docker-build:
	@IMAGE_TAG="$${DOCKER_IMAGE:-bijux-atlas:local}"; \
	IMAGE_VERSION="$${IMAGE_VERSION:-$$(git rev-parse --short=12 HEAD)}"; \
	VCS_REF="$${VCS_REF:-$$(git rev-parse HEAD)}"; \
	BUILD_DATE="$${BUILD_DATE:-$$(date -u +%Y-%m-%dT%H:%M:%SZ)}"; \
	RUST_VERSION="$${RUST_VERSION:-1.84.1}"; \
	docker build --pull=false -t "$$IMAGE_TAG" -f docker/Dockerfile \
	  --build-arg RUST_VERSION="$$RUST_VERSION" \
	  --build-arg IMAGE_VERSION="$$IMAGE_VERSION" \
	  --build-arg VCS_REF="$$VCS_REF" \
	  --build-arg BUILD_DATE="$$BUILD_DATE" \
	  --build-arg IMAGE_PROVENANCE="$${IMAGE_PROVENANCE:-$${IMAGE_TAG}}" \
	  .

docker-smoke:
	@$(MAKE) -s docker-build
	@./scripts/check/docker-runtime-smoke.sh "$${DOCKER_IMAGE:-bijux-atlas:local}"

docker-scan:
	@./scripts/check/docker-scan.sh "$${DOCKER_IMAGE:-bijux-atlas:local}"

docker-push:
	@if [ "$${CI:-0}" != "1" ]; then echo "docker-push is CI-only"; exit 2; fi
	@IMAGE_TAG="$${DOCKER_IMAGE:?DOCKER_IMAGE is required for docker-push}"; \
	docker push "$$IMAGE_TAG"

chart-package:
	@mkdir -p artifacts/chart
	@helm package ops/k8s/charts/bijux-atlas --destination artifacts/chart

chart-verify:
	@helm lint ops/k8s/charts/bijux-atlas
	@helm template atlas ops/k8s/charts/bijux-atlas >/dev/null

chart-validate: ## Validate chart via lint/template and values contract schema checks
	@$(MAKE) chart-verify
	@./scripts/contracts/generate_chart_values_schema.py
	@./scripts/contracts/check_chart_values_contract.py


docker-contracts: ## Validate Docker layout/policy/no-latest contracts
	@python3 ./scripts/check/check-docker-layout.py
	@python3 ./scripts/check/check-docker-policy.py
	@python3 ./scripts/check/check-no-latest-tags.py

rename-lint: ## Enforce durable naming rules for docs/scripts and concept ownership
	@python3 ./scripts/docs/check-durable-naming.py
	@./scripts/docs/check_duplicate_topics.sh

docs-lint-names: ## Enforce durable naming contracts, registries, and inventory
	@python3 ./scripts/docs/naming_inventory.py
	@./scripts/docs/ban_legacy_terms.sh
	@python3 ./scripts/docs/check_observability_docs_checklist.py
	@python3 ./scripts/docs/check_no_orphan_docs.py
	@python3 ./scripts/docs/check_script_locations.py
	@python3 ./scripts/docs/check_runbook_map_registration.py
	@python3 ./scripts/docs/check_contract_doc_pairs.py
	@python3 ./ops/load/scripts/validate_suite_manifest.py
	@./scripts/docs/check_index_pages.sh

doctor: ## Print tool/env/path diagnostics and store doctor report
	@RUN_ID="$${RUN_ID:-doctor-$(MAKE_RUN_TS)}" python3 ./scripts/layout/make_doctor.py

prereqs: ## Check required binaries and versions and store prereqs report
	@RUN_ID="$${RUN_ID:-prereqs-$(MAKE_RUN_TS)}" python3 ./scripts/layout/make_prereqs.py --run-id "$${RUN_ID:-prereqs-$(MAKE_RUN_TS)}"

dataset-id-lint: ## Validate DatasetId/DatasetKey contract usage across ops fixtures
	@python3 ./scripts/layout/dataset_id_lint.py

legacy/config-validate-core: ## Legacy config schema/contracts validation implementation
	@python3 ./scripts/public/generate-config-key-registry.py
	@python3 ./scripts/public/config-validate.py
	@python3 ./scripts/public/config-drift-check.py

config-print: ## Print canonical merged config payload as JSON
	@python3 ./scripts/public/config-print.py

config-drift: ## Check config/schema/docs drift without regeneration
	@python3 ./scripts/public/config-drift-check.py

configs-check: ## Config schemas + drift + ownership + symlink shim + SSOT checks
	@python3 ./scripts/configs/check_configs_readmes.py
	@python3 ./scripts/configs/check_config_ownership.py
	@python3 ./scripts/configs/validate_configs_schemas.py
	@python3 ./scripts/public/config-validate.py
	@python3 ./scripts/public/config-drift-check.py
	@python3 ./scripts/configs/check_ops_env_usage_declared.py
	@python3 ./scripts/configs/check_no_adhoc_versions.py
	@python3 ./scripts/configs/check_perf_thresholds_drift.py
	@python3 ./scripts/configs/check_slo_sync.py
	@python3 ./scripts/configs/check_openapi_snapshot_generated.py
	@python3 ./scripts/configs/check_tool_versions_doc_drift.py
	@python3 ./scripts/configs/check_root_config_shims.py
	@python3 ./scripts/layout/check_symlink_policy.py
	@python3 ./scripts/configs/check_duplicate_threshold_sources.py
	@python3 ./scripts/configs/check_docs_links_for_configs.py
	@python3 ./ops/_lint/no-shadow-configs.py

CI_ISO_ROOT := $(CURDIR)/artifacts/isolate/ci
CI_ENV := ISO_ROOT=$(CI_ISO_ROOT) CARGO_TARGET_DIR=$(CI_ISO_ROOT)/target CARGO_HOME=$(CI_ISO_ROOT)/cargo-home TMPDIR=$(CI_ISO_ROOT)/tmp TMP=$(CI_ISO_ROOT)/tmp TEMP=$(CI_ISO_ROOT)/tmp
LOCAL_ISO_ROOT := $(CURDIR)/artifacts/isolate/local
LOCAL_ENV := ISO_ROOT=$(LOCAL_ISO_ROOT) CARGO_TARGET_DIR=$(LOCAL_ISO_ROOT)/target CARGO_HOME=$(LOCAL_ISO_ROOT)/cargo-home TMPDIR=$(LOCAL_ISO_ROOT)/tmp TMP=$(LOCAL_ISO_ROOT)/tmp TEMP=$(LOCAL_ISO_ROOT)/tmp
LOCAL_FULL_ISO_ROOT := $(CURDIR)/artifacts/isolate/local-full
LOCAL_FULL_ENV := ISO_ROOT=$(LOCAL_FULL_ISO_ROOT) CARGO_TARGET_DIR=$(LOCAL_FULL_ISO_ROOT)/target CARGO_HOME=$(LOCAL_FULL_ISO_ROOT)/cargo-home TMPDIR=$(LOCAL_FULL_ISO_ROOT)/tmp TMP=$(LOCAL_FULL_ISO_ROOT)/tmp TEMP=$(LOCAL_FULL_ISO_ROOT)/tmp

gates-check: ## Run public-surface/docs/makefile boundary checks
	@$(call gate_json,public-surface,python3 ./scripts/layout/check_public_surface.py)
	@$(call gate_json,docs-public-surface,python3 ./scripts/docs/check_public_surface_docs.py)
	@$(call gate_json,suite-id-docs,python3 ./scripts/docs/check_suite_id_docs.py)
	@$(call gate_json,makefile-boundaries,python3 ./scripts/layout/check_makefile_target_boundaries.py)
	@$(call gate_json,public-target-budget,python3 ./scripts/layout/check_public_target_budget.py)
	@$(call gate_json,public-target-ownership,python3 ./scripts/layout/check_make_target_ownership.py)
	@$(call gate_json,public-target-docs,python3 ./scripts/layout/check_public_targets_documented.py)
	@$(call gate_json,public-target-descriptions,python3 ./scripts/layout/check_public_target_descriptions.py)
	@$(call gate_json,public-target-aliases,python3 ./scripts/layout/check_public_target_aliases.py)
	@$(call gate_json,internal-target-doc-refs,python3 ./scripts/layout/check_internal_targets_not_in_docs.py)
	@$(call gate_json,makefiles-contract,python3 ./scripts/layout/check_makefiles_contract.py)
	@$(call gate_json,makefiles-headers,python3 ./scripts/layout/check_makefile_headers.py)
	@$(call gate_json,makefiles-index-drift,python3 ./scripts/layout/check_makefiles_index_drift.py)
	@$(call gate_json,make-targets-catalog-drift,python3 ./scripts/layout/check_make_targets_catalog_drift.py)
	@$(call gate_json,cargo-dev-metadata,python3 ./scripts/layout/check_cargo_dev_metadata.py)
	@$(call gate_json,root-no-cargo-dev-deps,python3 ./scripts/layout/check_root_no_cargo_dev_deps.py)
	@$(call gate_json,cargo-invocation-scope,python3 ./scripts/layout/check_cargo_invocations_scoped.py)
	@$(call gate_json,root-diff-alarm,python3 ./scripts/layout/check_root_diff_alarm.py)
	@$(call gate_json,ci-entrypoints,python3 ./scripts/layout/check_ci_entrypoints.py)
	@$(call gate_json,help-excludes-internal,python3 ./scripts/layout/check_help_excludes_internal.py)
	@$(call gate_json,root-makefile-hygiene,python3 ./scripts/layout/check_root_makefile_hygiene.py)

gates: ## Print public targets grouped by namespace
	@python3 ./scripts/layout/render_public_help.py --mode gates

help: ## Show curated public make targets from SSOT
	@python3 ./scripts/layout/render_public_help.py

explain: ## Explain whether TARGET is a public make target
	@[ -n "$${TARGET:-}" ] || { echo "usage: make explain TARGET=<name>" >&2; exit 2; }
	@python3 ./scripts/layout/explain_public_target.py "$${TARGET}"

list: ## Print public make target set from SSOT with one-line descriptions
	@python3 ./scripts/layout/render_public_help.py --mode list

graph: ## Print compact dependency graph for TARGET
	@[ -n "$${TARGET:-}" ] || { echo "usage: make graph TARGET=<name>" >&2; exit 2; }
	@python3 ./scripts/layout/graph_public_target.py "$${TARGET}"

internal-list: ## Print internal make targets for maintainers
	@python3 ./scripts/layout/list_internal_targets.py

format: ## UX alias for fmt
	@$(MAKE) fmt

report/merge: ## Merge lane reports into unified make report JSON
	@run_id="$${RUN_ID:-$$(cat ops/_generated/root-local/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	python3 ./scripts/layout/make_report.py merge --run-id "$$run_id"

report/print: ## Print lane summary like CI/GitHub Actions output
	@run_id="$${RUN_ID:-$$(cat ops/_generated/root-local/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	python3 ./scripts/layout/make_report.py print --run-id "$$run_id"

report/md: ## Generate markdown summary for PR comments
	@run_id="$${RUN_ID:-$$(cat ops/_generated/root-local/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	python3 ./scripts/layout/make_report.py md --run-id "$$run_id"

report/junit: ## Optional JUnit conversion for CI systems
	@run_id="$${RUN_ID:-$$(cat ops/_generated/root-local/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	python3 ./scripts/layout/make_report.py junit --run-id "$$run_id"

report: ## Compatibility alias for report/merge
	@$(MAKE) -s report/merge

quick: ## Minimal tight loop (fmt + lint + test)
	@$(call with_iso,quick,$(MAKE) -s cargo/fmt cargo/lint cargo/test-fast)

cargo/all: ## Local exhaustive Rust lane
	@$(MAKE) -s lane-cargo

cargo/fmt: ## Cargo lane fmt
	@$(call with_iso,cargo-fmt,$(MAKE) -s fmt)

cargo/lint: ## Cargo lane lint
	@$(call with_iso,cargo-lint,$(MAKE) -s lint)

cargo/test-fast: ## Cargo fast unit-focused tests (nextest fast profile)
	@$(call with_iso,cargo-test-fast,NEXTEST_PROFILE=fast-unit $(MAKE) -s test)

cargo/test: ## Cargo CI-profile tests
	@$(call with_iso,cargo-test,NEXTEST_PROFILE=ci $(MAKE) -s test)

cargo/test-all: ## Cargo full nextest (including ignored)
	@$(call with_iso,cargo-test-all,NEXTEST_PROFILE=ci $(MAKE) -s test-all)

cargo/test-contracts: ## Cargo contract-focused tests
	@$(call with_iso,cargo-test-contracts,$(MAKE) -s test-contracts)

cargo/audit: ## Cargo audit gate
	@$(call with_iso,cargo-audit,$(MAKE) -s audit)

cargo/bench-smoke: ## Cargo benchmark smoke lane
	@$(call with_iso,cargo-bench-smoke,$(MAKE) -s bench-smoke)

cargo/coverage: ## Cargo coverage lane (kept out of root gate)
	@$(call with_iso,cargo-coverage,NEXTEST_PROFILE=ci $(MAKE) -s coverage)

docs/check: ## Fast docs verification
	@$(call with_iso,docs-check,$(MAKE) -s internal/docs/check)

docs/build: ## Build docs artifacts
	@$(call with_iso,docs-build,$(MAKE) -s internal/docs/build)

docs/fmt: ## Docs formatting helpers
	@$(call with_iso,docs-fmt,$(MAKE) -s internal/docs/fmt)

docs/lint: ## Docs lint checks
	@$(call with_iso,docs-lint,$(MAKE) -s internal/docs/lint)

docs/test: ## Docs tests
	@$(call with_iso,docs-test,$(MAKE) -s internal/docs/test)

docs/clean: ## Clean docs generated outputs only
	@$(call with_iso,docs-clean,$(MAKE) -s internal/docs/clean)

docs/all: ## Docs lane
	@$(call with_iso,docs-all,$(MAKE) -s internal/docs/all)

scripts/check: ## Deterministic scripts check lane
	@$(call with_iso,scripts-check,$(MAKE) -s internal/scripts/check)

scripts/build: ## Build script inventories and graph
	@$(call with_iso,scripts-build,$(MAKE) -s internal/scripts/build)

scripts/fmt: ## Scripts formatting
	@$(call with_iso,scripts-fmt,$(MAKE) -s internal/scripts/fmt)

scripts/lint: ## Scripts lint
	@$(call with_iso,scripts-lint-uniform,$(MAKE) -s internal/scripts/lint)

scripts/test: ## Scripts tests
	@$(call with_iso,scripts-test-uniform,$(MAKE) -s internal/scripts/test)

scripts/clean: ## Scripts generated-output cleanup
	@$(call with_iso,scripts-clean-uniform,$(MAKE) -s internal/scripts/clean)

scripts/all: ## Scripts lane (lint/tests/audit)
	@$(call with_iso,scripts-all,$(MAKE) -s internal/scripts/all)

ops/check: ## Fast ops verification (no cluster bring-up)
	@$(call with_iso,ops-check,$(MAKE) -s internal/ops/check)

ops/contract-check: ## Validate layer contract SSOT and drift/report gates
	@$(call with_iso,ops-contract-check,$(MAKE) -s ops-contract-check)

ops/smoke: ## Explicit ops smoke target
	@$(call with_iso,ops-smoke,$(MAKE) -s internal/ops/smoke)

ops/suite: ## Explicit ops suite target
	@$(call with_iso,ops-suite,$(MAKE) -s internal/ops/suite)

ops/fmt: ## Ops formatting
	@$(call with_iso,ops-fmt,$(MAKE) -s internal/ops/fmt)

ops/lint: ## Ops lint
	@$(call with_iso,ops-lint-uniform,$(MAKE) -s internal/ops/lint)

ops/test: ## Ops tests
	@$(call with_iso,ops-test,$(MAKE) -s internal/ops/test)

ops/build: ## Ops build/generated outputs
	@$(call with_iso,ops-build,$(MAKE) -s internal/ops/build)

ops/clean: ## Ops generated-output cleanup
	@$(call with_iso,ops-clean-uniform,$(MAKE) -s internal/ops/clean)

ops/all: ## Ops lane (lint + check + smoke)
	@$(call with_iso,ops-all,$(MAKE) -s internal/ops/all)

configs/check: ## Validate all config schemas + drift checks
	@$(MAKE) -s lane-configs-policies

configs/all: ## Configs lane (schema + drift checks)
	@$(MAKE) -s configs/check

policies/check: ## Run deny/audit + policy-relaxation checks
	@$(MAKE) -s lane-configs-policies

policies/all: ## Policies lane (deny/audit/policy checks)
	@$(MAKE) -s policies/check

policies/boundaries-check: ## Enforce e2e layer boundary rules and relaxations
	@python3 ./ops/_lint/layer-relaxations-audit.py
	@python3 ./ops/_lint/no-layer-fixups.py

local/all: ## Run all meaningful local gates
	@PARALLEL="$${PARALLEL:-1}" RUN_ID="$${RUN_ID:-$${MAKE_RUN_ID:-local-all-$(MAKE_RUN_TS)}}" MODE=root-local ./ops/run/root-lanes.sh

ci/all: ## Deterministic CI superset
	@$(call with_iso,ci-all,$(MAKE) -s gates-check lane-cargo lane-docs lane-scripts lane-configs-policies lane-ops ci-release-binaries ci-docs-build ci-release-compat-matrix-verify)

nightly/all: ## Slow nightly suites (perf/load/drills/realdata)
	@$(call with_iso,nightly-all,$(MAKE) -s ci/all ops-load-nightly ops-drill-suite ops-realdata)

lane-cargo: ## Lane: rust checks/tests in isolated lane-cargo path
	@$(MAKE) -s cargo/fmt cargo/lint
	@$(call with_iso,lane-cargo,NEXTEST_PROFILE=ci $(MAKE) -s check test test-all audit)

lane-docs: ## Lane: docs build/freeze/hardening
	@$(MAKE) -s docs/check

lane-ops: ## Lane: ops lint/contracts without cluster bring-up
	@$(MAKE) -s ops/check

lane-scripts: ## Lane: scripts lint/tests/audit
	@$(MAKE) -s scripts/check

lane-configs-policies: ## Lane: configs + policy checks
	@$(call with_iso,lane-configs-policies,$(MAKE) -s configs-check ci-deny policy-lint policy-schema-drift policy-audit policy-enforcement-status policy-allow-env-lint policies/boundaries-check)

internal/lane-ops-smoke: ## Internal lane: bounded ops smoke path
	@$(call with_iso,internal-lane-ops-smoke,$(MAKE) -s ops-k8s-smoke)

root: ## CI-fast lane subset (no cluster bring-up)
	@PARALLEL="$${PARALLEL:-1}" RUN_ID="$${RUN_ID:-$${MAKE_RUN_ID:-root-$(MAKE_RUN_TS)}}" MODE=root ./ops/run/root-lanes.sh

root-local: ## All lanes in parallel + ops smoke lane (PARALLEL=0 for serial)
	@PARALLEL="$${PARALLEL:-1}" RUN_ID="$${RUN_ID:-$${MAKE_RUN_ID:-root-local-$(MAKE_RUN_TS)}}" MODE=root-local ./ops/run/root-lanes.sh

root-local-fast: ## Debug alias for serial root-local execution
	@PARALLEL=0 $(MAKE) -s root-local

root-local-open: ## Open or print latest root-local summary report
	@SUMMARY_RUN_ID="$${RUN_ID:-}" MODE=open ./ops/run/root-lanes.sh

repro: ## Re-run one lane deterministically (usage: make repro TARGET=lane-cargo SEED=123)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make repro TARGET=<lane-target> [SEED=123]"; exit 2; }
	@seed="$${SEED:-0}"; \
	echo "repro target=$${TARGET} seed=$${seed}"; \
	TZ=UTC LANG=C.UTF-8 LC_ALL=C.UTF-8 TEST_RANDOM_SEED="$$seed" ATLAS_TEST_SEED="$$seed" $(MAKE) -s "$${TARGET}"

retry: ## Retry a target with same RUN_ID (usage: make retry TARGET=<target>)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make retry TARGET=<target>"; exit 2; }
	@run_id="$${RUN_ID:-$$(cat ops/_generated/root-local/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	echo "retry target=$${TARGET} run_id=$$run_id"; \
	RUN_ID="$$run_id" QUIET="$${QUIET:-0}" $(MAKE) -s "$${TARGET}"

local: ## Deprecated alias for quick
	@echo "[DEPRECATED] 'make local' -> 'make quick'" >&2
	@$(MAKE) -s quick

local-full: ## Deprecated alias for local/all
	@echo "[DEPRECATED] 'make local-full' -> 'make local/all'" >&2
	@$(MAKE) -s local/all

contracts: ## Deprecated alias for policies/all
	@echo "[DEPRECATED] 'make contracts' -> 'make policies/all'" >&2
	@$(MAKE) -s policies/all

hygiene: ## Deprecated alias for scripts/all
	@echo "[DEPRECATED] 'make hygiene' -> 'make scripts/all'" >&2
	@$(MAKE) -s scripts/all

config-validate: ## Deprecated alias for configs/all
	@echo "[DEPRECATED] 'make config-validate' -> 'make configs/all'" >&2
	@$(MAKE) -s configs/all

ci: ## Deprecated alias for ci/all
	@echo "[DEPRECATED] 'make ci' -> 'make ci/all'" >&2
	@$(MAKE) -s ci/all

nightly: ## Deprecated alias for nightly/all
	@echo "[DEPRECATED] 'make nightly' -> 'make nightly/all'" >&2
	@$(MAKE) -s nightly/all

legacy/root-fast: ## Legacy preserved deterministic lane
	@$(call with_iso,root,$(MAKE) -s gates-check configs/all lane-cargo ci-deny ops-contracts-check docs-lint-names)

legacy/root-local-full: ## Legacy local superset gate with 5 parallel isolated lanes + unified summary
	@PARALLEL=1 MODE=root-local RUN_ID="$${RUN_ID:-legacy-root-local-$(MAKE_RUN_TS)}" ./ops/run/root-lanes.sh

legacy/root-local-fast: ## Legacy local superset fast mode (skip stack-smoke lane)
	@PARALLEL=0 MODE=root RUN_ID="$${RUN_ID:-legacy-root-fast-$(MAKE_RUN_TS)}" ./ops/run/root-lanes.sh

root-local-summary: ## Print status and artifact paths for RUN_ID
	@SUMMARY_RUN_ID="$${RUN_ID:-}" MODE=summary ./ops/run/root-lanes.sh

legacy/ci: ## Legacy root + CI-only packaging/publish checks
	@$(call with_iso,ci,$(MAKE) -s root ci-release-binaries ci-docs-build ci-release-compat-matrix-verify)

legacy/nightly: ## Legacy nightly superset (ci + nightly ops suites)
	@$(call with_iso,nightly,$(MAKE) -s ci ops-load-nightly ops-drill-suite ops-realdata)

root-determinism: ## Assert make root determinism (inventory outputs stable across two runs)
	@./scripts/layout/check_root_determinism.sh

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

telemetry-contracts: ## Regenerate telemetry generated artifacts from observability contracts
	@python3 ./scripts/contracts/generate_contract_artifacts.py
	@cargo fmt --all

telemetry-verify: ## Run telemetry contract verification path (pack+smoke+contract tests)
	@$(MAKE) telemetry-contracts
	@cargo test -p bijux-atlas-server --test observability_contract
	@if [ "$${ATLAS_TELEMETRY_VERIFY_LIVE:-0}" = "1" ]; then \
	  $(MAKE) observability-pack-test; \
	else \
	  $(MAKE) ops-observability-pack-lint; \
	  echo "live pack smoke skipped (set ATLAS_TELEMETRY_VERIFY_LIVE=1)"; \
	fi

legacy/hygiene: ## Legacy repo hygiene checks
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) layout-check
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) scripts-audit
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) ci-workflows-make-only
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) ci-make-help-drift
	@ISO_ROOT=artifacts/isolate/hygiene $(MAKE) path-contract-check

architecture-check: ## Validate runtime architecture boundaries and dependency guardrails
	@python3 scripts/docs/generate_architecture_map.py
	@if ! git diff --quiet -- docs/architecture/architecture-map.md; then \
		echo "architecture map drift detected; regenerate docs/architecture/architecture-map.md" >&2; \
		git --no-pager diff -- docs/architecture/architecture-map.md >&2 || true; \
		exit 1; \
	fi
	@cargo test -p bijux-atlas-core --test guardrails crate_dependency_dag_matches_boundaries_doc -- --exact
	@cargo test -p bijux-atlas-core --test guardrails server_must_not_depend_on_ingest_crate -- --exact
	@cargo test -p bijux-atlas-core --test guardrails query_layer_must_not_depend_on_runtime_network_or_async_stacks -- --exact
	@cargo test -p bijux-atlas-core --test guardrails server_http_layers_must_not_read_raw_files_directly -- --exact
	@cargo test -p bijux-atlas-server --test import_boundary_guardrails

fetch-real-datasets:
	@./scripts/fixtures/fetch-real-datasets.sh

ssot-check:
	@./scripts/contracts/check_all.sh

policy-lint:
	@./scripts/public/policy-lint.sh

policy-schema-drift:
	@./scripts/public/policy-schema-drift.py

policy-audit: ## Audit policy relaxations report + enforce registry/expiry/budget gates
	@./scripts/public/policy-audit.py --enforce

policy-enforcement-status: ## Validate policy pass/fail coverage table and generate status doc
	@./scripts/public/policy-enforcement-status.py --enforce

policy-allow-env-lint: ## Forbid ALLOW_* escape hatches unless declared in env schema
	@./scripts/public/check-allow-env-schema.py

ops-policy-audit: ## Verify ops policy configs are reflected by ops make/scripts contracts
	@./scripts/public/ops-policy-audit.py

policy-drift-diff: ## Show policy contract drift between two refs (usage: make policy-drift-diff [FROM=HEAD~1 TO=HEAD])
	@./scripts/public/policy-drift-diff.sh "$${FROM:-HEAD~1}" "$${TO:-HEAD}"

release-update-compat-matrix:
	@[ -n "$$TAG" ] || { echo "usage: make release-update-compat-matrix TAG=<tag>"; exit 2; }
	@./scripts/release/update-compat-matrix.sh "$$TAG"


.PHONY: architecture-check artifacts-clean artifacts-index bootstrap bootstrap-tools bump cargo/all chart-package chart-verify ci ci/all ci-workflow-contract clean config-drift config-print config-validate configs-check configs/all contracts dataset-id-lint debug deep-clean docker-build docker-contracts docker-push docker-scan docker-smoke docs docs/all docs-lint-names doctor explain fetch-real-datasets format gates gates-check governance-check graph help hygiene internal-list inventory isolate-clean layout-check layout-migrate legacy/ci legacy/contracts legacy/hygiene legacy/local-fast-loop legacy/local-full-loop legacy/nightly legacy/root-fast legacy/root-local-fast legacy/root-local-full list local local/all local-full makefiles-contract nightly nightly/all no-direct-scripts ops-alerts-validate ops/all ops-artifacts-open ops-baseline-policy-check ops-cache-pin-set ops-cache-status ops-catalog-validate ops-check ops-clean ops-contracts-check ops-dashboards-validate ops-dataset-federated-registry-test ops-dataset-multi-release-test ops-dataset-promotion-sim ops-dataset-qc ops-datasets-fetch ops-deploy ops-doctor ops-down ops-drill-corruption-dataset ops-drill-memory-growth ops-drill-otel-outage ops-drill-overload ops-drill-pod-churn ops-drill-rate-limit ops-drill-rollback ops-drill-rollback-under-load ops-drill-store-outage ops-drill-suite ops-drill-toxiproxy-latency ops-drill-upgrade ops-drill-upgrade-under-load ops-e2e ops-e2e-smoke ops-full ops-full-pr ops-gc-smoke ops-gen ops-gen-check ops-incident-repro-kit ops-k8s-smoke ops-k8s-suite ops-k8s-template-tests ops-k8s-tests ops-load-ci ops-load-full ops-load-manifest-validate ops-load-nightly ops-load-shedding ops-load-smoke ops-load-soak ops-load-suite ops-local-full ops-local-full-stack ops-metrics-check ops-obs-down ops-obs-install ops-obs-mode ops-obs-uninstall ops-obs-verify ops-observability-pack-conformance-report ops-observability-pack-export ops-observability-pack-health ops-observability-pack-smoke ops-observability-pack-verify ops-observability-smoke ops-observability-validate ops-open-grafana ops-openapi-validate ops-perf-baseline-update ops-perf-cold-start ops-perf-nightly ops-perf-report ops-perf-warm-start ops-policy-audit ops-prereqs ops-proof-cached-only ops-publish ops-readiness-scorecard ops-realdata ops-redeploy ops-ref-grade-local ops-ref-grade-nightly ops-ref-grade-pr ops-release-matrix ops-release-rollback ops-release-update ops-report ops-slo-alert-proof ops-slo-burn ops-slo-report ops-smoke ops-tools-check ops-traces-check ops-undeploy ops-up ops-values-validate ops-warm ops-warm-datasets ops-warm-shards ops-warm-top policies/all policy-allow-env-lint policy-audit policy-drift-diff policy-enforcement-status policy-lint policy-schema-drift prereqs quick release release-dry-run release-update-compat-matrix rename-lint report root root-determinism root-local root-local-fast root-local-summary scripts-all scripts/all scripts-audit scripts-check scripts-clean scripts-format scripts-graph scripts-index scripts-lint scripts-test ssot-check verify-inventory lane-cargo lane-docs lane-ops lane-scripts lane-configs-policies root-local-open repro internal/lane-ops-smoke legacy/config-validate-core report/merge report/print report/md report/junit clean-safe clean-all print-env cargo/fmt cargo/lint cargo/test-fast cargo/test cargo/test-all cargo/test-contracts cargo/audit cargo/bench-smoke cargo/coverage configs/check policies/check policies/boundaries-check retry docs/check docs/build docs/fmt docs/lint docs/test docs/clean scripts/check scripts/build scripts/fmt scripts/lint scripts/test scripts/clean ops/check ops/smoke ops/suite ops/fmt ops/lint ops/test ops/build ops/clean



inventory: ## Regenerate inventories (ops surface, make targets, docs status, naming, repo surface)
	@python3 ./scripts/docs/generate_make_targets_catalog.py
	@python3 ./scripts/docs/generate_ops_surface.py
	@python3 ./scripts/docs/generate_make_targets_inventory.py
	@python3 ./scripts/docs/generate_makefiles_surface.py
	@python3 ./scripts/gen/generate_scripts_surface.py
	@python3 ./scripts/configs/generate_configs_surface.py
	@python3 ./scripts/configs/generate_tooling_versions_doc.py
	@python3 ./scripts/docs/lint_doc_status.py
	@python3 ./scripts/docs/naming_inventory.py
	@python3 ./scripts/docs/generate_repo_surface.py

verify-inventory: ## Fail if inventory outputs drift from generated state
	@$(MAKE) -s inventory
	@git diff --exit-code -- makefiles/targets.json docs/_generated/make-targets.md docs/_generated/repo-surface.md docs/_generated/doc-status.md docs/_generated/naming-inventory.md docs/_generated/ops-surface.md docs/_generated/configs-surface.md docs/_generated/tooling-versions.md docs/_generated/scripts-surface.md docs/development/make-targets.md docs/development/make-targets-inventory.md docs/development/makefiles/surface.md

artifacts-index: ## Generate artifacts index for inspection UIs
	@python3 ./scripts/layout/build_artifacts_index.py

artifacts-clean: ## Clean old artifacts with safe retention
	@python3 ./scripts/layout/clean_artifacts.py

isolate-clean: ## Remove isolate output directories safely
	@find artifacts/isolate -mindepth 1 -maxdepth 1 -type d -exec rm -r {} + 2>/dev/null || true

clean: ## Safe clean for generated local outputs
	@./ops/run/clean.sh

clean-safe: ## Clean only safe generated make artifact directories
	@python3 ./scripts/layout/clean_make_artifacts.py

clean-all: ## Clean all allowed generated dirs (requires CONFIRM=YES)
	@[ "$${CONFIRM:-}" = "YES" ] || { echo "refusing clean-all without CONFIRM=YES"; exit 2; }
	@python3 ./scripts/layout/clean_make_artifacts.py --all

deep-clean: ## Extended clean (prints and then removes generated outputs)
	@printf '%s\n' 'Deleting: artifacts/isolate artifacts/scripts artifacts/perf/results artifacts/ops'
	@$(MAKE) clean
	@rm -rf artifacts/ops

debug: ## Print internal make/env variables for maintainers
	@printf 'MAKE_PRIMARY_GOAL=%s\n' "$(MAKE_PRIMARY_GOAL)"
	@printf 'MAKE_RUN_ID=%s\n' "$(MAKE_RUN_ID)"
	@printf 'ISO_ROOT=%s\n' "$(ISO_ROOT)"
	@printf 'CARGO_TARGET_DIR=%s\n' "$(CARGO_TARGET_DIR)"
	@printf 'ATLAS_NS=%s\n' "$(ATLAS_NS)"
	@printf 'ATLAS_BASE_URL=%s\n' "$(ATLAS_BASE_URL)"
	@printf 'OPS_RUN_ID=%s\n' "$(OPS_RUN_ID)"

print-env: ## Print key env vars used by lanes and gates
	@printf 'RUN_ID=%s\n' "$${RUN_ID:-}"
	@printf 'ISO_ROOT=%s\n' "$${ISO_ROOT:-}"
	@printf 'ISO_RUN_ID=%s\n' "$${ISO_RUN_ID:-}"
	@printf 'ISO_TAG=%s\n' "$${ISO_TAG:-}"
	@printf 'CARGO_TARGET_DIR=%s\n' "$${CARGO_TARGET_DIR:-}"
	@printf 'CARGO_HOME=%s\n' "$${CARGO_HOME:-}"
	@printf 'TMPDIR=%s\n' "$${TMPDIR:-}"
	@printf 'TMP=%s\n' "$${TMP:-}"
	@printf 'TEMP=%s\n' "$${TEMP:-}"
	@printf 'TZ=%s\n' "$${TZ:-}"
	@printf 'LANG=%s\n' "$${LANG:-}"
	@printf 'LC_ALL=%s\n' "$${LC_ALL:-}"
	@printf 'ATLAS_BASE_URL=%s\n' "$${ATLAS_BASE_URL:-}"
	@printf 'ATLAS_NS=%s\n' "$${ATLAS_NS:-}"

bump: ## Bump workspace version (usage: make bump VERSION=x.y.z)
	@[ -n "$$VERSION" ] || { echo "usage: make bump VERSION=x.y.z"; exit 2; }
	@cargo set-version --workspace "$$VERSION"

release-dry-run: ## Build + docs + ops smoke release rehearsal
	@ISO_ROOT=artifacts/isolate/release-dry-run $(MAKE) fmt
	@ISO_ROOT=artifacts/isolate/release-dry-run $(MAKE) lint
	@ISO_ROOT=artifacts/isolate/release-dry-run $(MAKE) policy-audit
	@ISO_ROOT=artifacts/isolate/release-dry-run $(MAKE) test
	@ISO_ROOT=artifacts/isolate/release-dry-run $(MAKE) docs
	@ISO_ROOT=artifacts/isolate/release-dry-run $(MAKE) ops-full-pr

release: ## Release entrypoint (currently dry-run only)
	@$(MAKE) release-dry-run
makefiles-contract: ## Validate makefile contract boundaries and publication rules
	@python3 ./scripts/layout/check_makefiles_contract.py

ci-workflow-contract: ## Validate CI and nightly workflows use canonical make entrypoints
	@python3 ./scripts/layout/check_ci_entrypoints.py
