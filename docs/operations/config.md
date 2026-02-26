# Runtime Config Contract

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines runtime ConfigMap behavior and the operator workflow for safe config changes.

## Why

Prevents accidental assumptions about hot reload and makes config rollout behavior explicit.

## Scope

Applies to `ops/k8s/charts/bijux-atlas/templates/configmap.yaml` and production config-change runbooks.

## Non-goals

Does not provide in-process hot reload of ConfigMap values.

## Contracts

- Config changes require rollout restart by design; pods do not auto-reload patched ConfigMap values.
- Canonical workflow is values update -> `helm upgrade` -> rollout restart.
- Unknown ConfigMap keys are rejected by `bijux dev atlas ops k8s validate-configmap-keys` when `ATLAS_STRICT_CONFIG_KEYS=1`.
- Version stamps are required: `ATLAS_CONFIG_RELEASE_ID` and `ATLAS_CONFIG_SCHEMA_VERSION`.
- Runtime startup config contract artifacts are generated from server source:
  - `crates/bijux-atlas-server/docs/generated/runtime-startup-config.schema.json`
  - `crates/bijux-atlas-server/docs/generated/runtime-startup-config.md`

### Config Keys

- `ATLAS_CONFIG_RELEASE_ID`: release/revision stamp for debugging live config provenance.
- `ATLAS_CONFIG_SCHEMA_VERSION`: config schema stamp from `values.server.configSchemaVersion`.
- `ATLAS_CATALOG_ENDPOINT`: catalog base URL from `values.catalog.endpoint`.
- `ATLAS_STORE_ENDPOINT`: object store endpoint from `values.store.endpoint`.
- `ATLAS_STORE_S3_ENABLED`: toggles store S3 mode when store endpoint is configured.
- `ATLAS_STORE_S3_BASE_URL`: store base URL used by runtime store client.
- `ATLAS_STORE_S3_PRESIGNED_BASE_URL`: optional presigned URL base for store fetches.
- `ATLAS_MAX_DATASET_COUNT`: maximum dataset entries retained in cache index.
- `ATLAS_MAX_DISK_BYTES`: hard disk budget for cache artifacts.
- `ATLAS_PINNED_DATASETS`: comma-separated pinned dataset IDs.
- `ATLAS_MAX_CONCURRENT_DOWNLOADS`: concurrent dataset fetch cap.
- `ATLAS_STARTUP_WARMUP_JITTER_MAX_MS`: startup warmup jitter budget.
- `ATLAS_RATE_LIMIT_IP_CAPACITY`: per-IP token bucket capacity.
- `ATLAS_RATE_LIMIT_IP_REFILL_PER_SEC`: per-IP token bucket refill rate.
- `ATLAS_RATE_LIMIT_API_KEY_ENABLED`: enables per-api-key rate limit policy.
- `ATLAS_RATE_LIMIT_API_KEY_CAPACITY`: per-api-key token bucket capacity.
- `ATLAS_RATE_LIMIT_API_KEY_REFILL_PER_SEC`: per-api-key token bucket refill rate.
- `ATLAS_CONCURRENCY_CHEAP`: concurrency budget for cheap class queries.
- `ATLAS_CONCURRENCY_MEDIUM`: concurrency budget for medium class queries.
- `ATLAS_CONCURRENCY_HEAVY`: concurrency budget for heavy class queries.
- `ATLAS_REQUEST_TIMEOUT_MS`: request timeout budget in milliseconds.
- `ATLAS_SQL_TIMEOUT_MS`: SQL timeout budget in milliseconds.
- `ATLAS_RESPONSE_MAX_BYTES`: maximum serialized response bytes.
- `ATLAS_MAX_BODY_BYTES`: maximum accepted request body bytes.
- `ATLAS_SLOW_QUERY_THRESHOLD_MS`: threshold for slow query telemetry tagging.
- `ATLAS_ENABLE_DEBUG_DATASETS`: enables debug dataset endpoints.
- `ATLAS_ENABLE_EXEMPLARS`: enables metrics exemplars and trace links.
- `ATLAS_CACHED_ONLY_MODE`: enforces cached-only serving mode.
- `ATLAS_READ_ONLY_FS_MODE`: enables runtime read-only filesystem assumptions.
- `ATLAS_SHUTDOWN_DRAIN_MS`: shutdown drain window for in-flight requests.
- `ATLAS_READINESS_REQUIRES_CATALOG`: readiness gate requiring catalog availability.
- `ATLAS_MAX_SEQUENCE_BASES`: hard cap for sequence base requests.
- `ATLAS_SEQUENCE_API_KEY_REQUIRED_BASES`: threshold requiring API key for sequence calls.
- `ATLAS_SEQUENCE_TTL_MS`: sequence cache TTL in milliseconds.
- `ATLAS_SEQUENCE_RATE_LIMIT_CAPACITY`: sequence endpoint token bucket capacity.
- `ATLAS_SEQUENCE_RATE_LIMIT_REFILL_PER_SEC`: sequence endpoint token bucket refill rate.
- `ATLAS_CATALOG_BACKOFF_BASE_MS`: catalog refresh retry backoff base.
- `ATLAS_CATALOG_BREAKER_FAILURE_THRESHOLD`: catalog circuit-breaker failure threshold.
- `ATLAS_CATALOG_BREAKER_OPEN_MS`: catalog circuit-breaker open duration.

## Failure modes

- Patching ConfigMap without rollout restart leaves behavior unchanged.
- Unknown or dangling keys create configuration drift and unclear operational state.

## How to verify

```bash
make k8s/apply-config
make k8s/restart
ATLAS_E2E_TEST=test_configmap_update_reload.sh make ops-k8s-tests
ATLAS_E2E_TEST=test_configmap_version_stamp.sh make ops-k8s-tests
```

Expected output: config changes require explicit restart, stamps exist, and config tests pass.

## See also

- [Kubernetes Operations Index](k8s/INDEX.md)
- [Rollback Playbook](runbooks/rollback-playbook.md)
- [Kubernetes Values](k8s/values.md)
