# E2E Stack

- Owner: `bijux-atlas-operations`

## What

Canonical local stack definition for Kind + MinIO (+ optional Redis/OTEL).

## Why

Provides deterministic environment setup for reproducible e2e runs.

## Scope

`e2e/stack/*` manifests and `e2e/scripts/up.sh` / `down.sh` behavior.

## Non-goals

Does not replace per-component manifests.

## Contracts

- Cluster definition: `e2e/stack/kind/cluster.yaml`
- Store bootstrap: `e2e/stack/minio/bootstrap.sh`
- Bring up: `e2e/scripts/up.sh`
- Tear down: `e2e/scripts/down.sh`

## Failure modes

Version drift in local dependencies causes invalid or flaky e2e runs.

## How to verify

```bash
$ ./e2e/scripts/up.sh
$ ./e2e/scripts/down.sh
```

Expected output: cluster starts then fully tears down.

## See also

- [E2E Index](INDEX.md)
- [K8s Tests](k8s-tests.md)
- [Scripts](scripts.md)
