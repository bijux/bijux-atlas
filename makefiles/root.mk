# Scope: top-level publication surface and orchestration for public make targets.
# Public targets: declared here; all other makefiles expose internal-only targets.
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS  ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/env.mk
include makefiles/python.mk
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
include makefiles/product.mk
include makefiles/ops.mk
include makefiles/policies.mk

check: ## Umbrella check: scripts package checks + cargo checks + make contracts
	@$(MAKE) -s check-scripts
	@$(MAKE) -s _check
	@$(SCRIPTS) check make-help
	@$(SCRIPTS) check repo
	@$(SCRIPTS) check forbidden-paths
	@$(SCRIPTS) check ops-generated-tracked
	@$(SCRIPTS) check tracked-timestamps
	@$(SCRIPTS) check committed-generated-hygiene
	@$(MAKE) -s make/guard-no-script-paths
	@$(MAKE) -s make/command-allowlist

check-scripts: ## Run scripts package lint/tests/contracts
	@$(MAKE) -s scripts-check

gen: ## Run deterministic generators through package CLI
	@$(SCRIPTS) gen contracts
	@$(SCRIPTS) gen make-targets
	@$(SCRIPTS) gen surface
	@$(SCRIPTS) gen scripting-surface

clean-scripts: ## Clean scripts artifacts via package CLI
	@$(SCRIPTS) clean

artifacts-gc: ## Garbage collect scripts artifacts retention policy
	@$(SCRIPTS) report artifact-gc

ci-local: ## Local runner mirroring CI top-level entrypoint set
	@$(MAKE) -s ci/all

doctor: ## Run package doctor diagnostics
	@$(SCRIPTS) doctor

make/command-allowlist: ## Enforce direct-make command allowlist (cargo/docker/helm/kubectl/k6)
	@$(SCRIPTS) check make-command-allowlist

config-print: ## Print canonical merged config payload as JSON
	@$(ATLAS_SCRIPTS) configs print

config-drift: ## Check config/schema/docs drift without regeneration
	@$(ATLAS_SCRIPTS) configs drift

configs-gen-check: ## Regenerate configs generated docs and fail on drift
	@$(ATLAS_SCRIPTS) configs generate --check

configs-check: ## Config schemas + drift + ownership + symlink shim + SSOT checks
	@$(ATLAS_SCRIPTS) configs validate --report text --emit-artifacts

CI_ISO_ROOT := $(CURDIR)/artifacts/isolate/ci
CI_ENV := ISO_ROOT=$(CI_ISO_ROOT) CARGO_TARGET_DIR=$(CI_ISO_ROOT)/target CARGO_HOME=$(CI_ISO_ROOT)/cargo-home TMPDIR=$(CI_ISO_ROOT)/tmp TMP=$(CI_ISO_ROOT)/tmp TEMP=$(CI_ISO_ROOT)/tmp
LOCAL_ISO_ROOT := $(CURDIR)/artifacts/isolate/local
LOCAL_ENV := ISO_ROOT=$(LOCAL_ISO_ROOT) CARGO_TARGET_DIR=$(LOCAL_ISO_ROOT)/target CARGO_HOME=$(LOCAL_ISO_ROOT)/cargo-home TMPDIR=$(LOCAL_ISO_ROOT)/tmp TMP=$(LOCAL_ISO_ROOT)/tmp TEMP=$(LOCAL_ISO_ROOT)/tmp
LOCAL_FULL_ISO_ROOT := $(CURDIR)/artifacts/isolate/local-full
LOCAL_FULL_ENV := ISO_ROOT=$(LOCAL_FULL_ISO_ROOT) CARGO_TARGET_DIR=$(LOCAL_FULL_ISO_ROOT)/target CARGO_HOME=$(LOCAL_FULL_ISO_ROOT)/cargo-home TMPDIR=$(LOCAL_FULL_ISO_ROOT)/tmp TMP=$(LOCAL_FULL_ISO_ROOT)/tmp TEMP=$(LOCAL_FULL_ISO_ROOT)/tmp

gates-check: ## Run public-surface/docs/makefile boundary checks
	@$(MAKE) -s internal/scripts/cli-check
	@$(ATLAS_SCRIPTS) make contracts-check --emit-artifacts

gates: ## Run curated root gate preset through atlasctl orchestrator
	@$(ATLAS_SCRIPTS) --quiet gates run --preset root --all --report text

