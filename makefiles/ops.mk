SHELL := /bin/sh

# Ops SSOT targets
OPS_ENV_SCHEMA ?= configs/ops/env.schema.json

ATLAS_BASE_URL ?= http://127.0.0.1:18080
ATLAS_NS ?= atlas-e2e
ATLAS_VALUES_FILE ?= ops/k8s/values/local.yaml
ATLAS_OFFLINE_VALUES_FILE ?= ops/k8s/values/offline.yaml
ATLAS_PERF_VALUES_FILE ?= ops/k8s/values/perf.yaml
ATLAS_MULTI_REGISTRY_VALUES_FILE ?= ops/k8s/values/multi-registry.yaml
ATLAS_INGRESS_VALUES_FILE ?= ops/k8s/values/ingress.yaml
ATLAS_RUN_ID ?= local
ATLAS_E2E_TIMEOUT ?= 180s
ATLAS_E2E_ENABLE_REDIS ?= 0
ATLAS_E2E_ENABLE_OTEL ?= 0
ATLAS_E2E_ENABLE_TOXIPROXY ?= 0
ATLAS_E2E_TEST_GROUP ?=
ATLAS_E2E_TEST ?=
OPS_RUN_ID ?= $(ATLAS_RUN_ID)
OPS_RUN_DIR ?= artifacts/ops/$(OPS_RUN_ID)

export ATLAS_BASE_URL
export ATLAS_NS
export ATLAS_VALUES_FILE
export ATLAS_OFFLINE_VALUES_FILE
export ATLAS_PERF_VALUES_FILE
export ATLAS_MULTI_REGISTRY_VALUES_FILE
export ATLAS_INGRESS_VALUES_FILE
export ATLAS_RUN_ID
export ATLAS_E2E_TIMEOUT
export ATLAS_E2E_ENABLE_REDIS
export ATLAS_E2E_ENABLE_OTEL
export ATLAS_E2E_ENABLE_TOXIPROXY
export ATLAS_E2E_TEST_GROUP
export ATLAS_E2E_TEST
export ATLAS_E2E_NAMESPACE ?= $(ATLAS_NS)
export ATLAS_E2E_VALUES_FILE ?= $(ATLAS_VALUES_FILE)
export OPS_RUN_ID
export OPS_RUN_DIR

ops-env-validate: ## Validate canonical ops environment contract against schema
	@python3 ./scripts/layout/validate_ops_env.py --schema "$(OPS_ENV_SCHEMA)"

ops-env-print: ## Print canonical ops environment settings
	@python3 ./scripts/layout/validate_ops_env.py --schema "$(OPS_ENV_SCHEMA)" --print

ops-stack-up: ## Bring up stack components only (kind + stack manifests)
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-kind-version-check
	@$(MAKE) ops-kubectl-version-check
	@$(MAKE) ops-helm-version-check
	@./ops/e2e/scripts/up.sh
	@$(MAKE) ops-cluster-sanity

ops-up: ## Bring up local ops stack (kind + minio + prometheus + optional otel/redis)
	@$(MAKE) ops-stack-up

ops-cluster-sanity: ## Validate cluster node/dns/storageclass sanity
	@./ops/e2e/k8s/tests/test_cluster_sanity.sh

ops-stack-down: ## Tear down stack components and cluster
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/scripts/down.sh

ops-down: ## Tear down local ops stack
	@$(MAKE) ops-stack-down

ops-stack-validate: ## Validate stack manifests and formatting drift
	@./ops/stack/scripts/validate.sh

ops-stack-smoke: ## Stack-only smoke test without atlas deploy
	@./ops/stack/scripts/stack_smoke.sh

ops-stack-health-report: ## Collect stack health summary/events/logs
	@./ops/stack/scripts/health_report.sh "$${ATLAS_NS}" "artifacts/ops/stack/health-report.txt" >/dev/null
	@./ops/stack/scripts/collect_events.sh "$${ATLAS_NS}" "artifacts/ops/stack/events.txt" >/dev/null
	@./ops/stack/scripts/collect_pod_logs.sh "$${ATLAS_NS}" "artifacts/ops/stack/logs" >/dev/null

