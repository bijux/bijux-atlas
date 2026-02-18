SHELL := /bin/sh

# Ops SSOT targets
OPS_ENV_SCHEMA ?= configs/ops/env.schema.json

ops-env-validate: ## Validate canonical ops environment contract against schema
	@python3 ./scripts/layout/validate_ops_env.py --schema "$(OPS_ENV_SCHEMA)"

ops-env-print: ## Print canonical ops environment settings
	@python3 ./scripts/layout/validate_ops_env.py --schema "$(OPS_ENV_SCHEMA)" --print --format json

ops-doctor: ## Validate and print pinned ops tool versions and canonical env
	@$(MAKE) -s ops-tools-check
	@$(MAKE) -s ops-tools-print
	@$(MAKE) -s ops-env-print

OPS_DEPLOY_PROFILE ?= local

ops-stack-up: ## Bring up stack components only (kind + stack manifests)
	@$(MAKE) -s ops-env-validate
	@./ops/stack/kind/context_guard.sh
	@./ops/stack/kind/namespace_guard.sh
	@$(MAKE) ops-kind-up
	@$(MAKE) ops-kind-version-check
	@$(MAKE) ops-kubectl-version-check
	@$(MAKE) ops-helm-version-check
	@if [ "$${ATLAS_KIND_REGISTRY_ENABLE:-0}" = "1" ]; then $(MAKE) ops-kind-registry-up; fi
	@./ops/e2e/scripts/up.sh
	@$(MAKE) ops-cluster-sanity

ops-up: ## Bring up local ops stack (kind + minio + prometheus + optional otel/redis)
	@$(MAKE) ops-stack-up

ops-cluster-sanity: ## Validate cluster node/dns/storageclass sanity
	@./ops/e2e/k8s/tests/test_cluster_sanity.sh

ops-stack-down: ## Tear down stack components and cluster
	@$(MAKE) -s ops-env-validate
	@./ops/stack/kind/context_guard.sh
	@./ops/stack/kind/namespace_guard.sh
	@./ops/e2e/scripts/down.sh

ops-down: ## Tear down local ops stack
	@$(MAKE) ops-stack-down

ops-stack-validate: ## Validate stack manifests and formatting drift
	@./scripts/layout/check_stack_manifest_consolidation.sh
	@./scripts/layout/check_ops_stack_order.sh
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
	@cat ops/stack/versions.json

ops-kind-up: ## Create kind cluster with fixed name and selected profile
	@./ops/stack/kind/up.sh

ops-kind-down: ## Delete kind cluster with fixed name
	@./ops/stack/kind/down.sh

ops-kind-reset: ## Delete and recreate kind cluster
	@./ops/stack/kind/reset.sh

ops-kind-metrics-server-up: ## Install metrics-server for HPA/resource tests (optional)
	@./ops/stack/kind/metrics-server-up.sh

ops-kind-registry-up: ## Bring up local registry and connect to kind network
	@./ops/stack/registry/up.sh

ops-kind-image-resolution-test: ## Validate atlas image is resolvable inside kind runtime
	@./ops/e2e/k8s/tests/test_kind_image_resolution.sh

ops-kind-disk-pressure: ## Simulate node disk pressure (use MODE=clean to remove)
	@./ops/stack/faults/fill-node-disk.sh "$${MODE:-fill}"

ops-kind-cpu-throttle: ## Simulate cpu throttle pressure in cluster workloads
	@./ops/stack/faults/cpu-throttle.sh

ops-kind-network-latency: ## Simulate store network latency via toxiproxy
	@./ops/stack/faults/toxiproxy-latency.sh "$${LATENCY_MS:-250}" "$${JITTER_MS:-25}"

ops-kind-context-guard: ## Refuse non-kind kubectl context unless ALLOW_NON_KIND=1
	@./ops/stack/kind/context_guard.sh

ops-kind-namespace-guard: ## Enforce ops namespace naming policy atlas-ops-*
	@./ops/stack/kind/namespace_guard.sh