gates-list: ## Print public targets grouped by namespace
	@$(ATLAS_SCRIPTS) --quiet gates list

help: ## Show curated public make targets from SSOT
	@$(ATLAS_SCRIPTS) make help

help-advanced: ## Show curated public targets plus maintainer-oriented helpers
	@$(ATLAS_SCRIPTS) make help --mode advanced

help-all:
	@$(ATLAS_SCRIPTS) make help --mode all

explain: ## Explain whether TARGET is a public make target
	@[ -n "$${TARGET:-}" ] || { echo "usage: make explain TARGET=<name>" >&2; exit 2; }
	@$(ATLAS_SCRIPTS) make explain "$${TARGET}"

list: ## Print public make target set from SSOT with one-line descriptions
	@$(ATLAS_SCRIPTS) make list

targets: ## Print generated target catalog from SSOT
	@$(ATLAS_SCRIPTS) make list

graph: ## Print compact dependency graph for TARGET
	@[ -n "$${TARGET:-}" ] || { echo "usage: make graph TARGET=<name>" >&2; exit 2; }
	@$(ATLAS_SCRIPTS) make graph "$${TARGET}"

what: ## Print explain + dependency graph for TARGET
	@[ -n "$${TARGET:-}" ] || { echo "usage: make what TARGET=<name>" >&2; exit 2; }
	@$(ATLAS_SCRIPTS) make explain "$${TARGET}"
	@echo ""
	@$(ATLAS_SCRIPTS) make graph "$${TARGET}"

internal-list: ## Print internal make targets for maintainers
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/docs/list_internal_targets.py

format: ## UX alias for fmt
	@$(MAKE) fmt

report/merge: ## Merge lane reports into unified make report JSON
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report collect --run-id "$$run_id"

report/print: ## Print lane summary like CI/GitHub Actions output
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report print --run-id "$$run_id"

report/md: ## Generate markdown summary for PR comments
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report summarize --run-id "$$run_id"

report/junit: ## Optional JUnit conversion for CI systems
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report junit --run-id "$$run_id"

report/bundle: ## Export evidence bundle archive for RUN_ID
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report bundle --run-id "$$run_id"

logs/last-fail: ## Tail the last failed lane log from latest unified report
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report last-fail --run-id "$$run_id"

triage: ## Print failing lanes + last 20 log lines + evidence paths
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report triage --run-id "$$run_id"

report: ## Build unified report and print one-screen summary
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(SCRIPTS) report unified --run-id "$$run_id" >/dev/null; \
	$(SCRIPTS) report print --run-id "$$run_id"

evidence/open: ## Open evidence directory (supports AREA=<area> RUN_ID=<id>)
	@$(ATLAS_SCRIPTS) artifacts open

evidence/clean: ## Clean evidence directories using retention policy
	@$(ATLAS_SCRIPTS) report artifact-gc

evidence-gc: ## Enforce evidence retention policy
	@$(ATLAS_SCRIPTS) report artifact-gc

evidence/check: ## Validate evidence JSON schema contract for generated outputs
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/artifacts/evidence_check.py

evidence/bundle: ## Export latest evidence bundle as tar.zst for CI attachments
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report bundle --run-id "$$run_id"

evidence/pr-summary: ## Generate PR markdown summary from latest evidence unified report
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	$(ATLAS_SCRIPTS) report pr-summary --run-id "$$run_id"

artifacts-open: ## Open latest ops artifact bundle/report directory
	@$(call with_iso,artifacts-open,$(MAKE) -s ops-artifacts-open)

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

tools-check: ## Alias for python tooling/package checks
	@$(MAKE) -s scripts/check

atlasctl-lint: ## Lint atlasctl package (ruff + mypy strict domains)
	@$(MAKE) -s internal/scripts/install-lock
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/ruff" check --config packages/atlasctl/pyproject.toml packages/atlasctl/src packages/atlasctl/tests
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/mypy" packages/atlasctl/src/atlasctl/core packages/atlasctl/src/atlasctl/contracts

atlasctl-test: ## Test atlasctl package (compile + unit + integration)
	@$(MAKE) -s internal/scripts/install-lock
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/python" -m compileall -q packages/atlasctl/src
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/pytest" -q -m unit packages/atlasctl/tests
	@PYTHONPATH=packages/atlasctl/src "$(SCRIPTS_VENV)/bin/pytest" -q -m integration packages/atlasctl/tests