ops-stack-wait-ready: ## Wait for stack namespace readiness gates
	@./ops/stack/scripts/wait_ready.sh "$${ATLAS_NS}"

ops-stack-version: ## Print pinned stack component versions
	@cat ops/stack/version-manifest.json

ops-stack-uninstall: ## Uninstall stack resources and cluster
	@./ops/stack/scripts/uninstall.sh

ops-stack-slow-store: ## Enable slow-store mode via toxiproxy latency
	@ATLAS_E2E_ENABLE_TOXIPROXY=1 $(MAKE) ops-stack-up
	@./ops/stack/toxiproxy/enable_slow_store.sh

ops-reset: ## Reset ops state (namespace/PV store data + local store artifacts)
	@$(MAKE) -s ops-env-validate
	@kubectl delete ns "$${ATLAS_NS}" --ignore-not-found >/dev/null 2>&1 || true
	@kubectl delete pvc --all -n default >/dev/null 2>&1 || true
	@./ops/e2e/scripts/cleanup_store.sh

ops-publish-medium: ## Ingest + publish medium fixture dataset
	@$(MAKE) -s ops-env-validate
	@if [ ! -f ops/fixtures/medium/data/genes.gff3 ] || [ ! -f ops/fixtures/medium/data/genome.fa ] || [ ! -f ops/fixtures/medium/data/genome.fa.fai ]; then \
	  ./scripts/fixtures/fetch-medium.sh; \
	fi
	@./ops/e2e/scripts/publish_dataset.sh \
	  --gff3 ops/fixtures/medium/data/genes.gff3 \
	  --fasta ops/fixtures/medium/data/genome.fa \
	  --fai ops/fixtures/medium/data/genome.fa.fai \
	  --release 110 --species homo_sapiens --assembly GRCh38

ops-publish: ## Compatibility alias for ops-publish-medium
	@$(MAKE) ops-publish-medium

ops-deploy: ## Deploy atlas chart into local cluster
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/scripts/deploy_atlas.sh

ops-offline: ## Deploy atlas in cached-only offline profile
	@$(MAKE) -s ops-env-validate
	@ATLAS_VALUES_FILE="$(ATLAS_OFFLINE_VALUES_FILE)" $(MAKE) ops-deploy

ops-perf: ## Deploy atlas in perf profile and run load smoke
	@$(MAKE) -s ops-env-validate
	@ATLAS_VALUES_FILE="$(ATLAS_PERF_VALUES_FILE)" $(MAKE) ops-deploy
	@$(MAKE) ops-load-smoke

ops-multi-registry: ## Deploy atlas with multi-registry values profile
	@$(MAKE) -s ops-env-validate
	@ATLAS_VALUES_FILE="$(ATLAS_MULTI_REGISTRY_VALUES_FILE)" $(MAKE) ops-deploy

ops-ingress: ## Deploy atlas with ingress values profile
	@$(MAKE) -s ops-env-validate
	@ATLAS_VALUES_FILE="$(ATLAS_INGRESS_VALUES_FILE)" $(MAKE) ops-deploy

ops-warm: ## Run warmup workflow
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/scripts/warmup.sh

ops-soak: ## Run soak workflow (10-30 minutes)
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/scripts/soak.sh

ops-smoke: ## Run canonical API smoke queries
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/scripts/smoke_queries.sh
	@python3 ./scripts/docs/check_openapi_examples.py
	@$(MAKE) ops-metrics-check
	@./ops/observability/scripts/snapshot_metrics.sh
	@./ops/observability/scripts/snapshot_traces.sh
	@mkdir -p artifacts/ops/observability && cp ops/observability/grafana/atlas-observability-dashboard.json artifacts/ops/observability/dashboard.snapshot.json

