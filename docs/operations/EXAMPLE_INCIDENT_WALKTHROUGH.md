# Example Incident Walkthrough

- Owner: `bijux-atlas-operations`
- Purpose: show canonical incident response flow for operator training.
- Consumers: `checks_ops_final_polish_contracts`

## Scenario

Production query latency spike with partial error-rate increase.

## Steps

1. detect and acknowledge alert.
2. inspect health and rollout status.
3. execute rollback if error budget burn is sustained.
4. capture evidence and publish incident summary.

## Linked Contracts

- ops/OPS_INVARIANTS.md
- ops/THREAT_MODEL.md
