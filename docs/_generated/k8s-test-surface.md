# K8s Test Surface

Generated from `ops/k8s/tests/manifest.json` and `ops/k8s/tests/suites.json`.

## Suites
- `api-protection` groups=admission-control,rate-limit,redis
- `full` groups=*
- `graceful-degradation` groups=load,readiness,resilience
- `resilience` groups=autoscaling,availability,pdb,resilience,rolling-restart
- `smoke` groups=autoscaling,install,observability,readiness,sanity

## Group -> Tests
### `admission-control`
- `checks/perf/degradation/test_admission_control_api_behavior.sh`
- `checks/perf/degradation/test_overload_graceful_degradation.sh`

### `autoscaling`
- `checks/autoscaling/contracts/test_hpa_behavior_contract.sh`
- `checks/autoscaling/contracts/test_hpa_enabled_requires_metrics_stack.sh`
- `checks/autoscaling/contracts/test_hpa_enabled_requires_resources.sh`
- `checks/autoscaling/contracts/test_hpa_max_replicas_cap.sh`
- `checks/autoscaling/contracts/test_hpa_metrics_names_contract.sh`
- `checks/autoscaling/contracts/test_metrics_pipeline_ready.sh`
- `checks/autoscaling/runtime/test_hpa.sh`
- `checks/autoscaling/runtime/test_hpa_disabled_mode.sh`
- `checks/autoscaling/runtime/test_hpa_misconfig_negative.sh`
- `checks/autoscaling/runtime/test_hpa_under_inflight_metric.sh`
- `checks/autoscaling/runtime/test_hpa_under_latency_metric.sh`
- `checks/autoscaling/runtime/test_overload_hpa_trend.sh`
- `checks/datasets/test_pdb.sh`
- `checks/datasets/test_pdb_required_when_replicas_gt1.sh`

### `availability`
- `checks/datasets/test_pdb.sh`
- `checks/datasets/test_pdb_required_when_replicas_gt1.sh`
- `checks/perf/degradation/test_pod_churn_under_load.sh`
- `checks/rollout/test_rollback.sh`
- `checks/rollout/test_rolling_restart_no_downtime.sh`
- `checks/rollout/test_rollout.sh`

### `chart`
- `checks/obs/contracts/test_chart_contract_required_fields.sh`
- `checks/obs/contracts/test_chart_drift.sh`
- `checks/obs/contracts/test_helm_repo_pinning.sh`
- `checks/obs/contracts/test_helm_templates.sh`
- `checks/obs/contracts/test_layer_contract_render.sh`
- `checks/obs/contracts/test_no_checksum_rollout.sh`
- `checks/obs/contracts/test_values_contract.sh`
- `checks/obs/contracts/test_values_minimums.sh`
- `checks/obs/contracts/test_values_profiles_are_valid.sh`
- `checks/obs/contracts/test_values_schema_strict.sh`
- `checks/security/test_defaults_safe.sh`

### `chart-contract`
- `checks/autoscaling/contracts/test_hpa_behavior_contract.sh`
- `checks/autoscaling/contracts/test_hpa_enabled_requires_metrics_stack.sh`
- `checks/autoscaling/contracts/test_hpa_enabled_requires_resources.sh`
- `checks/autoscaling/contracts/test_hpa_max_replicas_cap.sh`
- `checks/autoscaling/contracts/test_hpa_metrics_names_contract.sh`
- `checks/config/test_configmap.sh`
- `checks/config/test_configmap_keys_match_runtime_expected.sh`
- `checks/config/test_configmap_must_exist.sh`
- `checks/config/test_configmap_schema_completeness.sh`
- `checks/config/test_configmap_unknown_keys_rejected.sh`
- `checks/config/test_configmap_update_reload.sh`
- `checks/config/test_configmap_version_stamp.sh`
- `checks/config/test_deployment_envFrom_configmap.sh`
- `checks/datasets/test_pdb_required_when_replicas_gt1.sh`
- `checks/obs/contracts/test_no_checksum_rollout.sh`
- `checks/obs/contracts/test_values_minimums.sh`
- `checks/obs/contracts/test_values_profiles_are_valid.sh`
- `checks/obs/contracts/test_values_schema_strict.sh`
- `checks/obs/runtime/test_observability_objects_contract.sh`
- `checks/security/test_image_digest_policy.sh`