ops-metrics-check: ## Validate runtime metrics and observability contracts
	@./ops/e2e/scripts/verify_metrics.sh
	@./scripts/observability/check_metrics_contract.py
	@./scripts/observability/check_dashboard_contract.py
	@./scripts/observability/check_alerts_contract.py
	@./scripts/observability/lint_runbooks.py
	@./scripts/observability/check_runtime_metrics.py
	@./ops/observability/scripts/snapshot_metrics.sh
	@./ops/observability/scripts/check_metric_cardinality.py
	@python3 ./ops/observability/scripts/validate_logs_schema.py

ops-traces-check: ## Validate trace signal (when OTEL enabled)
	@./ops/e2e/scripts/verify_traces.sh
	@if [ "$${ATLAS_E2E_ENABLE_OTEL:-0}" = "1" ]; then ./scripts/observability/check_tracing_contract.py; else echo "trace contract skipped (ATLAS_E2E_ENABLE_OTEL=0)"; fi

ops-k8s-tests: ## Run k8s e2e suite
	@$(MAKE) -s ops-env-validate
	@python3 ./scripts/ops/check_k8s_test_contract.py
	@SHELLCHECK_STRICT=1 $(MAKE) ops-shellcheck
	@group_args=""; \
	if [ -n "$${ATLAS_E2E_TEST_GROUP}" ]; then \
	  group_args="$$group_args --group $${ATLAS_E2E_TEST_GROUP}"; \
	fi; \
	if [ -n "$${ATLAS_E2E_TEST}" ]; then \
	  group_args="$$group_args --test $${ATLAS_E2E_TEST}"; \
	fi; \
	./ops/e2e/k8s/tests/run_all.sh $$group_args
	@python3 ./scripts/ops/check_k8s_flakes.py

ops-k8s-template-tests: ## Run helm template/lint edge-case checks
	@./ops/e2e/k8s/tests/test_helm_templates.sh

ops-load-prereqs: ## Validate load harness prerequisites (k6 + endpoint)
	@./ops/load/scripts/check_prereqs.sh

ops-load-smoke: ## Run short load suite
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-k6-version-check
	@$(MAKE) ops-load-manifest-validate
	@./ops/load/scripts/check_pinned_queries_lock.py
	@$(MAKE) ops-load-prereqs
	@./ops/load/scripts/run_suites_from_manifest.py --profile smoke --out artifacts/perf/results
	@./ops/load/scripts/validate_results.py artifacts/perf/results

ops-load-full: ## Run nightly/full load suites
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-k6-version-check
	@$(MAKE) ops-load-manifest-validate
	@./ops/load/scripts/check_pinned_queries_lock.py
	@./ops/load/scripts/run_suites_from_manifest.py --profile full --out artifacts/perf/results
	@./ops/load/scripts/validate_results.py artifacts/perf/results
	@./ops/load/reports/generate.py

ops-load-under-rollout: ## Run load while rollout is in progress
	@$(MAKE) -s ops-env-validate
	@./ops/load/scripts/load_under_rollout.sh

ops-load-under-rollback: ## Run load while rollback is in progress
	@$(MAKE) -s ops-env-validate
	@./ops/load/scripts/load_under_rollback.sh

ops-load-ci: ## Load CI profile (smoke suites + score/report)
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-k6-version-check
	@$(MAKE) ops-load-manifest-validate
	@./ops/load/scripts/run_suites_from_manifest.py --profile load-ci --out artifacts/perf/results
	@./ops/load/scripts/validate_results.py artifacts/perf/results
	@./ops/load/scripts/score_k6.py || true
	@./ops/load/reports/generate.py

