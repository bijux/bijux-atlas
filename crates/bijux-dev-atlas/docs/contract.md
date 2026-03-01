# Dev Control Plane Contract

## Scope

`bijux-dev-atlas` is the development control-plane for repository governance, checks, docs, configs, and ops automation contracts.

## Canonical Invocation

- Local development: `cargo run -p bijux-dev-atlas -- <args>`
- Installed umbrella dispatch: `bijux dev atlas <args>`
- Runtime product CLI namespace: `bijux atlas <args>`
- Naming contract is frozen: runtime commands use `bijux atlas ...`; repository control-plane commands use `bijux dev atlas ...`.

## Invariants

- Checks are pure by default.
- Effects require explicit capability flags (`--allow-write`, `--allow-subprocess`, `--allow-network`, `--allow-git`).
- Machine outputs are deterministic unless a command explicitly documents nondeterministic fields.
- Artifact writes are restricted to the control-plane artifact hierarchy under `artifacts/<run-kind>/<run-id>/...` (current dev-atlas concrete layout is `artifacts/atlas-dev/<domain>/<run-id>/...`).
- Command selection state is explicit in machine-readable outputs (suite/domain/tag/id filters and include flags).

## Output Formats

- `text`
- `json`
- `jsonl`

All JSON outputs must include a stable `schema_version` for their payload family.

Commands that do not support a format must reject unsupported values explicitly.

## Exit Behavior

- `0`: success / no failing checks
- non-zero: policy failure, usage error, contract error, or execution error (see exit-codes.md`)

## Artifact Rules

- Reports and generated outputs are written under `artifacts/<run-kind>/<run-id>/...`
- `bijux dev atlas` reserves `run-kind = atlas-dev/<domain>` and writes under `artifacts/atlas-dev/<domain>/<run-id>/...`
- Run IDs are stable when supplied (`--run-id`) and validated by the model crate
- Evidence paths must be timestamp-free when committed or emitted as deterministic references

## Visibility

- Public checks are listed by default
- Internal checks require `--include-internal`
- Slow checks require `--include-slow`
