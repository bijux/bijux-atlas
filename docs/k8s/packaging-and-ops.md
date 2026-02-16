# Kubernetes Packaging and Operational Contract

This document defines production deployment defaults for `bijux-atlas`.

## Container Image

- Dockerfile uses a reproducible multi-stage build.
- Build stage runs `cargo build --locked --release -p bijux-atlas-server`.
- Runtime stage is distroless (`gcr.io/distroless/cc-debian12:nonroot`).
- Runtime user is non-root and immutable by default.

## Helm Chart Surface

Chart path: `charts/bijux-atlas/`

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
- Ready only when catalog refresh policy allows serving current requests.

## Init Prewarm

Optional init container prewarm is exposed in values:

- `cache.initPrewarm.enabled`
- `cache.pinnedDatasets`

Purpose:

- Pull pinned/hot datasets before the main container starts.
- Reduce first-request tail latency.

## Config and Secret Wiring

- `ConfigMap` stores non-secret runtime config.
- `Secret` stores store credentials (`ATLAS_STORE_ACCESS_KEY`, `ATLAS_STORE_SECRET_KEY`).

## HPA Strategy

HPA includes:

- CPU utilization target.
- Request-rate custom metric.
- p95 latency custom metric.

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

## Compatibility Note

See `docs/compatibility/bijux-dna-atlas.md` for producer/consumer artifact contract alignment.
