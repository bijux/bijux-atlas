# Schema Evolution Workflow

- Owner: `bijux-atlas-operations`
- Purpose: define governed schema evolution flow.
- Consumers: `checks_ops_human_workflow_maturity`

## Workflow

1. propose schema change.
2. update compatibility lock and consumers.
3. validate contract suite.

## Required Inputs

- ops/schema/VERSIONING_POLICY.md
- ops/schema/generated/compatibility-lock.json

## Enforcement Links

- checks_ops_human_workflow_maturity
