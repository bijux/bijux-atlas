# Ops Canonical Layout

The canonical SSOT is `ops/CONTRACT.md`.

## Top-level layout

- `ops/stack/`
- `ops/k8s/`
- `ops/obs/`
- `ops/load/`
- `ops/datasets/`
- `ops/e2e/`
- `ops/run/`
- `ops/_lib/`
- `ops/_meta/`
- `ops/_schemas/`
- `ops/_generated/`
- `ops/_artifacts/`

## Rules

- Use Make targets from `ops/INDEX.md`.
- `ops/e2e/` is composition-only.
- `ops/run/` holds thin executable wrappers.
- No symlinked domain directories under `ops/`.
- Artifacts write to `ops/_artifacts/` unless allowlisted in `configs/ops/artifacts-allowlist.txt`.
