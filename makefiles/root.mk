SHELL := /bin/sh
JOBS  ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/env.mk
include makefiles/_macros.mk
include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/ci.mk
include makefiles/docs.mk
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

bootstrap-tools:
	@./scripts/bootstrap/install_tools.sh

scripts-index:
	@python3 ./scripts/generate_scripts_readme.py

scripts-graph: ## Generate make-target to scripts call graph
	@python3 ./scripts/docs/generate_scripts_graph.py

docker-build:
	@docker build -t bijux-atlas:local -f docker/Dockerfile .

docker-smoke:
	@docker run --rm bijux-atlas:local --version >/dev/null
	@echo "docker smoke passed"

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

no-direct-scripts:
	@./scripts/layout/check_no_direct_script_runs.sh
	@python3 ./scripts/layout/check_make_public_scripts.py

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

doctor: ## Public doctor target (delegates to ops/run/doctor.sh)
	@./ops/run/doctor.sh

prereqs: ## Public prereqs target (delegates to ops/run/prereqs.sh)
	@./ops/run/prereqs.sh

dataset-id-lint: ## Validate DatasetId/DatasetKey contract usage across ops fixtures
	@python3 ./scripts/layout/dataset_id_lint.py

config-validate: ## Validate config schemas/contracts and regenerate config key registry
	@python3 ./scripts/public/generate-config-key-registry.py
	@python3 ./scripts/public/config-validate.py
	@python3 ./scripts/public/config-drift-check.py

config-print: ## Print canonical merged config payload as JSON
	@python3 ./scripts/public/config-print.py

config-drift: ## Check config/schema/docs drift without regeneration
	@python3 ./scripts/public/config-drift-check.py

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
	@$(call gate_json,makefiles-contract,python3 ./scripts/layout/check_makefiles_contract.py)
	@$(call gate_json,ci-entrypoints,python3 ./scripts/layout/check_ci_entrypoints.py)
	@$(call gate_json,help-excludes-internal,python3 ./scripts/layout/check_help_excludes_internal.py)
	@$(call gate_json,root-makefile-hygiene,python3 ./scripts/layout/check_root_makefile_hygiene.py)

gates: ## Print curated root/ci/nightly gates and core public targets
	@python3 ./scripts/layout/render_public_help.py --mode gates

explain: ## Explain whether TARGET is a public make target
	@[ -n "$${TARGET:-}" ] || { echo "usage: make explain TARGET=<name>" >&2; exit 2; }
	@python3 ./scripts/layout/explain_public_target.py "$${TARGET}"

list-public: ## Print public make target set from SSOT
	@python3 ./scripts/layout/render_public_help.py --mode list-public

list-internal: ## Print non-public make targets for maintainers
	@python3 ./scripts/layout/list_internal_targets.py

format: ## UX alias for fmt
	@$(MAKE) fmt

report: ## Merge lane reports into unified report
	@./ops/run/report.sh

root: ## Deterministic CI-fast lane (fmt/lint/tests/audit/contracts/docs-lint)
	@$(call with_iso,root,$(MAKE) -s gates-check config-validate fmt lint test audit ci-deny ops-contracts-check docs-lint-names)

root-local: ## Local superset gate with 5 parallel isolated lanes + unified summary
	@./ops/run/root-local.sh

root-local-fast: ## Local superset fast mode (skip stack-smoke lane)
	@./ops/run/root-local.sh --fast

root-local-summary: ## Print status and artifact paths for RUN_ID
	@./ops/run/root-local.sh --summary "$${RUN_ID:-}"

ci: ## root + CI-only packaging/publish checks
	@$(call with_iso,ci,$(MAKE) -s root ci-release-binaries ci-docs-build ci-release-compat-matrix-verify)

nightly: ## Nightly superset (ci + nightly ops suites)
	@$(call with_iso,nightly,$(MAKE) -s ci ops-load-nightly ops-drill-suite ops-realdata)

root-determinism: ## Assert make root determinism (inventory outputs stable across two runs)
	@./scripts/layout/check_root_determinism.sh