ops-load-nightly: ## Load nightly profile (nightly suites + score/report)
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-k6-version-check
	@$(MAKE) ops-load-manifest-validate
	@./ops/load/scripts/run_suites_from_manifest.py --profile load-nightly --out artifacts/perf/results
	@./ops/load/scripts/validate_results.py artifacts/perf/results
	@./ops/load/scripts/score_k6.py || true
	@./ops/load/reports/generate.py

ops-drill-store-outage: ## Run store outage drill under load
	@$(MAKE) -s ops-env-validate
	@./ops/load/scripts/run_suite.sh store-outage-mid-spike.json artifacts/perf/results
	@./ops/observability/scripts/drill_store_outage.sh

ops-drill-minio-outage: ## Drill minio outage under load with cached endpoint checks
	@$(MAKE) -s ops-env-validate
	@./ops/observability/scripts/drill_minio_outage_mid_load.sh

ops-drill-prom-outage: ## Drill prometheus outage while atlas keeps serving
	@$(MAKE) -s ops-env-validate
	@./ops/observability/scripts/drill_prom_outage.sh

ops-drill-otel-outage: ## Drill otel outage while atlas keeps serving
	@$(MAKE) -s ops-env-validate
	@./ops/observability/scripts/drill_otel_outage.sh

ops-drill-toxiproxy-latency: ## Inject toxiproxy latency and assert store breaker signal
	@$(MAKE) -s ops-env-validate
	@ATLAS_E2E_ENABLE_TOXIPROXY=1 $(MAKE) ops-stack-up
	@./ops/observability/scripts/drill_toxiproxy_latency.sh

ops-drill-alerts: ## Run alert drill checks against configured rules
	@./ops/observability/scripts/drill_alerts.sh

ops-drill-overload: ## Verify overload signal drill assertions
	@$(MAKE) -s ops-env-validate
	@./ops/observability/scripts/drill_overload.sh

ops-drill-memory-growth: ## Verify memory-growth drill assertions
	@$(MAKE) -s ops-env-validate
	@./ops/observability/scripts/drill_memory_growth.sh

ops-drill-corruption: ## Run corruption handling drill
	@cargo test -p bijux-atlas-server cache_manager_tests::chaos_mode_random_byte_corruption_never_serves_results -- --exact

ops-drill-pod-churn: ## Run pod churn drill while service handles load
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/k8s/tests/drill_pod_churn.sh

ops-drill-upgrade: ## Run upgrade drill and verify semantic stability
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/realdata/upgrade_drill.sh

ops-drill-rollback: ## Run rollback drill and verify semantic stability
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/realdata/rollback_drill.sh

ops-upgrade-drill: ## Compatibility alias for ops-drill-upgrade
	@$(MAKE) ops-drill-upgrade

ops-rollback-drill: ## Compatibility alias for ops-drill-rollback
	@$(MAKE) ops-drill-rollback

ops-realdata: ## Run real-data e2e scenarios
	@$(MAKE) -s ops-env-validate
	@./ops/e2e/realdata/run_all.sh

ops-report: ## Gather ops evidence into artifacts/ops/<run-id>/
	@$(MAKE) -s ops-env-validate
	@out="$${OPS_RUN_DIR}"; \
	mkdir -p "$$out"/{logs,perf,metrics}; \
	kubectl get pods -A -o wide > "$$out/logs/pods.txt" 2>/dev/null || true; \
	kubectl get events -A --sort-by=.lastTimestamp > "$$out/logs/events.txt" 2>/dev/null || true; \
	kubectl logs -n "$${ATLAS_E2E_NAMESPACE:-atlas-e2e}" -l app.kubernetes.io/name=bijux-atlas --tail=2000 > "$$out/logs/atlas.log" 2>/dev/null || true; \
	cp -R artifacts/perf/results "$$out/perf/" 2>/dev/null || true; \
	curl -fsS "$${ATLAS_BASE_URL:-http://127.0.0.1:8080}/metrics" > "$$out/metrics/metrics.txt" 2>/dev/null || true; \
	echo "ops report written to $$out"; \
	ln -sfn "$${OPS_RUN_ID}" artifacts/ops/latest; \
	$(MAKE) artifacts-index

