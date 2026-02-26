# Kubernetes Packaging and Operational Contract

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

This document defines production deployment defaults for `bijux-atlas`.

## Container Image

- Dockerfile uses a reproducible multi-stage build.
- Build stage runs `cargo build --locked --release -p bijux-atlas-server`.
- Runtime stage is distroless (`gcr.io/distroless/cc-debian12:nonroot`).
- Runtime user is non-root and immutable by default.
- Runtime command follows plugin-mode contract (`/app/bijux-atlas atlas serve`), see `docs/k8s/plugin-mode-entrypoint.md`.

## Helm Chart Surface

Chart path: `ops/k8s/charts/bijux-atlas/`

Included resources:

- `Deployment`
- `Service`
- `HorizontalPodAutoscaler` (CPU + custom metrics)
- `PodDisruptionBudget`
- `ServiceMonitor`
- `NetworkPolicy`
- `ConfigMap` and optional `Secret`
- Optional `Rollout` (Argo)
- Optional `Job` template for catalog publish

## Security Defaults

Pod and container defaults:

- `runAsNonRoot: true`
- `readOnlyRootFilesystem: true`
- `allowPrivilegeEscalation: false`
- Linux capabilities dropped (`ALL`)

Filesystem strategy:

- Read-only root filesystem.
- Explicit writable mounts only for `/cache` and `/tmp` via `emptyDir`.
- `emptyDir` limits are explicit for both cache and tmp volumes.
- Pod requests/limits include `ephemeral-storage` to align kubelet eviction behavior with atlas cache policy.

## Network Policy

Default behavior is deny egress except explicit rules:

- Optional DNS egress to kube-system.
- Explicit egress CIDR allowlist for catalog/store endpoints.

No broad egress permit is included in default values.

## Requests/Limits and Tuning

Default chart values:

- Request: `500m` CPU, `512Mi` memory.
- Limit: `2000m` CPU, `2Gi` memory.

Tuning guidance:

- Increase memory before CPU for cache-heavy workloads.
- Keep request CPU high enough to avoid throttling at p95.
- Align `cache.diskSize` and `cache.maxDiskBytes` with node ephemeral storage quotas.
- Tune concurrency caps (`cheap`, `medium`, `heavy`) together with HPA targets.

## Probe Semantics

- `startupProbe`: `/readyz` with long failure budget for cold starts and cache warmup.
- `readinessProbe`: `/readyz`; pod must be removed from service endpoints if catalog is unavailable or server loop is unhealthy.
- `livenessProbe`: `/healthz`; used only to restart wedged processes.

Readiness contract for production:

- Ready only when process loop is healthy.
- Ready only when catalog is reachable, unless `cached-only` mode is explicitly enabled.

## Node-Local SSD Profile

Recommended for high-throughput clusters:

- Mount `/cache` on node-local SSD (`local PV` or host-backed fast storage).
- Keep `/tmp` on bounded `emptyDir`.
- Use startup warmup jitter to avoid synchronized store fetches.

Measurement workflow:

1. Deploy baseline on generic storage.
2. Run `ops/load/k6/suites/warm-steady.js` and `ops/load/k6/suites/regional-spike-10x-60s.js`.
3. Deploy node-local SSD profile and rerun same suites.
4. Compare p95/p99 and store download latency from `/metrics`.

## Init Prewarm

Optional init container prewarm is exposed in values:

- `cache.initPrewarm.enabled`
- `cache.pinnedDatasets`

Purpose:

- Pull pinned/hot datasets before the main container starts.
- Reduce first-request tail latency.

Optional warm-up job template:

- `datasetWarmupJob.enabled`
- Runs `atlas smoke` for configured pinned datasets.
- Useful for controlled cache priming before traffic cutover.

## Config and Secret Wiring

- `ConfigMap` stores non-secret runtime config.
- `Secret` stores store credentials (`ATLAS_STORE_ACCESS_KEY`, `ATLAS_STORE_SECRET_KEY`).

Sequence-specific knobs in chart values/config map:

- `ATLAS_MAX_SEQUENCE_BASES`
- `ATLAS_SEQUENCE_API_KEY_REQUIRED_BASES`
- `ATLAS_SEQUENCE_TTL_MS`
- `ATLAS_SEQUENCE_RATE_LIMIT_CAPACITY`
- `ATLAS_SEQUENCE_RATE_LIMIT_REFILL_PER_SEC`

## HPA Strategy

HPA includes:

- CPU utilization target.
- p95 latency custom metric (`bijux_http_request_latency_p95_seconds`).
- in-flight heavy query custom metric (`bijux_inflight_heavy_queries`).

This requires a custom metrics adapter wired from Prometheus.

## Canary Rollout

The chart supports optional Argo Rollouts resource behind `rollout.enabled`.

- Progressive steps are configured in `values.yaml`.
- Use only when Argo Rollouts CRDs and controller are installed.

## Catalog Publish Job Template

`catalog-publish-job.yaml` is a template for ingestion/publish operations.

- Intended for controlled publish pipelines.
- Not intended for per-request execution.

## Offline Deployment Profile

`values-offline.yaml` enables cached-only serving mode.

Characteristics:

- No external store access required at runtime.
- Tightened network policy.
- Optional warmup for pinned datasets.

For advanced node-local acceleration profile, see `docs/k8s/node-local-shared-cache-profile.md`.

## Compatibility Note

See `docs/contracts/compatibility.md` for producer/consumer artifact contract alignment.
## Referenced chart values keys

- `values.server`
- `values.store`
- `values.networkPolicy`
- `values.resources`
- `values.hpa`
- `values.pdb`
- `values.serviceMonitor`

## See also

- `ops-ci`