ops-kind-cleanup-leftovers: ## Delete stale atlas-ops-* namespaces
	@./ops/stack/kind/cleanup_namespaces.sh

ops-kind-version-drift-test: ## Validate kind version matches pinned tool-versions
	@./ops/e2e/k8s/tests/test_kind_version_drift.sh

ops-kind-cluster-drift-check: ## Require ops contract marker update when cluster profile changes
	@./scripts/layout/check_kind_cluster_contract_drift.sh

ops-kind-validate: ## Validate kind substrate (context/namespace/sanity/registry/image/version)
	@$(MAKE) ops-kind-context-guard
	@$(MAKE) ops-kind-namespace-guard
	@$(MAKE) ops-cluster-sanity
	@if [ "$${ATLAS_KIND_REGISTRY_ENABLE:-0}" = "1" ]; then $(MAKE) ops-kind-registry-up; fi
	@$(MAKE) ops-kind-image-resolution-test
	@$(MAKE) ops-kind-version-drift-test
	@$(MAKE) ops-kind-cluster-drift-check

ops-minio-up: ## Install minio stack component
	@kubectl apply -f ./ops/stack/minio/minio.yaml
	@./ops/stack/minio/bootstrap.sh

ops-minio-down: ## Uninstall minio stack component
	@kubectl delete -f ./ops/stack/minio/minio.yaml --ignore-not-found >/dev/null 2>&1 || true
	@kubectl -n "$${ATLAS_E2E_NAMESPACE:-atlas-e2e}" delete pod minio-bootstrap --ignore-not-found >/dev/null 2>&1 || true

ops-minio-reset: ## Reinstall minio stack component deterministically
	@$(MAKE) ops-minio-down
	@$(MAKE) ops-minio-up

ops-minio-ready: ## Validate minio readiness and endpoint health
	@ns="$${ATLAS_E2E_NAMESPACE:-atlas-e2e}"; \
	kubectl -n "$$ns" wait --for=condition=available deploy/minio --timeout="$${OPS_WAIT_TIMEOUT:-180s}"; \
	pid_file="$$(mktemp)"; \
	( kubectl -n "$$ns" port-forward svc/minio 19000:9000 >/dev/null 2>&1 & echo $$! > "$$pid_file" ); \
	pid="$$(cat "$$pid_file")"; rm -f "$$pid_file"; \
	trap 'kill "$$pid" >/dev/null 2>&1 || true' EXIT INT TERM; \
	for _ in $$(seq 1 30); do \
	  if curl -fsS http://127.0.0.1:19000/minio/health/ready >/dev/null; then \
	    echo "minio endpoint ready"; \
	    exit 0; \
	  fi; \
	  sleep 1; \
	done; \
	echo "minio endpoint not ready" >&2; \
	exit 1

ops-minio-bucket-check: ## Validate bootstrap bucket exists in minio
	@ns="$${ATLAS_E2E_NAMESPACE:-atlas-e2e}"; \
	bucket="$${MINIO_BUCKET:-atlas-artifacts}"; \
	user="$${MINIO_ROOT_USER:-minioadmin}"; \
	pass="$${MINIO_ROOT_PASSWORD:-minioadmin}"; \
	kubectl -n "$$ns" delete pod minio-bucket-check --ignore-not-found >/dev/null 2>&1 || true; \
	kubectl -n "$$ns" run minio-bucket-check \
	  --image=minio/mc:RELEASE.2025-01-17T23-25-50Z \
	  --restart=Never \
	  --rm -i --command -- /bin/sh -ceu " \
mc alias set local 'http://minio.$$ns.svc.cluster.local:9000' '$$user' '$$pass'; \
mc ls local/$$bucket >/dev/null"

ops-minio-creds-rotate-drill: ## Optional drill: rotate minio creds and verify service recovers
	@./ops/stack/minio/rotate_creds.sh

