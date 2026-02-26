# Ownership Rotation Policy

- Authority Tier: `tier2`
- Audience: `operators`
- Owner: `bijux-atlas-operations`
- Purpose: `define reviewer and operator ownership rotation cadence for ops domains and drills`
- Consumers: `checks_ops_human_workflow_maturity`

## Rotation Scope
- Domain owners in `ops/inventory/owners.json`
- Drill ownership in `ops/observe/drills/OWNERSHIP.md`
- Release readiness and evidence sign-off approvers

## Rotation Cadence
- Review rotation assignments on a recurring schedule.
- Record rotation effective date and next review date.

## Handover Requirements
- Current responsibilities
- Pending incidents or freezes
- Open deprecations and cutover dates
- Required runbooks and dashboards

## Enforcement Links
- `checks_ops_human_workflow_maturity`
