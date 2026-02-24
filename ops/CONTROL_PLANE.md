# Control Plane Definition

Control plane version: `1`

## Scope

`bijux dev atlas` is the repository control plane for developer governance, docs validation, configs validation, and ops stack orchestration.

Covered command groups:
- `check`
- `docs`
- `configs`
- `ops`

## SSOT Rules

- `ops/` holds operational SSOT manifests, inventory, contracts, and schemas.
- `configs/` holds configuration SSOT inputs and contracts.
- `docs/` and `mkdocs.yml` hold documentation SSOT structure.
- Makefiles and workflows are wrappers and entrypoints only; they must delegate to `bijux dev atlas ...`.

## Invariants

- Command surfaces are documented in command-list snapshot docs.
- Reports and artifacts use stable `schema_version` values.
- Artifacts are written only under `artifacts/atlas-dev/...`.
- Artifacts must not be tracked by git.
- Internal checks stay hidden unless `--include-internal` is provided.

## Effect Rules

- Writes require explicit write flags.
- Subprocess execution requires explicit subprocess flags.
- Network access is denied by default and requires explicit network flags.

## CI And Local Entrypoints

- CI entrypoints: `bijux dev atlas doctor` and `bijux dev atlas check run --suite ci`
- Local entrypoints: `bijux dev atlas doctor` and `bijux dev atlas check run --suite local`