scripts-install-dev: ## Install python tooling for scripts package development
	@$(MAKE) -s internal/scripts/install-dev

scripts-install: ## Install scripts package tooling into local venv
	@$(MAKE) -s internal/scripts/install

scripts-venv: ## Create deterministic scripts venv under artifacts/atlasctl/venv/
	@$(MAKE) -s internal/scripts/venv

scripts-lock-check: ## Validate scripts lock consistency against pyproject
	@$(MAKE) -s internal/scripts/lock-check

scripts-run: ## Run atlasctl command (usage: make scripts-run CMD="doctor --json")
	@[ -n "$${CMD:-}" ] || { echo "usage: make scripts-run CMD='doctor --json'" >&2; exit 2; }
	@$(MAKE) -s internal/scripts/run CMD="$${CMD}"

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
packages-check: ## Validate python package surfaces and repository scripting policy
	@$(MAKE) -s internal/packages/check
ops/check: ## Fast ops verification (no cluster bring-up)
	@$(call with_iso,ops-check,$(MAKE) -s internal/ops/check)

ops/contract-check: ## Validate layer contract SSOT and drift/report gates
	@$(call with_iso,ops-contract-check,$(MAKE) -s ops-contract-check)

ops/smoke: ## Explicit ops smoke target
	@reuse="$${REUSE:-1}"; \
	$(call with_iso,ops-smoke,REUSE="$$reuse" $(MAKE) -s ops-smoke)

k8s-smoke: ## One-command local k8s smoke runner
	@$(MAKE) -s ops-k8s-smoke
warm: ## Warm datasets + shards and record cache state
	@./ops/run/warm-dx.sh

cache/status: ## Print cache status and budget policy checks
	@CACHE_STATUS_STRICT=0 ./ops/run/cache-status.sh

cache/prune: ## Prune local dataset/cache artifacts
	@./ops/run/cache-prune.sh

tooling-versions: ## Print Rust + Python tooling versions (pinned + local)
	@$(MAKE) -s internal/tooling-versions
ops/suite: ## Explicit ops suite target
	@$(call with_iso,ops-suite,$(MAKE) -s internal/ops/suite)

k8s/restart: ## Safe k8s rollout restart for atlas deployment
	@$(call with_iso,k8s-restart,$(MAKE) -s ops-k8s-restart)

k8s/apply-config: ## Validate values, apply deploy, and restart if configmap changed
	@$(call with_iso,k8s-apply-config,$(MAKE) -s ops-k8s-apply-config)

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
	@$(call with_iso,configs-check,$(MAKE) -s lane-configs-policies)

configs/all: ## Configs lane (schema + drift checks)
	@$(call with_iso,configs-all,$(MAKE) -s configs/check)

policies/check: ## Run deny/audit + policy-relaxation checks
	@$(call with_iso,policies-check,$(ATLAS_SCRIPTS) policies check --report text --emit-artifacts)

policies-check: ## Alias for policies/check
	@$(MAKE) -s policies/check

budgets/check: ## Validate universal budgets and budget-relaxation expiry policy
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/ops/checks/check_ops_budgets.py
	@$(ATLAS_SCRIPTS) run ./ops/_lint/budget-relaxations-audit.py

perf/baseline-update: ## Run smoke suite, update baseline, write diff summary and changelog
	@PROFILE="$${PROFILE:-$${ATLAS_PERF_BASELINE_PROFILE:-local}}"; \
	$(MAKE) -s ops-load-smoke; \
	PERF_BASELINE_UPDATE_FLOW=1 ATLAS_PERF_BASELINE_PROFILE="$$PROFILE" $(MAKE) -s ops-perf-baseline-update

perf/regression-check: ## Fail if p95 regression exceeds configured budget
	@PROFILE="$${PROFILE:-$${ATLAS_PERF_BASELINE_PROFILE:-local}}"; \
	$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/load/baseline/regression_check.py --profile "$$PROFILE" --results "$${RESULTS:-artifacts/perf/results}"

perf/triage: ## Print top p95 regressions by suite from latest perf results
	@PROFILE="$${PROFILE:-$${ATLAS_PERF_BASELINE_PROFILE:-local}}"; \
	$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/load/baseline/triage_regressions.py --profile "$$PROFILE" --results "$${RESULTS:-artifacts/perf/results}"