### `cluster`
- `checks/rollout/test_cluster_sanity.sh`
- `checks/rollout/test_kind_image_resolution.sh`
- `checks/rollout/test_kind_version_drift.sh`

### `config`
- `checks/config/test_configmap.sh`
- `checks/config/test_configmap_keys_match_runtime_expected.sh`
- `checks/config/test_configmap_must_exist.sh`
- `checks/config/test_configmap_schema_completeness.sh`
- `checks/config/test_configmap_unknown_keys_rejected.sh`
- `checks/config/test_configmap_update_reload.sh`
- `checks/config/test_configmap_version_stamp.sh`
- `checks/config/test_deployment_envFrom_configmap.sh`

### `contracts`
- `checks/obs/contracts/test_chart_contract_required_fields.sh`
- `checks/obs/contracts/test_layer_contract_render.sh`
- `checks/obs/contracts/test_values_contract.sh`
- `checks/obs/runtime/test_logs_json.sh`
- `checks/rollout/test_kind_version_drift.sh`

### `datasets`
- `checks/datasets/test_catalog_publish_job.sh`
- `checks/datasets/test_dataset_missing_behavior.sh`
- `checks/datasets/test_warmup_job.sh`
- `checks/storage/test_store_bootstrap_idempotent.sh`
- `checks/storage/test_store_bucket_policy.sh`
- `checks/storage/test_store_reachable.sh`

### `docs`
- `checks/config/test_configmap_schema_completeness.sh`

### `flake-sensitive`
- `checks/autoscaling/runtime/test_hpa.sh`

### `hpa`
- `checks/autoscaling/contracts/test_hpa_behavior_contract.sh`
- `checks/autoscaling/contracts/test_hpa_enabled_requires_metrics_stack.sh`
- `checks/autoscaling/contracts/test_hpa_enabled_requires_resources.sh`
- `checks/autoscaling/contracts/test_hpa_max_replicas_cap.sh`
- `checks/autoscaling/contracts/test_hpa_metrics_names_contract.sh`
- `checks/autoscaling/contracts/test_metrics_pipeline_ready.sh`
- `checks/autoscaling/runtime/test_hpa.sh`
- `checks/autoscaling/runtime/test_hpa_disabled_mode.sh`
- `checks/autoscaling/runtime/test_hpa_misconfig_negative.sh`
- `checks/autoscaling/runtime/test_hpa_under_inflight_metric.sh`
- `checks/autoscaling/runtime/test_hpa_under_latency_metric.sh`
- `checks/autoscaling/runtime/test_overload_hpa_trend.sh`

### `idempotency`
- `checks/rollout/test_install_twice.sh`
- `checks/rollout/test_uninstall_reinstall.sh`
- `checks/storage/test_store_bootstrap_idempotent.sh`

### `ingress`
- `checks/network/test_ingress_profile.sh`

### `install`
- `checks/rollout/test_install.sh`
- `checks/rollout/test_install_twice.sh`
- `checks/rollout/test_kind_image_resolution.sh`
- `checks/rollout/test_uninstall_reinstall.sh`

### `jobs`
- `checks/datasets/test_catalog_publish_job.sh`
- `checks/datasets/test_warmup_job.sh`

### `liveness`
- `checks/perf/degradation/test_liveness_under_load.sh`

### `load`
- `checks/autoscaling/runtime/test_overload_hpa_trend.sh`
- `checks/perf/degradation/test_liveness_under_load.sh`
- `checks/perf/degradation/test_overload_graceful_degradation.sh`
- `checks/perf/degradation/test_pod_churn_under_load.sh`
- `checks/perf/degradation/test_store_outage_under_load.sh`
- `checks/perf/pressure/test_noisy_neighbor_cpu_throttle.sh`

### `networkpolicy`
- `checks/network/test_networkpolicy.sh`
- `checks/network/test_networkpolicy_metadata_egress.sh`

