SHELL := /bin/sh

# Ops SSOT targets

ops-up: ## Bring up local ops stack (kind + minio + prometheus + optional otel/redis)
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

ops-smoke: ## Run canonical API smoke queries
	@./ops/e2e/scripts/smoke_queries.sh

ops-metrics-check: ## Validate runtime metrics and observability contracts
	@./ops/e2e/scripts/verify_metrics.sh
	@./scripts/observability/check_metrics_contract.py
	@./scripts/observability/check_dashboard_contract.py
	@./scripts/observability/check_alerts_contract.py
	@./scripts/observability/lint_runbooks.py
	@./scripts/observability/check_runtime_metrics.py

ops-traces-check: ## Validate trace signal (when OTEL enabled)
	@./ops/e2e/scripts/verify_traces.sh
	@./scripts/observability/check_tracing_contract.py

ops-k8s-tests: ## Run k8s e2e suite
	@./ops/e2e/k8s/tests/run_all.sh

ops-load-smoke: ## Run short load suite
	@./scripts/perf/run_suite.sh mixed_80_20.js artifacts/perf/results

ops-load-full: ## Run nightly/full load suites
	@./scripts/perf/run_nightly_perf.sh

ops-drill-store-outage: ## Run store outage drill under load
	@./scripts/perf/run_suite.sh store_outage_mid_spike.js artifacts/perf/results

ops-drill-corruption: ## Run corruption handling drill
	@cargo test -p bijux-atlas-server cache_manager_tests::chaos_mode_random_byte_corruption_never_serves_results -- --exact

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
