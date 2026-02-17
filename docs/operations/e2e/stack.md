# E2E Stack

- Owner: `bijux-atlas-operations`

## What

Canonical local stack definition for Kind + MinIO (+ optional Redis/OTEL).

## Why

Provides deterministic environment setup for reproducible e2e runs.

## Scope

`ops/e2e/stack/*` manifests and `ops/e2e/scripts/up.sh` / `down.sh` behavior.

## Non-goals

Does not replace per-component manifests.

## Contracts

- Cluster definition: `ops/e2e/stack/kind/cluster.yaml`
- Store bootstrap: `ops/e2e/stack/minio/bootstrap.sh`
- Bring up: `ops/e2e/scripts/up.sh`
- Tear down: `ops/e2e/scripts/down.sh`

## Failure modes

Version drift in local dependencies causes invalid or flaky e2e runs.

## How to verify

```bash
$ ./ops/e2e/scripts/up.sh
$ ./ops/e2e/scripts/down.sh
```

Expected output: cluster starts then fully tears down.

## See also

- [E2E Index](INDEX.md)
- [K8s Tests](k8s-tests.md)
- [Scripts](scripts.md)
