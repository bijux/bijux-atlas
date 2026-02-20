# Chart Values Schema Contract

Canonical reference: `docs/contracts/chart-values.md`.

Generated reference list: `docs/_generated/contracts/CHART_VALUES.md`.

Policy:
- New top-level keys require updating `docs/contracts/chart-values.md` through the SSOT generation workflow.
- Contract drift is enforced by `bin/bijux-atlas contracts check --checks chart-values`.
- Default values must stay conservative and production-safe.
## Referenced chart values keys

- `values.server`
- `values.store`

## See also

- `ops-ci`
