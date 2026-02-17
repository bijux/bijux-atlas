#!/usr/bin/env bash
set -euo pipefail

DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
: "${ATLAS_E2E_NAMESPACE:=atlas-e2e-$(date +%s)}"
export ATLAS_E2E_NAMESPACE

run() { "$DIR/$1"; }

for t in \
  test_cluster_sanity.sh \
  test_values_contract.sh \
  test_defaults_safe.sh \
  test_chart_drift.sh \
  test_helm_templates.sh \
  test_rbac_minimalism.sh \
  test_install.sh \
  test_networkpolicy.sh \
  test_secrets.sh \
  test_configmap.sh \
  test_cached_only_mode.sh \
  test_pdb.sh \
  test_hpa.sh \
  test_rollout.sh \
  test_rollback.sh \
  test_warmup_job.sh \
  test_catalog_publish_job.sh \
  test_readiness_semantics.sh \
  test_resource_limits.sh \
  test_service_monitor.sh \
  test_prometheus_rule.sh \
  test_minio_bootstrap_idempotent.sh \
  test_minio_bucket_policy.sh \
  test_minio_reachable.sh \
  test_prom_scrape.sh \
  test_otel_spans.sh \
  test_redis_optional.sh \
  test_redis_backend_metric.sh \
  test_toxiproxy_latency_drill.sh \
  test_logs_json.sh \
  test_liveness_under_load.sh \
  test_node_local_cache_profile.sh \
  test_storage_modes.sh \
  test_multi_registry_profile.sh \
  test_image_digest_pinning.sh \
  test_ingress_profile.sh \
  test_offline_profile.sh
 do
  run "$t"
 done
