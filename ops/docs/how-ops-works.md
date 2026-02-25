# How Ops Works

- Owner: bijux-atlas-operations
- Stability: stable

## Model

- `ops/` is declarative specification and evidence only.
- `bijux dev atlas ops ...` is the execution surface.
- `make ops-*` targets are thin wrappers around `bijux dev atlas`.

## Control Flow

1. Edit canonical contracts in `ops/inventory/`, `ops/schema/`, and domain directories.
2. Validate with `make ops` or `bijux dev atlas ops doctor`.
3. Generate deterministic evidence artifacts into `artifacts/...` at runtime.
4. Curate representative examples under `ops/_generated.example/`.

## Guardrails

- No behavior code under `ops/`.
- No executable bits under `ops/`.
- Canonical directories and required marker files are enforced by ops checks.
- Pins and toolchain inventories are validated through schema and contract checks.

## Release

- Release evidence is assembled from `ops/report/generated/*.json`.
- Pin freeze state is declared in `ops/inventory/pin-freeze.json`.
- Final readiness is represented by report readiness score and bundle status.
