# K8s Test Contract

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines the exact Kubernetes behaviors verified by suite IDs in `ops/k8s/tests/suites.json`.

## Why

Locks deployment/runtime expectations to executable checks.

## Contracts

- Harness: `crates/bijux-dev-atlas/src/commands/ops/k8s/tests/harness.py` emits JSON + JUnit and enforces per-test `timeout_seconds`.
- Metadata SSOT: `ops/k8s/tests/manifest.json` defines groups/retries/owners.
- Suite SSOT: `ops/k8s/tests/suites.json` defines public suite IDs.
- Public suite IDs:
  - `smoke`
  - `resilience`
  - `full`

## Failure modes

Any failing test blocks k8s contract acceptance for atlas deployments.
Flake policy:
- Retry-pass tests are recorded in `artifacts/ops/k8s/flake-report.json`.
- CI treats flakes as failures until quarantine TTL is explicitly set in `ops/k8s/tests/manifest.json`.
Failure artifacts:
- On any failure, bundle is captured under `artifacts/ops/k8s-failures/` and tarred as `artifacts/ops/k8s-failure-bundle-<timestamp>.tar.gz`.
- Bundle includes events, logs, `helm get manifest`, and `kubectl top pods` when metrics-server is available.

## How to verify

```bash
$ make ops-k8s-tests
$ make ops-k8s-suite SUITE=smoke PROFILE=kind
$ make ops-k8s-suite SUITE=resilience PROFILE=kind
```

Expected output: all contract tests pass; on failure a report appears in `artifacts/ops/k8s-failures/`.

## See also

- [K8s Index](INDEX.md)
- [Helm Chart Contract](chart.md)
- [E2E Kubernetes Tests](../e2e/k8s-tests.md)
- `ops-k8s-tests`

- Chart values anchor: `values.server`