ops-prom-up: ## Install prometheus stack component
	@kubectl apply -f ./ops/stack/prometheus/prometheus.yaml

ops-prom-down: ## Uninstall prometheus stack component
	@kubectl delete -f ./ops/stack/prometheus/prometheus.yaml --ignore-not-found >/dev/null 2>&1 || true

ops-prom-ready: ## Validate prometheus readiness
	@kubectl -n "$${ATLAS_E2E_NAMESPACE:-atlas-e2e}" wait --for=condition=available deploy/prometheus --timeout="$${OPS_WAIT_TIMEOUT:-180s}"

ops-prom-scrape-atlas-check: ## Validate prometheus scrape target for atlas
	@./ops/e2e/k8s/tests/test_prom_scrape.sh

ops-grafana-up: ## Install grafana stack component
	@kubectl apply -f ./ops/stack/grafana/grafana.yaml

ops-grafana-down: ## Uninstall grafana stack component
	@kubectl delete -f ./ops/stack/grafana/grafana.yaml --ignore-not-found >/dev/null 2>&1 || true

ops-grafana-ready: ## Validate grafana readiness
	@kubectl -n "$${ATLAS_E2E_NAMESPACE:-atlas-e2e}" wait --for=condition=available deploy/grafana --timeout="$${OPS_WAIT_TIMEOUT:-180s}"

ops-grafana-datasource-check: ## Validate grafana datasource points to prometheus service
	@kubectl -n "$${ATLAS_E2E_NAMESPACE:-atlas-e2e}" get configmap grafana-datasources -o yaml | grep -Eq "http://prometheus\\.atlas-e2e\\.svc\\.cluster\\.local:9090"

ops-grafana-dashboards-check: ## Validate grafana dashboard configmaps are provisioned
	@ns="$${ATLAS_E2E_NAMESPACE:-atlas-e2e}"; \
	kubectl -n "$$ns" get configmap grafana-dashboard-provider >/dev/null; \
	kubectl -n "$$ns" get configmap grafana-dashboard-atlas >/dev/null

ops-otel-up: ## Install otel collector stack component
	@kubectl apply -f ./ops/stack/otel/otel-collector.yaml

ops-otel-down: ## Uninstall otel collector stack component
	@kubectl delete -f ./ops/stack/otel/otel-collector.yaml --ignore-not-found >/dev/null 2>&1 || true

ops-otel-spans-check: ## Validate otel collector receives spans when enabled
	@./ops/e2e/k8s/tests/test_otel_spans.sh

ops-redis-up: ## Install redis stack component (optional)
	@kubectl apply -f ./ops/stack/redis/redis.yaml

ops-redis-down: ## Uninstall redis stack component
	@kubectl delete -f ./ops/stack/redis/redis.yaml --ignore-not-found >/dev/null 2>&1 || true

ops-redis-optional-check: ## Validate atlas runs when redis is absent
	@./ops/e2e/k8s/tests/test_redis_optional.sh

ops-redis-used-check: ## Validate redis usage evidence when enabled
	@ATLAS_E2E_ENABLE_REDIS=1 ./ops/e2e/k8s/tests/test_redis_backend_metric.sh

ops-toxi-up: ## Install toxiproxy stack component (optional)
	@kubectl apply -f ./ops/stack/toxiproxy/toxiproxy.yaml
	@./ops/stack/toxiproxy/bootstrap.sh

ops-toxi-down: ## Uninstall toxiproxy stack component
	@kubectl delete -f ./ops/stack/toxiproxy/toxiproxy.yaml --ignore-not-found >/dev/null 2>&1 || true
	@kubectl -n "$${ATLAS_E2E_NAMESPACE:-atlas-e2e}" delete pod toxiproxy-bootstrap --ignore-not-found >/dev/null 2>&1 || true

ops-toxi-latency-inject: ## Inject store latency through toxiproxy
	@./ops/stack/faults/toxiproxy-latency.sh "$${LATENCY_MS:-250}" "$${JITTER_MS:-25}"