perf/compare: ## Compare two evidence perf runs (FROM=<run_id> TO=<run_id>)
	@[ -n "$${FROM:-}" ] || { echo "usage: make perf/compare FROM=<run_id> TO=<run_id>" >&2; exit 2; }
	@[ -n "$${TO:-}" ] || { echo "usage: make perf/compare FROM=<run_id> TO=<run_id>" >&2; exit 2; }
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/load/baseline/compare_runs.py --from-run "$${FROM}" --to-run "$${TO}"

policies/all: ## Policies lane (deny/audit/policy checks)
	@$(call with_iso,policies-all,$(MAKE) -s policies/check)

policies/boundaries-check: ## Enforce e2e layer boundary rules and relaxations
	@$(ATLAS_SCRIPTS) run ./ops/_lint/layer-relaxations-audit.py
	@$(ATLAS_SCRIPTS) run ./ops/_lint/no-layer-fixups.py
	@$(ATLAS_SCRIPTS) run ./ops/_lint/no-k8s-test-fixups.py
	@$(ATLAS_SCRIPTS) run ./ops/_lint/no-stack-layer-literals.py

local/all: ## Run all meaningful local gates
	@PARALLEL="$${PARALLEL:-1}" RUN_ID="$${RUN_ID:-$${MAKE_RUN_ID:-local-all-$(MAKE_RUN_TS)}}" MODE=root-local ./ops/run/root-lanes.sh

ci/all: ## Deterministic CI superset
	@$(call with_iso,ci-all,$(MAKE) -s gates-check lane-cargo lane-docs lane-scripts lane-configs lane-policies lane-ops ci-release-binaries ci-docs-build ci-release-compat-matrix-verify)

nightly/all: ## Slow nightly suites (perf/load/drills/realdata)
	@$(call with_iso,nightly-all,$(MAKE) -s ci/all ops-load-nightly perf/regression-check ops-drill-suite ops-drill-metric-cardinality-blowup ops-realdata ops-obs-verify SUITE=full ops-observability-lag-check)

lane-cargo: ## Lane: rust checks/tests in isolated lane-cargo path
	@$(MAKE) -s cargo/fmt cargo/lint
	@$(call with_iso,lane-cargo,NEXTEST_PROFILE=ci $(MAKE) -s check test test-all audit)

lane-docs: ## Lane: docs build/freeze/hardening
	@$(MAKE) -s docs/check

lane-ops: ## Lane: ops lint/contracts without cluster bring-up
	@$(MAKE) -s ops/check

lane-scripts: ## Lane: scripts lint/tests/audit
	@$(MAKE) -s scripts/check

lane-configs: ## Lane: configs checks and drift gates
	@$(call with_iso,lane-configs,$(MAKE) -s configs-check budgets/check atlasctl-budgets)

lane-policies: ## Lane: policy checks and boundary enforcement
	@$(call with_iso,lane-policies,$(MAKE) -s ci-deny policy-lint policy-schema-drift policy-audit policy-enforcement-status policy-allow-env-lint policies/boundaries-check)

lane-configs-policies: ## Alias lane for configs + policies
	@$(MAKE) -s lane-configs lane-policies

internal/lane-ops-smoke: ## Internal lane: bounded ops smoke path
	@$(call with_iso,internal-lane-ops-smoke,$(MAKE) -s ops-k8s-smoke)

internal/lane-obs-cheap: ## Internal lane: cheap observability verification for CI-fast
	@$(call with_iso,internal-lane-obs-cheap,$(MAKE) -s ops-obs-verify SUITE=cheap ops-observability-lag-check)

internal/lane-obs-full: ## Internal lane: full observability verification for root-local
	@$(call with_iso,internal-lane-obs-full,$(MAKE) -s ops-obs-verify SUITE=root-local)

