SHELL := /bin/sh

# Ops SSOT targets

ops-up: ## Bring up local ops stack (kind + minio + prometheus + optional otel/redis)
	@$(MAKE) ops-kind-version-check
	@$(MAKE) ops-kubectl-version-check
	@$(MAKE) ops-helm-version-check
	@./ops/e2e/scripts/up.sh

ops-down: ## Tear down local ops stack
	@./ops/e2e/scripts/down.sh

ops-reset: ## Reset ops state (namespace/PV store data + local store artifacts)
	@kubectl delete ns "$${ATLAS_E2E_NAMESPACE:-atlas-e2e}" --ignore-not-found >/dev/null 2>&1 || true
	@kubectl delete pvc --all -n default >/dev/null 2>&1 || true
	@./ops/e2e/scripts/cleanup_store.sh

ops-publish-medium: ## Ingest + publish medium fixture dataset
	@./scripts/fixtures/fetch-medium.sh
	@./ops/e2e/scripts/publish_dataset.sh \
	  --gff3 ops/fixtures/medium/data/genes.gff3 \
	  --fasta ops/fixtures/medium/data/genome.fa \
	  --fai ops/fixtures/medium/data/genome.fa.fai \
	  --release 110 --species homo_sapiens --assembly GRCh38

ops-deploy: ## Deploy atlas chart into local cluster
	@./ops/e2e/scripts/deploy_atlas.sh

ops-warm: ## Run warmup workflow
	@./ops/e2e/scripts/warmup.sh

ops-soak: ## Run soak workflow (10-30 minutes)
	@./ops/e2e/scripts/soak.sh

ops-smoke: ## Run canonical API smoke queries
	@./ops/e2e/scripts/smoke_queries.sh

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
	@./scripts/observability/check_tracing_contract.py

ops-k8s-tests: ## Run k8s e2e suite
	@./ops/e2e/k8s/tests/run_all.sh

ops-k8s-template-tests: ## Run helm template/lint edge-case checks
	@./ops/e2e/k8s/tests/test_helm_templates.sh

ops-load-prereqs: ## Validate load harness prerequisites (k6 + endpoint)
	@./ops/load/scripts/check_prereqs.sh

ops-load-smoke: ## Run short load suite
	@$(MAKE) ops-k6-version-check
	@./scripts/perf/check_pinned_queries_lock.py
	@$(MAKE) ops-load-prereqs
	@./ops/load/scripts/run_suite.sh mixed.json artifacts/perf/results
	@./scripts/perf/validate_results.py artifacts/perf/results

ops-load-full: ## Run nightly/full load suites
	@$(MAKE) ops-k6-version-check
	@./scripts/perf/check_pinned_queries_lock.py
	@./scripts/perf/run_nightly_perf.sh

ops-drill-store-outage: ## Run store outage drill under load
	@./ops/load/scripts/run_suite.sh store-outage-mid-spike.json artifacts/perf/results
	@./ops/observability/scripts/drill_store_outage.sh

ops-drill-alerts: ## Run alert drill checks against configured rules
	@./ops/observability/scripts/drill_alerts.sh

ops-drill-overload: ## Verify overload signal drill assertions
	@./ops/observability/scripts/drill_overload.sh

ops-drill-memory-growth: ## Verify memory-growth drill assertions
	@./ops/observability/scripts/drill_memory_growth.sh

ops-drill-corruption: ## Run corruption handling drill
	@cargo test -p bijux-atlas-server cache_manager_tests::chaos_mode_random_byte_corruption_never_serves_results -- --exact

ops-drill-pod-churn: ## Run pod churn drill while service handles load
	@./ops/e2e/k8s/tests/drill_pod_churn.sh

ops-drill-upgrade: ## Run upgrade drill and verify semantic stability
	@./ops/e2e/realdata/upgrade_drill.sh

ops-drill-rollback: ## Run rollback drill and verify semantic stability
	@./ops/e2e/realdata/rollback_drill.sh

