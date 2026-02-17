# E2E Stack

- Owner: `bijux-atlas-operations`

## What

Canonical local stack definition for Kind + MinIO (+ optional Redis/OTEL).

## Why

Provides deterministic environment setup for reproducible e2e runs.

## Scope

`ops/stack/*` manifests and `ops-*` make target behavior.

## Non-goals

Does not replace per-component manifests.

## Contracts

- Cluster definition: `ops/stack/kind/cluster.yaml`
- Store bootstrap: `ops/stack/minio/bootstrap.sh`
- Stack components: `ops/stack/{prometheus,otel,redis,toxiproxy}`
- Canonical values profiles: `ops/k8s/values/local.yaml`, `ops/k8s/values/offline.yaml`, `ops/k8s/values/perf.yaml`
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
$ make ops-k8s-template-tests
$ make ops-down
```

Expected output: cluster starts, chart template checks pass, then stack tears down.

## See also

- [E2E Index](INDEX.md)
- [K8s Tests](k8s-tests.md)
- [Scripts](scripts.md)
