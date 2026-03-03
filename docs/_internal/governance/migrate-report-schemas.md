# Migrate Report Schemas

- Owner: `bijux-atlas-governance`
- Type: `how-to`
- Audience: `contributor`
- Stability: `stable`

## Example

- When a governed report schema version changes, add `docs/reference/reports/migrations/<id>.md`.
- Add a `report-schema` entry in `configs/governance/deprecations.yaml`.
- Update the producer and validate `governance breaking validate`.