### `observability`
- `checks/obs/runtime/test_logs_json.sh`
- `checks/obs/runtime/test_observability_objects_contract.sh`
- `checks/obs/runtime/test_prometheus_rule.sh`
- `checks/obs/runtime/test_redis_backend_metric.sh`
- `checks/obs/runtime/test_service_monitor.sh`

### `pdb`
- `checks/datasets/test_pdb.sh`
- `checks/datasets/test_pdb_required_when_replicas_gt1.sh`

### `profiles`
- `checks/datasets/test_multi_registry_profile.sh`
- `checks/datasets/test_offline_profile.sh`
- `checks/network/test_ingress_profile.sh`
- `checks/obs/contracts/test_values_profiles_are_valid.sh`
- `checks/perf/degradation/test_redis_optional.sh`
- `checks/storage/test_node_local_cache_profile.sh`

### `rate-limit`
- `checks/perf/degradation/test_admission_control_api_behavior.sh`
- `checks/perf/degradation/test_overload_graceful_degradation.sh`
- `checks/perf/degradation/test_redis_rate_limit.sh`

### `readiness`
- `checks/datasets/test_cached_only_mode.sh`
- `checks/datasets/test_readiness_catalog_refresh.sh`
- `checks/datasets/test_readiness_semantics.sh`

### `redis`
- `checks/obs/runtime/test_redis_backend_metric.sh`
- `checks/perf/degradation/test_redis_optional.sh`
- `checks/perf/degradation/test_redis_rate_limit.sh`

### `registry`
- `checks/datasets/test_multi_registry_profile.sh`

### `resilience`
- `checks/autoscaling/runtime/test_hpa_under_inflight_metric.sh`
- `checks/autoscaling/runtime/test_hpa_under_latency_metric.sh`
- `checks/autoscaling/runtime/test_overload_hpa_trend.sh`
- `checks/datasets/test_cached_only_mode.sh`
- `checks/datasets/test_dataset_missing_behavior.sh`
- `checks/datasets/test_offline_profile.sh`
- `checks/perf/degradation/test_admission_control_api_behavior.sh`
- `checks/perf/degradation/test_overload_graceful_degradation.sh`
- `checks/perf/degradation/test_pod_churn_under_load.sh`
- `checks/perf/degradation/test_store_outage_under_load.sh`
- `checks/perf/pressure/test_disk_pressure.sh`
- `checks/perf/pressure/test_memory_pressure.sh`
- `checks/perf/pressure/test_noisy_neighbor_cpu_throttle.sh`
- `checks/rollout/test_rolling_restart_no_downtime.sh`

### `resources`
- `checks/perf/pressure/test_disk_pressure.sh`
- `checks/perf/pressure/test_memory_pressure.sh`
- `checks/perf/pressure/test_noisy_neighbor_cpu_throttle.sh`
- `checks/perf/pressure/test_resource_limits.sh`

### `rollback`
- `checks/rollout/test_rollback.sh`

### `rolling-restart`
- `checks/rollout/test_rolling_restart_no_downtime.sh`

### `rollout`
- `checks/rollout/test_rollout.sh`

### `safety`
- `checks/security/test_defaults_safe.sh`

### `sanity`
- `checks/rollout/test_cluster_sanity.sh`
- `checks/storage/test_store_reachable.sh`

### `schema`
- `checks/obs/contracts/test_values_schema_strict.sh`

### `security`
- `checks/config/test_configmap_unknown_keys_rejected.sh`
- `checks/network/test_networkpolicy.sh`
- `checks/network/test_networkpolicy_metadata_egress.sh`
- `checks/security/test_rbac_minimalism.sh`
- `checks/security/test_secrets.sh`
- `checks/security/test_secrets_rotation.sh`
- `checks/storage/test_store_bucket_policy.sh`

### `storage`
- `checks/storage/test_node_local_cache_profile.sh`
- `checks/storage/test_storage_modes.sh`

### `supply-chain`
- `checks/obs/contracts/test_helm_repo_pinning.sh`
- `checks/security/test_image_digest_pinning.sh`
- `checks/security/test_image_digest_policy.sh`
- `checks/security/test_rbac_minimalism.sh`
