# K8s Test Contract

- Owner: `bijux-atlas-operations`

## What

Defines the exact Kubernetes behaviors verified by `make ops-k8s-tests`.

## Why

Locks deployment/runtime expectations to executable checks.

## Contracts

- Harness: `ops/e2e/k8s/tests/harness.py` emits JSON + JUnit and enforces per-test `timeout_seconds`.
- Metadata SSOT: `ops/e2e/k8s/tests/manifest.json` defines groups, retries, owner, expected failure modes, timeout budget.
- Ownership SSOT: `ops/e2e/k8s/tests/ownership.json`.
- Install/idempotency: `test_install.sh`, `test_install_twice.sh`, `test_uninstall_reinstall.sh`.
- Readiness/liveness: `test_readiness_semantics.sh`, `test_readiness_catalog_refresh.sh`, `test_liveness_under_load.sh`.
- Rollout/no-downtime: `test_rolling_restart_no_downtime.sh`.
- Security/config: `test_secrets.sh`, `test_secrets_rotation.sh`, `test_configmap.sh`, `test_configmap_update_reload.sh`.
- Network policy: `test_networkpolicy.sh`, `test_networkpolicy_metadata_egress.sh`.
- Scaling/availability: `test_hpa.sh` (scale up + down), `test_pdb.sh`.
- Profiles: `test_offline_profile.sh`, `test_multi_registry_profile.sh`, `test_ingress_profile.sh`.
- Observability CRD-aware: `test_service_monitor.sh`, `test_prometheus_rule.sh`.
- Resource/storage: `test_resource_limits.sh`, `test_storage_modes.sh`.

## Failure modes

Any failing test blocks k8s contract acceptance for atlas deployments.
Flake policy:
- Retry-pass tests are recorded in `artifacts/ops/k8s/flake-report.json`.
- CI treats flakes as failures until quarantine TTL is explicitly set in `ops/e2e/k8s/tests/manifest.json`.
Failure artifacts:
- On any failure, bundle is captured under `artifacts/ops/k8s-failures/` and tarred as `artifacts/ops/k8s-failure-bundle-<timestamp>.tar.gz`.
- Bundle includes events, logs, `helm get manifest`, and `kubectl top pods` when metrics-server is available.

## How to verify

```bash
$ make ops-k8s-tests
$ make ops-k8s-tests ATLAS_E2E_TEST_GROUP=networkpolicy
$ make ops-k8s-tests ATLAS_E2E_TEST=test_install.sh
```

Expected output: all contract tests pass; on failure a report appears in `artifacts/ops/k8s-failures/`.

## See also

- [K8s Index](INDEX.md)
- [Helm Chart Contract](chart.md)
- [E2E Kubernetes Tests](../e2e/k8s-tests.md)
- `ops-k8s-tests`