ops-toxi-cut-store: ## Cut or restore store connection (MODE=on|off)
	@./ops/stack/faults/block-minio.sh "$${MODE:-on}"

ops-stack-order-check: ## Validate stack install/uninstall order contract
	@./scripts/layout/check_ops_stack_order.sh

ops-stack-security-check: ## Validate stack security defaults (no privileged containers)
	@ns="$${ATLAS_E2E_NAMESPACE:-atlas-e2e}"; \
	violations="$$(kubectl -n "$$ns" get pods -o jsonpath='{range .items[*]}{.metadata.name}{" "}{range .spec.containers[*]}{.securityContext.privileged}{" "}{end}{"\n"}{end}' 2>/dev/null | awk '$$0 ~ / true / {print $$1}')"; \
	if [ -n "$$violations" ]; then \
	  echo "privileged stack pods found in $$ns:" >&2; \
	  echo "$$violations" >&2; \
	  exit 1; \
	fi; \
	echo "stack security defaults check passed"

ops-stack-uninstall: ## Uninstall stack resources and cluster
	@./ops/stack/scripts/uninstall.sh

ops-stack-slow-store: ## Enable slow-store mode via toxiproxy latency
	@ATLAS_E2E_ENABLE_TOXIPROXY=1 $(MAKE) ops-stack-up
	@./ops/stack/toxiproxy/enable_slow_store.sh

ops-reset: ## Reset ops state (namespace/PV store data + local store artifacts)
	@if [ "$${CONFIRM_RESET:-}" != "YES" ]; then \
	  echo "ops-reset is destructive; rerun with CONFIRM_RESET=YES" >&2; \
	  exit 2; \
	fi
	@if [ -n "$${CI:-}" ] && [ "$${OPS_ALLOW_PROMPT:-0}" = "1" ]; then \
	  echo "interactive prompts are forbidden in CI ops runs" >&2; \
	  exit 2; \
	fi
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

ops-deploy: ## Deploy atlas chart into local cluster (PROFILE=local|offline|perf)
	@$(MAKE) -s ops-env-validate
	@profile="$${PROFILE:-$(OPS_DEPLOY_PROFILE)}"; \
	case "$$profile" in \
	  local) export ATLAS_VALUES_FILE="$(ATLAS_VALUES_FILE)" ;; \
	  offline) export ATLAS_VALUES_FILE="$(ATLAS_OFFLINE_VALUES_FILE)" ;; \
	  perf) export ATLAS_VALUES_FILE="$(ATLAS_PERF_VALUES_FILE)" ;; \
	  *) echo "invalid PROFILE=$$profile (expected: local|offline|perf)" >&2; exit 2 ;; \
	esac; \
	export ATLAS_E2E_VALUES_FILE="$$ATLAS_VALUES_FILE"; \
	$(MAKE) -s ops-values-validate; \
	$(MAKE) -s ops-chart-render-diff; \
	$(MAKE) -s docker-build; \
	./ops/e2e/scripts/deploy_atlas.sh

ops-undeploy: ## Uninstall atlas helm release from namespace
	@ns="$${ATLAS_E2E_NAMESPACE:-$${ATLAS_NS:-atlas-e2e}}"; \
	release="$${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"; \
	helm -n "$$ns" uninstall "$$release" >/dev/null 2>&1 || true

ops-redeploy: ## Uninstall and deploy atlas chart again
	@$(MAKE) ops-undeploy
	@$(MAKE) ops-deploy PROFILE="$${PROFILE:-$(OPS_DEPLOY_PROFILE)}"

ops-offline: ## Deploy atlas in cached-only offline profile
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-deploy PROFILE=offline

