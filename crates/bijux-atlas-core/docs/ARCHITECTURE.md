# bijux-atlas-core Architecture

## Purpose

`bijux-atlas-core` is the deterministic base crate for cross-crate contracts and pure transforms.

## Pure domain surface

Pure logic lives under `src/domain` and `src/types`.

- `domain::canonical`: stable hash and canonical JSON normalization.
- `domain::config`: deterministic config/cache path resolution from environment.
- `types`: invariant value objects (`DatasetId`, `ShardId`, `RunId`).

Pure domain code must not perform filesystem, network, process, or wall-clock operations.

## Port surface

Effect contracts live in `src/ports`.

- `FsPort`
- `ClockPort`
- `NetPort`
- `ProcessPort`

Ports define capability boundaries only. Runtime adapters belong in other crates.

## Stable public API

Stable exports are listed in `docs/public-api.md`.

Guidelines:
- Add public items only with matching docs and tests.
- Keep `Error` and `Result<T>` as the canonical fallible surface.
- Use invariant newtypes instead of unconstrained primitive strings.
