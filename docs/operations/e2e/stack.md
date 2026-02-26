# E2E Stack

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

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
- Stack versions are pinned in `ops/inventory/toolchain.json`.
- Service surfaces/ports (inventory-backed): atlas `8080`, prometheus `9090`, grafana `3000`, minio `9000/9001`, otel `4317/4318`, redis `6379`.
- Stack dependencies are documented in `ops/stack/dependencies.md`.
- Canonical values profiles: `ops/k8s/values/local.yaml`, `ops/k8s/values/offline.yaml`, `ops/k8s/values/perf.yaml`
- Bring up: `ops-up`
- Stack-only bring-up: `ops-stack-up`
- Tear down: `ops-down`
- Stack-only tear-down: `ops-stack-down`
- Stack-only smoke: `ops-stack-smoke`
- Stack validation: `ops-stack-validate`
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
- [Ops Command Inventory](../../development/tooling/ops-command-inventory.md)