root: ## CI-fast lane subset (no cluster bring-up)
	@run_id="$${RUN_ID:-$${MAKE_RUN_ID:-root-$(MAKE_RUN_TS)}}"; \
	$(MAKE) -s tools-check; \
	$(MAKE) -s scripts/test; \
	parallel_flag=""; if [ "$${PARALLEL:-1}" = "1" ]; then parallel_flag="--parallel"; fi; \
	RUN_ID="$$run_id" $(ATLAS_SCRIPTS) --quiet gates run --preset root --all $$parallel_flag --jobs "$${JOBS:-4}"; \
	$(ATLAS_SCRIPTS) --quiet report collect --run-id "$$run_id" >/dev/null; \
	$(ATLAS_SCRIPTS) --quiet report scorecard --run-id "$$run_id" >/dev/null; \
	test -f "artifacts/evidence/make/$$run_id/unified.json"; \
	test -f "ops/_generated_committed/scorecard.json"; \
	$(ATLAS_SCRIPTS) --quiet report print --run-id "$$run_id"

root-local: ## All lanes in parallel + ops smoke lane (PARALLEL=0 for serial)
	@run_id="$${RUN_ID:-$${MAKE_RUN_ID:-root-local-$(MAKE_RUN_TS)}}"; \
	$(MAKE) -s tools-check; \
	$(MAKE) -s scripts/test; \
	parallel_flag=""; if [ "$${PARALLEL:-1}" = "1" ]; then parallel_flag="--parallel"; fi; \
	RUN_ID="$$run_id" $(ATLAS_SCRIPTS) --quiet gates run --preset root-local --all $$parallel_flag --jobs "$${JOBS:-4}"; \
	if [ "$${PERF_CHEAP_REGRESSION:-0}" = "1" ]; then $(MAKE) -s ops-load-smoke perf/regression-check PROFILE="$${PROFILE:-local}"; fi; \
	$(ATLAS_SCRIPTS) --quiet report collect --run-id "$$run_id" >/dev/null; \
	$(ATLAS_SCRIPTS) --quiet report scorecard --run-id "$$run_id" >/dev/null; \
	test -f "artifacts/evidence/make/$$run_id/unified.json"; \
	test -f "ops/_generated_committed/scorecard.json"; \
	$(ATLAS_SCRIPTS) --quiet report print --run-id "$$run_id"

root-local/no-ops: ## Local lanes without ops smoke lane (explicit skip)
	@NO_OPS=1 PARALLEL="$${PARALLEL:-1}" RUN_ID="$${RUN_ID:-$${MAKE_RUN_ID:-root-local-no-ops-$(MAKE_RUN_TS)}}" MODE=root-local ./ops/run/root-lanes.sh

root-local-no-ops: ## Alias for root-local/no-ops
	@$(MAKE) -s root-local/no-ops

root-local-fast: ## Debug serial root-local skipping expensive extras (ops-smoke, obs-full)
	@run_id="$${RUN_ID:-$${MAKE_RUN_ID:-root-local-fast-$(MAKE_RUN_TS)}}"; \
	PARALLEL=0 FAST=1 RUN_ID="$$run_id" MODE=root-local ./ops/run/root-lanes.sh; \
	$(ATLAS_SCRIPTS) report collect --run-id "$$run_id" >/dev/null; \
	$(ATLAS_SCRIPTS) report print --run-id "$$run_id"

root-local-open: ## Open or print latest root-local summary report
	@SUMMARY_RUN_ID="$${RUN_ID:-}" MODE=open ./ops/run/root-lanes.sh

repro: ## Re-run one lane deterministically (usage: make repro TARGET=lane-cargo SEED=123)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make repro TARGET=<lane-target> [SEED=123]"; exit 2; }
	@seed="$${SEED:-0}"; \
	echo "repro target=$${TARGET} seed=$${seed}"; \
	TZ=UTC LANG=C.UTF-8 LC_ALL=C.UTF-8 TEST_RANDOM_SEED="$$seed" ATLAS_TEST_SEED="$$seed" $(MAKE) -s "$${TARGET}"

retry: ## Retry a target with same RUN_ID (usage: make retry TARGET=<target>)
	@[ -n "$${TARGET:-}" ] || { echo "usage: make retry TARGET=<target>"; exit 2; }
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || echo $(MAKE_RUN_ID))}"; \
	echo "retry target=$${TARGET} run_id=$$run_id"; \
	RUN_ID="$$run_id" QUIET="$${QUIET:-0}" $(MAKE) -s "$${TARGET}"

legacy/check: ## Verify legacy inventory and policy contracts
	@$(ATLAS_SCRIPTS) legacy check --report text

legacy/audit: ## List non-scripts files still referencing scripts/ paths
	@$(ATLAS_SCRIPTS) legacy audit --report text