ops-perf: ## Deploy atlas in perf profile and run load smoke
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-deploy PROFILE=perf
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
	@python3 ./ops/smoke/generate_report.py
	@python3 ./scripts/docs/check_openapi_examples.py
	@$(MAKE) ops-metrics-check
	@./ops/observability/scripts/snapshot_metrics.sh
	@./ops/observability/scripts/snapshot_traces.sh
	@mkdir -p artifacts/ops/observability && cp ops/observability/grafana/atlas-observability-dashboard.json artifacts/ops/observability/dashboard.snapshot.json

ops-metrics-check: ## Validate runtime metrics and observability contracts
	@./ops/e2e/scripts/verify_metrics.sh
	@./scripts/public/observability/check_metrics_contract.py
	@./scripts/public/observability/check_dashboard_contract.py
	@./scripts/public/observability/check_alerts_contract.py
	@./scripts/public/observability/lint_runbooks.py
	@./scripts/public/observability/check_runtime_metrics.py
	@./ops/observability/scripts/snapshot_metrics.sh
	@./ops/observability/scripts/check_metric_cardinality.py
	@python3 ./ops/observability/scripts/validate_logs_schema.py

ops-traces-check: ## Validate trace signal (when OTEL enabled)
	@./ops/e2e/scripts/verify_traces.sh
	@if [ "$${ATLAS_E2E_ENABLE_OTEL:-0}" = "1" ]; then ./scripts/public/observability/check_tracing_contract.py; else echo "trace contract skipped (ATLAS_E2E_ENABLE_OTEL=0)"; fi

ops-k8s-tests: ## Run k8s e2e suite
	@$(MAKE) -s ops-env-validate
	@python3 ./scripts/ops/check_k8s_test_contract.py
	@SHELLCHECK_STRICT=1 $(MAKE) ops-shellcheck
	@lock_dir="artifacts/ops/locks/ops-k8s-tests.lock"; \
	mkdir -p "artifacts/ops/locks"; \
	if ! mkdir "$$lock_dir" 2>/dev/null; then \
	  echo "ops-k8s-tests is already running (lock: $$lock_dir)" >&2; \
	  exit 1; \
	fi; \
	trap 'rmdir "$$lock_dir" >/dev/null 2>&1 || true' EXIT INT TERM; \
	group_args=""; \
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
	@./ops/load/scripts/score_k6.py || true
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

ops-drill-upgrade-under-load: ## Run upgrade drill under load
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-load-under-rollout

ops-drill-rollback-under-load: ## Run rollback drill under load
	@$(MAKE) -s ops-env-validate
	@$(MAKE) ops-load-under-rollback

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

ops-drill-rate-limit: ## Run abuse pattern and assert stable 429 behavior
	@$(MAKE) -s ops-env-validate
	@./ops/observability/scripts/drill_overload.sh

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
	cp -R artifacts/ops/e2e/k6 "$$out/perf/k6/" 2>/dev/null || true; \
	cp -R "$$out/smoke/report.md" "$$out/perf/smoke-report.md" 2>/dev/null || true; \
	curl -fsS "$${ATLAS_BASE_URL:-http://127.0.0.1:8080}/metrics" > "$$out/metrics/metrics.txt" 2>/dev/null || true; \
	./ops/e2e/scripts/write_metadata.sh "$$out"; \
	python3 ./ops/report/generate.py --run-dir "$$out" --schema ops/report/schema.json; \
	echo "ops report written to $$out"; \
	RUN_ID="$${OPS_RUN_ID}" OUT_DIR="$$out/bundle" ./scripts/public/report_bundle.sh >/dev/null; \
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

ops-jq-version-check: ## Validate pinned jq version from configs/ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py jq

ops-yq-version-check: ## Validate pinned yq version from configs/ops/tool-versions.json
	@python3 ./scripts/layout/check_tool_versions.py yq

ops-tools-check: ## Validate all pinned ops tools versions
	@$(MAKE) ops-kind-version-check
	@$(MAKE) ops-k6-version-check
	@$(MAKE) ops-helm-version-check
	@$(MAKE) ops-kubectl-version-check
	@$(MAKE) ops-jq-version-check
	@$(MAKE) ops-yq-version-check