ops-report: ## Gather ops evidence into artifacts/ops/<timestamp>/
	@ts=$$(date +%Y%m%d-%H%M%S); \
	out="artifacts/ops/$$ts"; \
	mkdir -p "$$out"/{logs,perf,metrics}; \
	kubectl get pods -A -o wide > "$$out/logs/pods.txt" 2>/dev/null || true; \
	kubectl get events -A --sort-by=.lastTimestamp > "$$out/logs/events.txt" 2>/dev/null || true; \
	kubectl logs -n "$${ATLAS_E2E_NAMESPACE:-atlas-e2e}" -l app.kubernetes.io/name=bijux-atlas --tail=2000 > "$$out/logs/atlas.log" 2>/dev/null || true; \
	cp -R artifacts/perf/results "$$out/perf/" 2>/dev/null || true; \
	curl -fsS "$${ATLAS_BASE_URL:-http://127.0.0.1:8080}/metrics" > "$$out/metrics/metrics.txt" 2>/dev/null || true; \
	echo "ops report written to $$out"

ops-slo-burn: ## Compute SLO burn artifact from k6 score + metrics snapshot
	@python3 ./ops/observability/scripts/compute_slo_burn.py

ops-script-coverage: ## Validate every ops/**/scripts entrypoint is exposed via make
	@./scripts/layout/check_ops_script_targets.sh

ops-kind-version-check: ## Validate pinned kind version from ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py kind

ops-k6-version-check: ## Validate pinned k6 version from ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py k6

ops-helm-version-check: ## Validate pinned helm version from ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py helm

ops-kubectl-version-check: ## Validate pinned kubectl version from ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py kubectl

ops-perf-prepare-store: ## Perf helper: prepare local perf store fixture
	@./scripts/perf/prepare_perf_store.sh

ops-perf-e2e: ## Perf helper: run e2e perf suite
	@./scripts/perf/run_e2e_perf.sh

ops-perf-nightly: ## Perf helper: run nightly perf suite
	@./scripts/perf/run_nightly_perf.sh

ops-perf-cold-start: ## Perf helper: run cold-start benchmark
	@./scripts/perf/cold_start_benchmark.sh

ops-perf-cold-start-prefetch-5pods: ## Perf helper: run 5-pod prefetch cold-start benchmark
	@./scripts/perf/cold_start_prefetch_5pods.sh

ops-perf-compare-redis: ## Perf helper: compare Redis-on vs Redis-off perf runs
	@./scripts/perf/compare_redis.sh

ops-baseline-policy-check: ## Enforce explicit approval policy for baseline updates
	@./scripts/perf/check_baseline_update_policy.sh

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
	@$(MAKE) ops-dashboards-validate
	@$(MAKE) ops-alerts-validate
	@./scripts/observability/check_metrics_contract.py
	@./scripts/observability/check_tracing_contract.py
	@./ops/observability/scripts/snapshot_metrics.sh
	@./ops/observability/scripts/check_metric_cardinality.py
	@python3 ./ops/observability/scripts/validate_logs_schema.py

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

ops-ci: ## Nightly ops pipeline: up/deploy/warm/tests/load/drills/report
	@$(MAKE) ops-up
	@$(MAKE) ops-reset
	@$(MAKE) ops-publish-medium
	@$(MAKE) ops-deploy
	@$(MAKE) ops-warm
	@$(MAKE) ops-k8s-tests
	@$(MAKE) ops-load-smoke
	@$(MAKE) ops-drill-store-outage
	@$(MAKE) ops-drill-corruption
	@$(MAKE) ops-report

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
	@./scripts/perf/run_e2e_perf.sh

e2e-realdata:
	@./ops/e2e/realdata/run_all.sh

observability-check:
	@$(MAKE) ops-metrics-check
	@$(MAKE) ops-traces-check
	@cargo test -p bijux-atlas-server --test observability_contract --test logging_format