local: ## Fast local loop (fmt + lint + test)
	@mkdir -p "$(LOCAL_ISO_ROOT)/target" "$(LOCAL_ISO_ROOT)/cargo-home" "$(LOCAL_ISO_ROOT)/tmp"
	@$(LOCAL_ENV) $(MAKE) fmt
	@$(LOCAL_ENV) $(MAKE) lint
	@$(LOCAL_ENV) $(MAKE) test

local-full: ## Full local loop (fmt + lint + audit + test + coverage + docs)
	@mkdir -p "$(LOCAL_FULL_ISO_ROOT)/target" "$(LOCAL_FULL_ISO_ROOT)/cargo-home" "$(LOCAL_FULL_ISO_ROOT)/tmp"
	@$(LOCAL_FULL_ENV) $(MAKE) fmt
	@$(LOCAL_FULL_ENV) $(MAKE) lint
	@$(LOCAL_FULL_ENV) $(MAKE) audit
	@$(LOCAL_FULL_ENV) $(MAKE) test
	@$(LOCAL_FULL_ENV) $(MAKE) coverage
	@$(LOCAL_FULL_ENV) $(MAKE) docs
	@$(LOCAL_FULL_ENV) $(MAKE) docs-freeze

contracts: ## Contracts meta pipeline (generate + format + drift checks)
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

hygiene: ## Repo hygiene checks (layout + symlink + tracked-noise gates)
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



.PHONY: architecture-check artifacts-clean artifacts-index bootstrap bootstrap-tools bump chart-package chart-verify ci ci-workflow-contract clean config-drift config-print config-validate contracts dataset-id-lint debug deep-clean docker-build docker-smoke docs docs-lint-names doctor explain fetch-real-datasets format gates gates-check governance-check help hygiene inventory isolate-clean layout-check layout-migrate list-internal list-public local local-full makefiles-contract nightly no-direct-scripts ops-alerts-validate ops-artifacts-open ops-baseline-policy-check ops-cache-pin-set ops-cache-status ops-catalog-validate ops-check ops-clean ops-contracts-check ops-dashboards-validate ops-dataset-federated-registry-test ops-dataset-multi-release-test ops-dataset-promotion-sim ops-dataset-qc ops-datasets-fetch ops-deploy ops-doctor ops-down ops-drill-corruption-dataset ops-drill-memory-growth ops-drill-otel-outage ops-drill-overload ops-drill-pod-churn ops-drill-rate-limit ops-drill-rollback ops-drill-rollback-under-load ops-drill-store-outage ops-drill-suite ops-drill-toxiproxy-latency ops-drill-upgrade ops-drill-upgrade-under-load ops-e2e-smoke ops-full ops-full-pr ops-gc-smoke ops-gen ops-gen-check ops-incident-repro-kit ops-k8s-smoke ops-k8s-suite ops-k8s-template-tests ops-k8s-tests ops-load-ci ops-load-full ops-load-manifest-validate ops-load-nightly ops-load-shedding ops-load-smoke ops-load-soak ops-load-suite ops-local-full ops-local-full-stack ops-metrics-check ops-obs-down ops-obs-install ops-obs-mode ops-obs-uninstall ops-obs-verify ops-observability-pack-conformance-report ops-observability-pack-export ops-observability-pack-health ops-observability-pack-smoke ops-observability-pack-verify ops-observability-smoke ops-observability-validate ops-open-grafana ops-openapi-validate ops-perf-baseline-update ops-perf-cold-start ops-perf-nightly ops-perf-report ops-perf-warm-start ops-policy-audit ops-prereqs ops-proof-cached-only ops-publish ops-readiness-scorecard ops-realdata ops-redeploy ops-ref-grade-local ops-ref-grade-nightly ops-ref-grade-pr ops-release-matrix ops-release-rollback ops-release-update ops-report ops-slo-alert-proof ops-slo-burn ops-slo-report ops-smoke ops-tools-check ops-traces-check ops-undeploy ops-up ops-values-validate ops-warm ops-warm-datasets ops-warm-shards ops-warm-top policy-allow-env-lint policy-audit policy-drift-diff policy-enforcement-status policy-lint policy-schema-drift prereqs release release-dry-run release-update-compat-matrix rename-lint report root root-determinism root-local root-local-fast root-local-summary scripts-audit scripts-clean scripts-format scripts-graph scripts-index scripts-lint scripts-test ssot-check verify-inventory
scripts-lint: ## Lint script surface (shellcheck + header + make/public gate + optional ruff)
	@$(MAKE) scripts-audit
	@python3 ./scripts/docs/check_script_headers.py
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./scripts/layout/check_scripts_buckets.py
	@python3 ./scripts/layout/check_script_relative_calls.py
	@python3 ./scripts/layout/check_script_naming_convention.py
	@python3 ./scripts/layout/check_no_mixed_script_name_variants.py
	@python3 ./scripts/layout/check_duplicate_script_intent.py
	@./ops/_lint/naming.sh
	@python3 ./ops/_lint/no-shadow-configs.py
	@python3 ./scripts/layout/check_public_entrypoint_cap.py
	@SHELLCHECK_STRICT=1 $(MAKE) -s ops-shellcheck
	@if command -v shellcheck >/dev/null 2>&1; then find scripts/public scripts/internal scripts/dev -type f -name '*.sh' -print0 | xargs -0 shellcheck --rcfile ./configs/shellcheck/shellcheckrc -x; else echo "shellcheck not installed (optional for local scripts lint)"; fi
	@if command -v shfmt >/dev/null 2>&1; then shfmt -d scripts ops/load/scripts; else echo "shfmt not installed (optional)"; fi
	@if command -v ruff >/dev/null 2>&1; then ruff check scripts ops/load/scripts; else echo "ruff not installed (optional)"; fi