ops-tools-print: ## Print tool paths and local versions
	@printf 'kind: path=%s version=%s\n' "$$(command -v kind 2>/dev/null || echo missing)" "$$(kind version 2>/dev/null || echo missing)"
	@printf 'kubectl: path=%s version=%s\n' "$$(command -v kubectl 2>/dev/null || echo missing)" "$$(kubectl version --client 2>/dev/null | head -n1 || echo missing)"
	@printf 'helm: path=%s version=%s\n' "$$(command -v helm 2>/dev/null || echo missing)" "$$(helm version --short 2>/dev/null || echo missing)"
	@printf 'k6: path=%s version=%s\n' "$$(command -v k6 2>/dev/null || echo missing)" "$$(k6 version 2>/dev/null | head -n1 || echo missing)"
	@printf 'jq: path=%s version=%s\n' "$$(command -v jq 2>/dev/null || echo missing)" "$$(jq --version 2>/dev/null || echo missing)"
	@printf 'yq: path=%s version=%s\n' "$$(command -v yq 2>/dev/null || echo missing)" "$$(yq --version 2>/dev/null | head -n1 || echo missing)"

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
	@./scripts/public/openapi-diff-check.sh
	@python3 ./scripts/docs/check_openapi_examples.py

ops-chart-render-diff: ## Ensure chart render is deterministic for local profile
	@tmp_a="$$(mktemp)"; tmp_b="$$(mktemp)"; \
	helm template atlas ops/k8s/charts/bijux-atlas -f ops/k8s/values/local.yaml > "$$tmp_a"; \
	helm template atlas ops/k8s/charts/bijux-atlas -f ops/k8s/values/local.yaml > "$$tmp_b"; \
	diff -u "$$tmp_a" "$$tmp_b" >/dev/null; \
	rm -f "$$tmp_a" "$$tmp_b"

ops-dashboards-validate: ## Validate dashboard references against metrics contract
	@./scripts/public/observability/check_dashboard_contract.py

ops-alerts-validate: ## Validate alert rules and contract coverage
	@./scripts/public/observability/check_alerts_contract.py

ops-observability-validate: ## Validate observability assets/contracts end-to-end
	@set -e; \
	trap 'out="artifacts/ops/observability/validate-fail-$$(date +%Y%m%d-%H%M%S)"; mkdir -p "$$out"; kubectl get pods -A -o wide > "$$out/pods.txt" 2>/dev/null || true; kubectl get events -A --sort-by=.lastTimestamp > "$$out/events.txt" 2>/dev/null || true; cp -f ops/observability/grafana/atlas-observability-dashboard.json "$$out/dashboard.json" 2>/dev/null || true; cp -f ops/observability/alerts/atlas-alert-rules.yaml "$$out/alerts.yaml" 2>/dev/null || true; echo "observability validation failed, artifacts: $$out" >&2' ERR; \
	$(MAKE) ops-dashboards-validate; \
	$(MAKE) ops-alerts-validate; \
	./scripts/public/observability/check_metrics_contract.py; \
	if [ "$${ATLAS_E2E_ENABLE_OTEL:-0}" = "1" ]; then ./scripts/public/observability/check_tracing_contract.py; else echo "trace contract skipped (ATLAS_E2E_ENABLE_OTEL=0)"; fi; \
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

ops-open-grafana: ## Print local ops service URLs
	@./ops/ui/print_urls.sh

ops-ci: ## Nightly ops pipeline: up/deploy/warm/tests/ops/load/drills/report
	@SHELLCHECK_STRICT=1 $(MAKE) ops-shellcheck
	@$(MAKE) ops-up
	@CONFIRM_RESET=YES $(MAKE) ops-reset
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


