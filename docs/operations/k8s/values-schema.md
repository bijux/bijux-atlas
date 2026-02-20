# Chart Values Schema Contract

Canonical reference: `docs/contracts/chart-values.md`.

Generated reference list: `docs/_generated/contracts/CHART_VALUES.md`.

Policy:
- New top-level keys require updating `docs/contracts/chart-values.md` through the SSOT generation workflow.
- Contract drift is enforced by `scripts/areas/contracts/check_chart_values_contract.py`.
- Default values must stay conservative and production-safe.
## Referenced chart values keys

- `values.server`
- `values.store`

## See also

- `ops-ci`
