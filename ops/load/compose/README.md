# Compose Perf Profiles

- Owner: `bijux-atlas-operations`

## What

Optional docker-compose profiles for local load harness runs.

## Why

Provides a fallback runtime when kind/k8s is not used.

## Contracts

- Canonical load system uses `ops/load/scenarios/*.json` and `ops/load/scripts/run_suite.sh`.
- Canonical production-like runtime is kind/k8s.
- Compose runtime is optional local fallback only.
- Compose files:
  - `ops/load/compose/docker-compose.perf.yml`
  - `ops/load/compose/docker-compose.perf.redis.yml`

## How to verify

```bash
$ make ops-load-smoke
$ make ops-load-full
```

Expected output: suites run and emit result artifacts.

## See also

- [Load Suites](../../../docs/operations/load/suites.md)
- [Load CI Policy](../../../docs/operations/load/ci-policy.md)
- `ops-load-full`