ops-full: ## Full local ops flow (OPS_MODE=fast|full, OPS_DRY_RUN=1 supported)
	@set -e; \
	trap 'echo "ops-full failed; collecting failure bundle"; $(MAKE) ops-stack-health-report; $(MAKE) ops-report' ERR; \
	if [ "$${OPS_DRY_RUN:-0}" = "1" ]; then \
	  echo "DRY-RUN ops-full mode=$${OPS_MODE:-fast} run_id=$${OPS_RUN_ID} ns=$${ATLAS_NS}"; \
	  echo "$(MAKE) ops-up && $(MAKE) ops-deploy && $(MAKE) ops-publish && $(MAKE) ops-warm && $(MAKE) ops-smoke && $(MAKE) ops-k8s-tests"; \
	  if [ "$${OPS_MODE:-fast}" = "full" ]; then \
	    echo "$(MAKE) ops-load-full && $(MAKE) ops-realdata && $(MAKE) ops-observability-validate"; \
	  else \
	    echo "$(MAKE) ops-load-smoke && $(MAKE) ops-observability-validate"; \
	  fi; \
	  exit 0; \
	fi; \
	$(MAKE) ops-up; \
	$(MAKE) ops-deploy; \
	$(MAKE) ops-publish; \
	$(MAKE) ops-warm; \
	$(MAKE) ops-smoke || $(MAKE) ops-smoke; \
	$(MAKE) ops-k8s-tests; \
	if [ "$${OPS_MODE:-fast}" = "full" ]; then \
	  $(MAKE) ops-load-full; \
	  $(MAKE) ops-realdata; \
	else \
	  $(MAKE) ops-load-smoke; \
	fi; \
	$(MAKE) ops-observability-validate

ops-full-pr: ## Lightweight PR flow for ops validation
	@set -e; \
	trap 'echo "ops-full-pr failed; collecting failure bundle"; $(MAKE) ops-stack-health-report; $(MAKE) ops-report' ERR; \
	$(MAKE) ops-up; \
	$(MAKE) ops-deploy; \
	$(MAKE) ops-publish; \
	$(MAKE) ops-warm; \
	$(MAKE) ops-smoke || $(MAKE) ops-smoke; \
	ATLAS_E2E_TEST_GROUP=install $(MAKE) ops-k8s-tests; \
	$(MAKE) ops-load-ci

ops-full-nightly: ## Nightly full flow incl. realdata and full load suites
	@set -e; \
	trap 'echo "ops-full-nightly failed; collecting failure bundle"; $(MAKE) ops-stack-health-report; $(MAKE) ops-report' ERR; \
	$(MAKE) ops-up; \
	$(MAKE) ops-deploy; \
	$(MAKE) ops-publish; \
	$(MAKE) ops-warm; \
	$(MAKE) ops-smoke || $(MAKE) ops-smoke; \
	$(MAKE) ops-k8s-tests; \
	$(MAKE) ops-realdata; \
	$(MAKE) ops-load-nightly; \
	$(MAKE) ops-observability-validate

ops-idempotency-check: ## Enforce idempotent ops-full rerun contract
	@OPS_RUN_ID= OPS_NAMESPACE= $(MAKE) OPS_MODE=fast ops-full
	@sleep 1
	@OPS_RUN_ID= OPS_NAMESPACE= $(MAKE) OPS_MODE=fast ops-full

ops-clean: ## Local cleanup of ops outputs and test namespaces
	@days="$${OPS_RETENTION_DAYS:-7}"; \
	kubectl delete ns "$${ATLAS_NS}" --ignore-not-found >/dev/null 2>&1 || true; \
	find artifacts/ops -mindepth 1 -maxdepth 1 -type d -mtime +$$days -exec rm -rf {} + 2>/dev/null || true; \
	rm -rf artifacts/perf/results artifacts/e2e-datasets artifacts/e2e-store

# Compatibility aliases (pre-ops.mk surface)
e2e-local:
	@$(MAKE) ops-up
	@CONFIRM_RESET=YES $(MAKE) ops-reset
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
