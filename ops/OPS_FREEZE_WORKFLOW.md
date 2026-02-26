# Ops Freeze Workflow

- Authority Tier: `machine`
- Audience: `operators`
- Owner: `bijux-atlas-operations`
- Purpose: `define freeze activation and release exception handling for ops changes`
- Consumers: `checks_ops_human_workflow_maturity`

## Workflow
- Declare freeze window scope, start, and end times.
- Record freeze reason and affected domains.
- Block non-exception ops changes during freeze.
- Require explicit exception approval for urgent changes.
- Close freeze with post-freeze review notes.

## Freeze Exceptions
- Exception requests must include risk, rollback, and evidence impact.
- Exception approvals must identify approving owner and expiry.

## Required Inputs
- `ops/RELEASE_READINESS_SIGNOFF_CHECKLIST.md`
- `ops/inventory/owners.json`
- `ops/inventory/gates.json`

## Enforcement Links
- `checks_ops_human_workflow_maturity`