ops-slo-burn: ## Compute SLO burn artifact from k6 score + metrics snapshot
	@python3 ./ops/observability/scripts/compute_slo_burn.py

ops-script-coverage: ## Validate every ops/**/scripts entrypoint is exposed via make
	@./scripts/layout/check_ops_script_targets.sh
	@SHELLCHECK_STRICT=1 $(MAKE) ops-shellcheck
	@$(MAKE) -s ops-shfmt

ops-shellcheck: ## Lint all ops shell scripts via shared wrapper
	@./ops/_lib/shellcheck.sh

ops-shfmt: ## Format-check all ops shell scripts (optional if shfmt unavailable)
	@if command -v shfmt >/dev/null 2>&1; then \
	  find ops -type f -name '*.sh' -print0 | xargs -0 shfmt -d; \
	else \
	  echo "shfmt not installed (optional)"; \
	fi

ops-kind-version-check: ## Validate pinned kind version from configs/ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py kind

ops-k6-version-check: ## Validate pinned k6 version from configs/ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py k6

ops-helm-version-check: ## Validate pinned helm version from configs/ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py helm

ops-kubectl-version-check: ## Validate pinned kubectl version from configs/ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py kubectl

ops-tools-check: ## Validate all pinned ops tools versions
	@$(MAKE) ops-kind-version-check
	@$(MAKE) ops-k6-version-check
	@$(MAKE) ops-helm-version-check
	@$(MAKE) ops-kubectl-version-check

ops-tool-check: ## Compatibility alias for ops-tools-check
	@$(MAKE) ops-tools-check

ops-kubeconform-version-check: ## Optional kubeconform version check (if installed)
	@if command -v kubeconform >/dev/null 2>&1; then kubeconform -v; else echo "kubeconform not installed (optional)"; fi

ops-perf-prepare-store: ## Perf helper: prepare local perf store fixture
	@./ops/load/scripts/prepare_perf_store.sh

ops-perf-e2e: ## Perf helper: run e2e perf suite
	@$(MAKE) ops-load-manifest-validate
	@./ops/load/scripts/run_e2e_perf.sh

ops-perf-nightly: ## Perf helper: run nightly perf suite
	@$(MAKE) ops-load-manifest-validate
	@./ops/load/scripts/run_nightly_perf.sh

ops-perf-report: ## Generate perf markdown + baseline report from artifacts
	@./ops/load/scripts/generate_report.py
	@./ops/load/reports/generate.py

ops-perf-cold-start: ## Perf helper: run cold-start benchmark
	@./ops/load/scripts/cold_start_benchmark.sh

ops-perf-cold-start-prefetch-5pods: ## Perf helper: run 5-pod prefetch cold-start benchmark
	@./ops/load/scripts/cold_start_prefetch_5pods.sh

ops-perf-compare-redis: ## Perf helper: compare Redis-on vs Redis-off perf runs
	@ATLAS_ENABLE_REDIS_EXPERIMENT=1 ./ops/load/scripts/validate_suite_manifest.py
	@./ops/load/scripts/compare_redis.sh

ops-baseline-policy-check: ## Enforce explicit approval policy for baseline updates
	@./ops/load/scripts/check_baseline_update_policy.sh

ops-perf-baseline-update: ## Update named baseline from artifacts/perf/baseline.json (requires approval for commit)
	@./ops/load/scripts/update_baseline.sh "$${ATLAS_PERF_BASELINE_PROFILE:-local}"

ops-load-manifest-validate: ## Validate load suite SSOT, naming conventions, and pinned query lock
	@./ops/load/scripts/validate_suite_manifest.py
	@./ops/load/scripts/check_runbook_suite_names.py