inventory: ## Regenerate inventories (ops surface, make targets, docs status, naming, repo surface)
	@python3 ./scripts/docs/generate_ops_surface.py
	@python3 ./scripts/docs/generate_make_targets_inventory.py
	@python3 ./scripts/docs/generate_makefiles_surface.py
	@python3 ./scripts/docs/lint_doc_status.py
	@python3 ./scripts/docs/naming_inventory.py
	@python3 ./scripts/docs/generate_repo_surface.py

verify-inventory: ## Fail if inventory outputs drift from generated state
	@$(MAKE) -s inventory
	@git diff --exit-code -- docs/_generated/repo-surface.md docs/_generated/doc-status.md docs/_generated/naming-inventory.md docs/_generated/ops-surface.md docs/development/make-targets.md docs/development/make-targets-inventory.md docs/development/makefiles/surface.md

scripts-format: ## Format scripts (python + shell where available)
	@if command -v ruff >/dev/null 2>&1; then ruff format scripts; else echo "ruff not installed (optional)"; fi
	@if command -v shfmt >/dev/null 2>&1; then find scripts ops/load/scripts -type f -name '*.sh' -print0 | xargs -0 shfmt -w; else echo "shfmt not installed (optional)"; fi

scripts-test: ## Run scripts-focused tests
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./ops/load/scripts/validate_suite_manifest.py
	@python3 ./ops/load/scripts/check_pinned_queries_lock.py
	@python3 -m unittest scripts.tests.test_paths

scripts-audit: ## Audit script headers, taxonomy buckets, and no-implicit-cwd contract
	@python3 ./scripts/docs/check_script_headers.py
	@python3 ./scripts/layout/check_scripts_buckets.py
	@python3 ./scripts/layout/check_make_public_scripts.py
	@python3 ./scripts/layout/check_script_relative_calls.py

scripts-clean: ## Remove generated script artifacts
	@rm -rf artifacts/scripts


artifacts-index: ## Generate artifacts index for inspection UIs
	@python3 ./scripts/layout/build_artifacts_index.py

artifacts-clean: ## Clean old artifacts with safe retention
	@python3 ./scripts/layout/clean_artifacts.py

isolate-clean: ## Remove isolate output directories safely
	@find artifacts/isolate -mindepth 1 -maxdepth 1 -type d -exec rm -r {} + 2>/dev/null || true

clean: ## Safe clean for generated local outputs
	@./ops/run/clean.sh

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
