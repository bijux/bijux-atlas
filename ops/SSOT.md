# Ops SSOT Policy

This page defines authored truth versus generated truth in `ops/`.

## Authored

- Author intent files under:
  - `ops/inventory/`
  - `ops/schema/` (source schemas, not generated indexes)
  - `ops/env/`
  - `ops/stack/` and `ops/k8s/` source manifests
  - `ops/observe/`, `ops/load/`, `ops/datasets/`, `ops/e2e/`, `ops/report/`

## Generated

- Runtime generated outputs: `ops/_generated/`
- Curated committed generated evidence: `ops/_generated.example/`
- Generated indexes and summaries under domain `generated/` directories

## Rule

- Semantic data can have one authored source only.
- Any additional copy must be generated from the authored source and documented as generated.
- Ops markdown is spec-oriented: keep only canonical headers (`README.md`, `INDEX.md`, `CONTRACT.md`, `REQUIRED_FILES.md`, `OWNER.md`) plus explicit governance allowlist files.
- `ops/` is not a handbook surface; workflow and tutorial prose belongs under `docs/`.

## Authority Graph

- `ops/inventory/**`: authoritative operational graph and registry metadata.
- `ops/schema/**`: validation and compatibility contracts for inventory/domain artifacts.
- `docs/**`: human-readable guidance that must reference inventory/schema truth, never redefine it.