cleanup/verify: ## One-time cleanup safety verification before deleting legacy paths
	@$(MAKE) -s legacy/check scripts-check ops-contracts-check
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/docs/check_help_snapshot.py && $(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/contracts/hygiene/check_no_dead_entrypoints.py && $(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/docs/check_no_orphan_docs_refs.py && $(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/contracts/orphans/check_no_orphan_configs.py && $(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/contracts/orphans/check_no_orphan_owners.py

local: ## Developer confidence suite
	@$(MAKE) -s root-local

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

ci: ## CI entrypoint mirror
	@$(MAKE) -s ci/all

nightly: ## Deprecated alias for nightly/all
	@$(MAKE) -s nightly/all

ops: ## Run canonical ops verification lane
	@$(ATLAS_SCRIPTS) ops check --report text

root-local-summary: ## Print status and artifact paths for RUN_ID
	@SUMMARY_RUN_ID="$${RUN_ID:-}" MODE=summary ./ops/run/root-lanes.sh

lane-status: ## Print all lane statuses for RUN_ID (or latest)
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || true)}"; \
	[ -n "$$run_id" ] || { echo "RUN_ID is required (or run root/root-local first)" >&2; exit 2; }; \
	$(ATLAS_SCRIPTS) report print --run-id "$$run_id"

open: ## Open unified report for RUN_ID (or print path if opener unavailable)
	@run_id="$${RUN_ID:-$$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || true)}"; \
	[ -n "$$run_id" ] || { echo "RUN_ID is required (or run root/root-local first)" >&2; exit 2; }; \
	path="artifacts/evidence/make/$$run_id/unified.json"; \
	[ -f "$$path" ] || $(ATLAS_SCRIPTS) report collect --run-id "$$run_id" >/dev/null; \
	echo "$$path"; \
	if command -v open >/dev/null 2>&1; then open "$$path" >/dev/null 2>&1 || true; \
	elif command -v xdg-open >/dev/null 2>&1; then xdg-open "$$path" >/dev/null 2>&1 || true; fi

rerun-failed: ## Rerun only failed lanes from RUN_ID (NEW_RUN_ID optional)
	@src="$${RUN_ID:-}"; \
	[ -n "$$src" ] || { echo "RUN_ID is required (source run id)" >&2; exit 2; }; \
	new="$${NEW_RUN_ID:-$${src}-rerun-$(MAKE_RUN_TS)}"; \
	PARALLEL="$${PARALLEL:-0}" MODE=rerun-failed SOURCE_RUN_ID="$$src" RUN_ID="$$new" ./ops/run/root-lanes.sh; \
	$(ATLAS_SCRIPTS) report collect --run-id "$$new" >/dev/null; \
	$(ATLAS_SCRIPTS) report print --run-id "$$new"

dev-bootstrap: ## Setup local python tooling for atlas-scripts (uv sync)
	@if command -v uv >/dev/null 2>&1; then \
		uv sync --project packages/atlasctl; \
	else \
		echo "uv is not installed; falling back to make scripts-install"; \
		$(MAKE) -s scripts-install; \
	fi

make/guard-no-python-scripts: ## Guard against direct python scripts path invocation in make recipes
	@! rg -n "python(3)?\\s+\\.?/?scripts/" makefiles/*.mk >/dev/null || { \
		echo "direct python path invocation is forbidden; use $(ATLAS_SCRIPTS) or $(PY_RUN)"; \
		rg -n "python(3)?\\s+\\.?/?scripts/" makefiles/*.mk; \
		exit 1; \
	}

make/guard-no-script-paths: ## Guard against direct bash/python scripts path invocation in make recipes
	@! rg -n "(python(3)?|bash|sh)\\s+\\.?/?scripts/" makefiles/*.mk >/dev/null || { \
		echo "direct scripts/ path invocation is forbidden in make recipes; use atlasctl commands"; \
		rg -n "(python(3)?|bash|sh)\\s+\\.?/?scripts/" makefiles/*.mk; \
		exit 1; \
	}

root-determinism: ## Assert make root determinism (inventory outputs stable across two runs)
	@./packages/atlasctl/src/atlasctl/checks/layout/check_root_determinism.sh


telemetry-contracts: ## Regenerate telemetry generated artifacts from observability contracts
	@$(ATLAS_SCRIPTS) contracts generate --generators artifacts
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

architecture-check: ## Validate runtime architecture boundaries and dependency guardrails
	@$(ATLAS_SCRIPTS) docs generate-architecture-map --report text
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
	@./ops/datasets/scripts/fixtures/fetch-real-datasets.sh

ssot-check:
	@$(ATLAS_SCRIPTS) contracts generate --generators artifacts chart-schema
	@$(ATLAS_SCRIPTS) contracts check --checks breakage drift endpoints error-codes sqlite-indexes chart-values

policy-lint:
	@./bin/atlasctl policies check --fail-fast

policy-schema-drift:
	@./bin/atlasctl policies schema-drift

policy-audit: ## Audit policy relaxations report + enforce registry/expiry/budget gates
	@./bin/atlasctl policies check --fail-fast

policy-enforcement-status: ## Validate policy pass/fail coverage table and generate status doc
	@./bin/atlasctl policies enforcement-status --enforce

policy-allow-env-lint: ## Forbid ALLOW_* escape hatches unless declared in env schema
	@./bin/atlasctl policies allow-env-lint

ops-policy-audit: ## Verify ops policy configs are reflected by ops make/scripts contracts
	@$(ATLAS_SCRIPTS) ops policy-audit

policy-drift-diff: ## Show policy contract drift between two refs (usage: make policy-drift-diff [FROM=HEAD~1 TO=HEAD])
	@./bin/atlasctl policies drift-diff --from-ref "$${FROM:-HEAD~1}" --to-ref "$${TO:-HEAD}"

release-update-compat-matrix:
	@[ -n "$$TAG" ] || { echo "usage: make release-update-compat-matrix TAG=<tag>"; exit 2; }
	@$(ATLAS_SCRIPTS) compat update-matrix --tag "$$TAG"

.PHONY: root-local-no-ops architecture-check artifacts-clean artifacts-index artifacts-open bootstrap bootstrap-tools bump cargo/all chart chart-package chart-verify ci ci/all ci-workflow-contract clean config-drift config-print config-validate configs-gen-check configs-check configs/all contracts dataset-id-lint debug deep-clean docker docker-build docker-contracts docker-push docker-scan docker-smoke docs docs/all docs-lint-names doctor evidence/open evidence/clean evidence/check evidence/bundle evidence/pr-summary explain what fetch-real-datasets format gates gates-check governance-check graph help help-advanced help-all open lane-status rerun-failed hygiene internal-list inventory isolate-clean layout-check layout-migrate list local local/all local-full makefiles-contract nightly nightly/all no-direct-scripts obs/update-goldens ops-alerts-validate ops ops/all ops-api-protection ops-artifacts-open ops-baseline-policy-check ops-cache-pin-set ops-cache-status ops-catalog-validate ops-check ops-clean ops-contracts-check ops-dashboards-validate ops-dataset-federated-registry-test ops-dataset-multi-release-test ops-dataset-promotion-sim ops-dataset-qc ops-datasets-fetch ops-deploy ops-doctor ops-down ops-drill-corruption-dataset ops-drill-memory-growth ops-drill-otel-outage ops-drill-overload ops-drill-pod-churn ops-drill-rate-limit ops-drill-rollback ops-drill-rollback-under-load ops-drill-store-outage ops-drill-suite ops-drill-toxiproxy-latency ops-drill-upgrade ops-drill-upgrade-under-load ops-e2e ops-e2e-smoke ops-full ops-full-pr ops-gc-smoke ops-gen ops-gen-check ops-graceful-degradation ops-incident-repro-kit ops-k8s-smoke k8s-smoke ops-k8s-suite ops-k8s-template-tests ops-k8s-tests ops-load-ci ops-load-full ops-load-manifest-validate ops-load-nightly ops-load-shedding ops-load-smoke ops-load-soak ops-load-suite ops-local-full ops-local-full-stack ops-metrics-check ops-obs-down ops-obs-install ops-obs-mode ops-obs-uninstall ops-obs-verify ops-observability-pack-conformance-report ops-observability-pack-export ops-observability-pack-health ops-observability-pack-smoke ops-observability-pack-verify ops-observability-smoke ops-observability-validate ops-open-grafana ops-openapi-validate ops-perf-baseline-update ops-perf-cold-start ops-perf-nightly ops-perf-report ops-perf-warm-start ops-policy-audit ops-prereqs ops-proof-cached-only ops-publish ops-readiness-scorecard ops-realdata ops-redeploy ops-ref-grade-local ops-ref-grade-nightly ops-ref-grade-pr ops-release-matrix ops-release-rollback ops-release-update ops-report ops-slo-alert-proof ops-slo-burn ops-slo-report ops-smoke ops-tools-check ops-traces-check ops-undeploy ops-up ops-values-validate ops-warm ops-warm-datasets ops-warm-shards ops-warm-top policies/all policy-allow-env-lint policy-audit policy-drift-diff policy-enforcement-status policy-lint policy-schema-drift prereqs quick release release-dry-run release-update-compat-matrix rename-lint report k8s load obs root root-determinism root-local root-local-fast root-local-summary scripts-all scripts/all scripts-audit scripts-check scripts-clean scripts-format scripts-graph scripts-index scripts-lint scripts-test scripts-install-dev ssot-check verify-inventory lane-cargo lane-docs lane-ops lane-scripts lane-configs-policies root-local-open repro lane-status open rerun-failed internal/lane-ops-smoke internal/lane-obs-cheap internal/lane-obs-full report/merge report/print report/md report/junit report/bundle clean-safe clean-all print-env cargo/fmt cargo/lint cargo/test-fast cargo/test cargo/test-all cargo/test-contracts cargo/audit cargo/bench-smoke cargo/coverage configs/check budgets/check perf/baseline-update perf/regression-check perf/triage perf/compare policies/check policies/boundaries-check retry docs/check docs/build docs/fmt docs/lint docs/test docs/clean scripts/check scripts/build scripts/fmt scripts/lint scripts/test scripts/clean ops/check ops/smoke ops/suite ops/fmt ops/lint ops/test ops/build ops/clean pins/check pins/update logs/last-fail cache/status cache/prune root-local/no-ops



inventory: ## Regenerate inventories from atlasctl SSOT generators
	@$(ATLAS_SCRIPTS) make inventory --out-dir docs/_generated
	@$(ATLAS_SCRIPTS) inventory all --format both --out-dir docs/_generated

verify-inventory: ## Fail if inventory outputs drift from generated state
	@$(ATLAS_SCRIPTS) make inventory --out-dir docs/_generated --check
	@$(MAKE) -s inventory
	@$(ATLAS_SCRIPTS) inventory budgets --check --format json --dry-run >/dev/null
	@git diff --exit-code -- docs/_generated/INDEX.md docs/_generated/make-targets.md docs/_generated/make-targets.json docs/_generated/ops-surface.md docs/_generated/ops-surface.json docs/_generated/configs-surface.md docs/_generated/configs-surface.json docs/_generated/schema-index.md docs/_generated/schema-index.json docs/_generated/ownership.md docs/_generated/ownership.json docs/_generated/contracts-index.md docs/_generated/contracts-index.json docs/_generated/inventory-budgets.md docs/_generated/inventory-budgets.json

upgrade-guide: ## Generate make target upgrade guide for renamed/deprecated aliases
	@$(ATLAS_SCRIPTS) docs generate-upgrade-guide --report text

artifacts-index: ## Generate artifacts index for inspection UIs
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/artifacts/build_artifacts_index.py

artifacts-clean: ## Clean old artifacts with safe retention
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/artifacts/clean_artifacts.py

isolate-clean: ## Remove isolate output directories safely
	@find artifacts/isolate -mindepth 1 -maxdepth 1 -type d -exec rm -r {} + 2>/dev/null || true

clean: ## Safe clean for generated local outputs
	@$(ATLAS_SCRIPTS) cleanup --older-than "$${OLDER_THAN_DAYS:-14}"
	@./ops/run/clean.sh

clean-safe: ## Clean only safe generated make artifact directories
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/artifacts/clean_make_artifacts.py

clean-all: ## Clean all allowed generated dirs (requires CONFIRM=YES)
	@[ "$${CONFIRM:-}" = "YES" ] || { echo "refusing clean-all without CONFIRM=YES"; exit 2; }
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/artifacts/clean_make_artifacts.py --all

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
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_makefiles_contract.py

ci-workflow-contract: ## Validate CI and nightly workflows use canonical make entrypoints
	@$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/checks/layout/workflows/check_ci_entrypoints.py
