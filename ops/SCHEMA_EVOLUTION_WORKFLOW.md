# Schema Evolution Workflow

- Owner: `bijux-atlas-operations`
- Purpose: `define the human workflow for schema changes, compatibility review, and release gating`
- Consumers: `checks_ops_human_workflow_maturity`
- Authority Tier: `tier2`
- Audience: `contributors`

## Workflow

1. Propose schema change in `ops/schema/**` with clear compatibility intent.
2. Update authored contracts that reference the schema.
3. Regenerate schema index and compatibility lock outputs.
4. Review compatibility impact against `ops/schema/generated/compatibility-lock.json`.
5. Run ops governance checks and document evidence bundle impact.
6. Merge only with schema reviewer sign-off and release readiness acknowledgment.

## Required Inputs

- `ops/schema/VERSIONING_POLICY.md`
- `ops/schema/generated/compatibility-lock.json`
- `ops/schema/generated/schema-index.json`
- `ops/inventory/contracts-map.json`

## Approval and Escalation

- Schema changes that alter required fields or semantics require explicit compatibility review.
- Breaking schema changes require release readiness sign-off and migration documentation in the same commit series.

## Enforcement Links

- `checks_ops_schema_presence`
- `checks_ops_human_workflow_maturity`
