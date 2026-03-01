# Dev Control Plane Artifacts

## Root

- Default artifact root: `artifacts/atlas-dev`
- Commands may accept `--artifacts-root <path>`; writes remain capability-gated.
- Layout SSOT pattern: `artifacts/<run-kind>/<run-id>/...`
- `bijux dev atlas` maps `run-kind` to `atlas-dev/<domain>` (for example `atlas-dev/ops`, `atlas-dev/docs`, `atlas-dev/configs`, `atlas-dev/checks`).

## Run IDs

- Use `--run-id` to make output locations deterministic and reproducible.
- If omitted, commands derive a deterministic default according to command contract (or emit without writes).

## Layout

- Checks: `artifacts/atlas-dev/checks/<run-id>/...`
- Docs: `artifacts/atlas-dev/docs/<run-id>/...`
- Configs: `artifacts/atlas-dev/configs/<run-id>/...`
- Ops: `artifacts/atlas-dev/ops/<run-id>/...`

## Determinism Rules

- Index files must sort paths deterministically.
- Hashes must use `sha256` and be emitted as lowercase hex.
- Committed references to artifacts must not embed timestamps in evidence paths.
- Writes require explicit effect flags; read-only commands must not create artifact files implicitly.
- Only control-plane commands may write under control-plane artifact roots.
