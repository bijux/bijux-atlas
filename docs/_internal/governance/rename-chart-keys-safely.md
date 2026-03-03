# Rename Chart Keys Safely

- Owner: `bijux-atlas-governance`
- Type: `how-to`
- Audience: `contributor`
- Stability: `stable`

## Example

- Keep both keys accepted in `ops/k8s/charts/bijux-atlas/values.schema.json`.
- Record the rename in `configs/governance/deprecations.yaml` with `surface: chart-value`.
- Move chart defaults and maintained overlays to the canonical key.
- Verify `ops profiles validate --allow-subprocess` emits no production compatibility warnings.
