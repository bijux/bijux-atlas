# Ops Deletion Impact ADR Summary

- Owner: `bijux-atlas-operations`
- Purpose: summarize deletion-impact simulation outcomes and canonical evidence location.
- Consumers: `checks_ops_final_polish_contracts`

## Decision Summary

Repository deletion-impact simulations are represented as governance summaries in source control.
Execution evidence is retained in artifacts outputs and referenced by run identifier.

## Evidence Location

- Canonical runtime evidence root: `artifacts/ops/<run_id>/`
- Curated example report: `ops/_generated.example/what-breaks-if-removed-report.json`

## Follow-up Rule

Any future simulation model change must update this summary and the associated evidence schema contract in the same change.
