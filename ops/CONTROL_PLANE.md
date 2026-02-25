# Control Plane Definition

> Contract warning: this document must match the current command and repository reality; mismatches are contract drift and must be fixed immediately.

Control plane version: `1`

## Scope

`bijux dev atlas` is the repository control plane for developer governance, docs validation, configs validation, and ops stack orchestration.

Naming contract (frozen):
- Runtime product CLI: `bijux atlas ...`
- Repository control plane: `bijux dev atlas ...`

Covered command groups:
- `check`
- `docs`
- `configs`
- `ops`

Control plane truth is derived from `cargo metadata` and synchronized to `ops/_generated.example/control-plane.snapshot.md`.
This page must either match that snapshot or contain only stable contract statements with no hardcoded crate lists.

## SSOT Rules

- `ops/` holds operational SSOT manifests, inventory, contracts, and schemas.
- `configs/` holds configuration SSOT inputs and contracts.
- `docs/` and `mkdocs.yml` hold documentation SSOT structure.
- Makefiles and workflows are wrappers and entrypoints only; they must delegate to `bijux dev atlas ...`.
- `ops/` is data-only for repository-owned ops assets (yaml/json/toml/md/charts/k6 scripts); executable entrypoints must be provided by `bijux dev atlas`.
- `ops/` must not contain `Makefile` files; make wrappers belong under `makefiles/*.mk`.
- Helm charts may live under `ops/`, but rendering/apply flows are owned by `bijux dev atlas ops k8s ...` / `bijux dev atlas ops render ...`.
- k6 scripts may live under `ops/`, but execution is owned by `bijux dev atlas ops load ...`.
- Shell is not a repository entrypoint. If third-party shell assets remain, they must be treated as vendor/third_party assets and never invoked as in-tree control-plane entrypoints.

## Invariants

- Command surfaces are documented in command-list snapshot docs.
- Reports and artifacts use stable `schema_version` values.
- Control-plane artifacts follow `artifacts/<run-kind>/<run-id>/...`; current dev-atlas run kinds are rooted at `artifacts/atlas-dev/<domain>/<run-id>/...`.
- Artifacts must not be tracked by git.
- Internal checks stay hidden unless `--include-internal` is provided.

## Effect Rules

- Writes require explicit write flags.
- Subprocess execution requires explicit subprocess flags.
- Network access is denied by default and requires explicit network flags.
- No Python tooling is required for active control-plane operation.

## CI And Local Entrypoints

- CI entrypoints: `bijux dev atlas doctor` and `bijux dev atlas check run --suite ci`
- Local entrypoints: `bijux dev atlas doctor` and `bijux dev atlas check run --suite local`

## Output And Exit Contracts

- `bijux dev atlas` machine-readable outputs are JSON with explicit `schema_version`.
- Exit codes are documented in `crates/bijux-dev-atlas/docs/EXIT_CODES.md` and treated as the control-plane contract.
