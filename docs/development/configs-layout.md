# Configs Layout

`configs/` is the canonical config surface grouped by domain.

## Core Areas
- `configs/ops`: ops runtime/tool/policy/slo configs.
- `configs/policy`: runtime policy + relaxation registry.
- `configs/perf`: load/perf thresholds.
- `configs/openapi`: OpenAPI snapshot/generated outputs.
- `configs/rust`: rustfmt/clippy policy docs.
- `configs/security`: deny/audit policy docs.

## Contracts
- Every top-level `configs/<area>/` must contain `README.md`.
- Ownership map: `configs/inventory/owners.json`.
- Schemas live under `configs/schema/`.
- Generated surfaces:
  - `docs/_generated/configs-surface.md`
  - `docs/_generated/tooling-versions.md`