ops-perf-suite: ## Perf helper: run an arbitrary perf suite (SCENARIO=<file.js> OUT=<dir>)
	@[ -n "$$SCENARIO" ] || { echo "usage: make ops-perf-suite SCENARIO=<file.js> [OUT=artifacts/perf/results]" >&2; exit 2; }
	@./ops/load/scripts/run_suite.sh "$$SCENARIO" "$${OUT:-artifacts/perf/results}"

ops-values-validate: ## Validate chart values against SSOT contract
	@./scripts/contracts/generate_chart_values_schema.py
	@./scripts/contracts/check_chart_values_contract.py
	@./ops/e2e/k8s/tests/test_chart_drift.sh

ops-release-matrix: ## Generate k8s release install matrix document from CI summary
	@./ops/k8s/ci/install-matrix.sh

ops-openapi-validate: ## Validate OpenAPI drift and schema/examples consistency
	@./scripts/openapi-diff-check.sh
	@python3 ./scripts/docs/check_openapi_examples.py

ops-dashboards-validate: ## Validate dashboard references against metrics contract
	@./scripts/observability/check_dashboard_contract.py

ops-alerts-validate: ## Validate alert rules and contract coverage
	@./scripts/observability/check_alerts_contract.py

ops-observability-validate: ## Validate observability assets/contracts end-to-end
	@set -e; \
	trap 'out="artifacts/ops/observability/validate-fail-$$(date +%Y%m%d-%H%M%S)"; mkdir -p "$$out"; kubectl get pods -A -o wide > "$$out/pods.txt" 2>/dev/null || true; kubectl get events -A --sort-by=.lastTimestamp > "$$out/events.txt" 2>/dev/null || true; cp -f ops/observability/grafana/atlas-observability-dashboard.json "$$out/dashboard.json" 2>/dev/null || true; cp -f ops/observability/alerts/atlas-alert-rules.yaml "$$out/alerts.yaml" 2>/dev/null || true; echo "observability validation failed, artifacts: $$out" >&2' ERR; \
	$(MAKE) ops-dashboards-validate; \
	$(MAKE) ops-alerts-validate; \
	./scripts/observability/check_metrics_contract.py; \
	if [ "$${ATLAS_E2E_ENABLE_OTEL:-0}" = "1" ]; then ./scripts/observability/check_tracing_contract.py; else echo "trace contract skipped (ATLAS_E2E_ENABLE_OTEL=0)"; fi; \
	./ops/observability/scripts/snapshot_metrics.sh; \
	./ops/observability/scripts/check_metric_cardinality.py; \
	python3 ./ops/observability/scripts/validate_logs_schema.py

ops-obs-validate: ## Compatibility alias for ops-observability-validate
	@$(MAKE) ops-observability-validate

ops-obs-install: ## Install observability pack
	@$(MAKE) ops-obs-up

ops-obs-uninstall: ## Uninstall observability pack
	@$(MAKE) ops-obs-down

ops-observability-smoke: ## Install observability pack and run smoke checks
	@$(MAKE) ops-obs-up
	@./ops/observability/scripts/snapshot_metrics.sh
	@./ops/observability/scripts/snapshot_traces.sh
	@$(MAKE) ops-observability-validate
	@./ops/observability/scripts/drill_alerts.sh

ops-obs-up: ## Install observability pack (prometheus/otel, CRD-aware)
	@./ops/observability/scripts/install_obs_pack.sh

ops-obs-down: ## Uninstall observability pack
	@./ops/observability/scripts/uninstall_obs_pack.sh


ops-obs-mode: ## Install observability pack in requested mode (ATLAS_OBS_MODE=minimal|full)
	@[ -n "$${ATLAS_OBS_MODE:-}" ] || { echo "set ATLAS_OBS_MODE=minimal|full" >&2; exit 2; }
	@./ops/observability/scripts/install_obs_pack.sh

ops-obs-mode-minimal: ## Install observability pack in minimal mode
	@ATLAS_OBS_MODE=minimal ./ops/observability/scripts/install_obs_pack.sh

