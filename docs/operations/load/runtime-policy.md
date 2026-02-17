# Compose Vs Kind Policy

- Owner: `bijux-atlas-operations`

## What

Defines canonical runtime for load validation.

## Why

Prevents split truth between local compose and k8s-like execution.

## Contracts

- Canonical reference runtime: kind/k8s via `make ops-k8s-tests` + load suites.
- Optional runtime: docker-compose under `ops/load/compose/` for local quick loops.
- Scenario/query SSOT is shared and independent of runtime.

## How to verify

```bash
$ make ops-load-smoke
$ make ops-load-full
```

Expected output: same scenario names and result contracts across runtimes.

## See also

- [Load Suites](suites.md)
- [Compose Perf Profiles](../../../ops/load/compose/README.md)
- `ops-load-full`
