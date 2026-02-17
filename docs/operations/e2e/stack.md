# E2E Stack

- Owner: `bijux-atlas-operations`

## What

Canonical local stack definition for Kind + MinIO (+ optional Redis/OTEL).

## Why

Provides deterministic environment setup for reproducible e2e runs.

## Scope

`ops/e2e/stack/*` manifests and `ops-*` make target behavior.

## Non-goals

Does not replace per-component manifests.

## Contracts

- Cluster definition: `ops/e2e/stack/kind/cluster.yaml`
- Store bootstrap: `ops/e2e/stack/minio/bootstrap.sh`
- Bring up: `ops-up`
- Tear down: `ops-down`
- Reset/wipe state: `ops-reset`
- Deploy/warm/smoke: `ops-deploy`, `ops-warm`, `ops-smoke`
- Full nightly-style pipeline: `ops-ci`

## Failure modes

Version drift in local dependencies causes invalid or flaky e2e runs.

## How to verify

```bash
$ make ops-up
$ make ops-down
```

Expected output: cluster starts then fully tears down.

## See also

- [E2E Index](INDEX.md)
- [K8s Tests](k8s-tests.md)
- [Scripts](scripts.md)
