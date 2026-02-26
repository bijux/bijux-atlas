# Example Incident Walkthrough

- Owner: `bijux-atlas-operations`
- Purpose: `show a representative incident response flow using runbooks, drills, and evidence contracts`
- Consumers: `checks_ops_final_polish_contracts`

## Scenario

- Store outage under load with degraded serving and recovery validation.

## Steps

1. Identify alert and impacted SLOs.
2. Open runbook (`docs/operations/runbooks/store-outage.md`).
3. Run/inspect corresponding drill evidence (`ops.drill.store_outage`).
4. Confirm observability and load evidence links in release evidence bundle.
5. Record remediation and verify readiness impact.

## Linked Contracts

- `ops/observe/drills/drills.json`
- `ops/inventory/scenario-slo-map.json`
- `ops/report/generated/release-evidence-bundle.json`
