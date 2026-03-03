# Exception Inventory

- Owner: `bijux-atlas-governance`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last reviewed: `2026-03-03`
- Reason to exist: point contributors to the generated exception inventory artifacts without treating them as authored policy.

## Generated artifacts

- Summary JSON: `artifacts/governance/exceptions-summary.json`
- Read-only table: `artifacts/governance/exceptions-table.md`
- Expiry warning JSON: `artifacts/governance/exceptions-expiry-warning.json`
- Churn JSON: `artifacts/governance/exceptions-churn.json`

## Usage rules

- Treat the generated table as a view over `configs/governance/exceptions.yaml`.
- Do not edit generated exception inventory artifacts by hand.
- Update the registry and rerun `governance exceptions validate` when the governed set changes.