ops-obs-mode-full: ## Install observability pack in full mode
	@ATLAS_OBS_MODE=full ./ops/observability/scripts/install_obs_pack.sh

ops-observability-pack-tests: ## Run observability pack conformance tests
	@./ops/observability/tests/run_all.sh

ops-observability-pack-lint: ## Run observability pack lint-only contract checks
	@./ops/observability/tests/test_pack_contracts.sh

ops-ci: ## Nightly ops pipeline: up/deploy/warm/tests/ops/load/drills/report
	@SHELLCHECK_STRICT=1 $(MAKE) ops-shellcheck
	@$(MAKE) ops-up
	@$(MAKE) ops-reset
	@$(MAKE) ops-publish-medium
	@$(MAKE) ops-deploy
	@$(MAKE) ops-warm
	@$(MAKE) ops-k8s-tests
	@$(MAKE) ops-load-smoke
	@$(MAKE) ops-drill-store-outage
	@$(MAKE) ops-drill-prom-outage
	@$(MAKE) ops-drill-otel-outage
	@$(MAKE) ops-drill-corruption
	@$(MAKE) ops-report

ops-ci-nightly: ## Compatibility alias for ops-ci
	@$(MAKE) ops-ci


ops-full: ## Full local ops flow: up->deploy->warm->smoke->k8s-tests->load-smoke->obs-validate
	@$(MAKE) ops-up
	@$(MAKE) ops-deploy
	@$(MAKE) ops-publish
	@$(MAKE) ops-warm
	@$(MAKE) ops-smoke || $(MAKE) ops-smoke
	@$(MAKE) ops-k8s-tests
	@$(MAKE) ops-load-smoke
	@$(MAKE) ops-observability-validate

ops-full-pr: ## Lightweight PR flow for ops validation
	@$(MAKE) ops-up
	@$(MAKE) ops-deploy
	@$(MAKE) ops-publish
	@$(MAKE) ops-warm
	@$(MAKE) ops-smoke || $(MAKE) ops-smoke
	@ATLAS_E2E_TEST_GROUP=install $(MAKE) ops-k8s-tests
	@$(MAKE) ops-load-ci

ops-full-nightly: ## Nightly full flow incl. realdata and full load suites
	@$(MAKE) ops-up
	@$(MAKE) ops-deploy
	@$(MAKE) ops-publish
	@$(MAKE) ops-warm
	@$(MAKE) ops-smoke || $(MAKE) ops-smoke
	@$(MAKE) ops-k8s-tests
	@$(MAKE) ops-realdata
	@$(MAKE) ops-load-nightly
	@$(MAKE) ops-observability-validate

ops-clean: ## Local cleanup of ops outputs and test namespaces
	@kubectl delete ns "$${ATLAS_NS}" --ignore-not-found >/dev/null 2>&1 || true
	@rm -rf artifacts/perf/results artifacts/ops artifacts/e2e-datasets artifacts/e2e-store

# Compatibility aliases (pre-ops.mk surface)
e2e-local:
	@$(MAKE) ops-up
	@$(MAKE) ops-reset
	@$(MAKE) ops-publish-medium
	@$(MAKE) ops-deploy
	@$(MAKE) ops-warm
	@$(MAKE) ops-smoke
	@$(MAKE) ops-metrics-check

e2e-k8s-install-gate:
	@$(MAKE) ops-up
	@./ops/e2e/k8s/tests/test_install.sh

e2e-k8s-suite:
	@$(MAKE) ops-up
	@$(MAKE) ops-k8s-tests

e2e-perf:
	@./ops/load/scripts/run_e2e_perf.sh

e2e-realdata:
	@./ops/e2e/realdata/run_all.sh

observability-check:
	@$(MAKE) ops-metrics-check
	@$(MAKE) ops-traces-check
	@cargo test -p bijux-atlas-server --test observability_contract --test logging_format
