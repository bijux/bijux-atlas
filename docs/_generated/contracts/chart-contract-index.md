# Chart Contract Index

Generated from `ops/k8s/tests/manifest.json` entries tagged with `chart-contract`.

| Contract Test | Owner | Timeout (s) | Failure Modes |
| --- | --- | ---: | --- |
| `checks/autoscaling/contracts/test_hpa_behavior_contract.sh` | `chart` | 120 | `hpa_behavior_contract_drift` |
| `checks/autoscaling/contracts/test_hpa_enabled_requires_metrics_stack.sh` | `chart` | 120 | `hpa_enabled_without_metrics_stack` |
| `checks/autoscaling/contracts/test_hpa_enabled_requires_resources.sh` | `chart` | 120 | `hpa_enabled_without_resources` |
| `checks/autoscaling/contracts/test_hpa_max_replicas_cap.sh` | `chart` | 120 | `hpa_max_replicas_cap_exceeded` |
| `checks/autoscaling/contracts/test_hpa_metrics_names_contract.sh` | `chart` | 120 | `hpa_metric_names_contract_drift` |
| `checks/config/test_configmap.sh` | `server` | 240 | `invalid_config_accepted` |
| `checks/config/test_configmap_keys_match_runtime_expected.sh` | `server` | 120 | `configmap_keys_not_in_runtime_contract` |
| `checks/config/test_configmap_must_exist.sh` | `server` | 180 | `configmap_missing_or_missing_required_keys` |
| `checks/config/test_configmap_schema_completeness.sh` | `server` | 120 | `config_schema_docs_drift` |
| `checks/config/test_configmap_unknown_keys_rejected.sh` | `server` | 180 | `unknown_configmap_keys_allowed` |
| `checks/config/test_configmap_update_reload.sh` | `server` | 300 | `config_reload_contract_broken` |
| `checks/config/test_configmap_version_stamp.sh` | `server` | 180 | `configmap_version_stamp_missing` |
| `checks/config/test_deployment_envFrom_configmap.sh` | `chart` | 120 | `deployment_envfrom_configmap_drift` |
| `checks/datasets/test_pdb_required_when_replicas_gt1.sh` | `chart` | 120 | `pdb_not_required_for_replicas_gt1` |
| `checks/obs/contracts/test_no_checksum_rollout.sh` | `chart` | 120 | `checksum_rollout_policy_violation` |
| `checks/obs/contracts/test_values_minimums.sh` | `chart` | 120 | `unsafe_values_minimums` |
| `checks/obs/contracts/test_values_profiles_are_valid.sh` | `chart` | 240 | `invalid_values_profile` |
| `checks/obs/contracts/test_values_schema_strict.sh` | `chart` | 120 | `values_schema_not_strict` |
| `checks/obs/runtime/test_observability_objects_contract.sh` | `observability` | 120 | `observability_objects_contract_drift` |
| `checks/security/test_image_digest_policy.sh` | `chart` | 120 | `image_digest_policy_violation` |

## Regenerate

```bash
atlasctl docs generate-chart-contract-index
```
