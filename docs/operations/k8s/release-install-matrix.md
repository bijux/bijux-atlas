# Release Install Matrix

- Owner: `bijux-atlas-operations`

## What

Generated matrix of k8s install/test profiles from CI summaries.

## Why

Provides a stable compatibility view across supported chart profiles.

## Contracts

Generated at: `2026-02-17T12:51:02Z`

Profiles:
- `local`
- `offline`
- `perf`
- `ingress`
- `multi-registry`

Verified test groups:
- `install`
- `networkpolicy`
- `hpa`
- `pdb`
- `rollout`
- `rollback`
- `secrets`
- `configmap`
- `serviceMonitor`

## Failure modes

Missing profile/test entries indicate CI generation drift or skipped suites.

## How to verify

```bash
$ make ops-release-matrix
$ make docs
```

Expected output: matrix doc updated and docs checks pass.

## See also

- [K8s Test Contract](k8s-test-contract.md)
- [Helm Chart Contract](chart.md)
- `ops-k8s-tests`

- Chart values anchor: `values.server`
