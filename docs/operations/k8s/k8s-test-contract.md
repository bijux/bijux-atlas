# K8s Test Contract

- Owner: `bijux-atlas-operations`

## What

Defines the exact Kubernetes behaviors verified by `make ops-k8s-tests`.

## Why

Locks deployment/runtime expectations to executable checks.

## Contracts

- Template gates: `test_helm_templates.sh`
- Install/reachability/readiness: `test_install.sh`
- NetworkPolicy allow/deny egress: `test_networkpolicy.sh`
- Secret missing/wrong failure modes: `test_secrets.sh`
- ConfigMap invalid config fail-fast: `test_configmap.sh`
- Cached-only + readiness outage semantics: `test_cached_only_mode.sh`, `test_readiness_semantics.sh`
- PDB eviction safety: `test_pdb.sh`
- HPA scaling reaction: `test_hpa.sh`
- Rollout/rollback semantics: `test_rollout.sh`, `test_rollback.sh`
- Warmup/catalog jobs: `test_warmup_job.sh`, `test_catalog_publish_job.sh`
- Resource/liveness behavior: `test_resource_limits.sh`, `test_liveness_under_load.sh`
- ServiceMonitor/metrics/log JSON: `test_service_monitor.sh`, `test_logs_json.sh`
- Profile rendering: `test_node_local_cache_profile.sh`, `test_multi_registry_profile.sh`, `test_offline_profile.sh`

## Failure modes

Any failing test blocks k8s contract acceptance for atlas deployments.

## How to verify

```bash
$ make ops-k8s-tests
$ ops/k8s/tests/report.sh
```

Expected output: all contract tests pass; on failure a report appears in `artifacts/ops/k8s-failures/`.

## See also

- [K8s Index](INDEX.md)
- [Helm Chart Contract](chart.md)
- [E2E Kubernetes Tests](../ops/e2e/k8s-tests.md)
- `ops-k8s-tests`
