# Compose Perf Profiles

- Owner: `bijux-atlas-operations`

## Scope

This compose set is **load-harness specific** and intentionally different from `ops/observe/pack/compose`.

- `ops/load/compose/*`: load execution substrate for perf scenarios and runner ergonomics.
- `ops/observe/pack/compose/*`: observability pack runtime (prom/grafana/otel) and pack conformance.

They are not duplicates and are owned by different concepts.

## Commands

- `make ops-load-smoke`
- `make ops-load-full`

Canonical docs: `ops/README.md`, `docs/operations/INDEX.md`.
