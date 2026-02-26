# Emergency Override Workflow

- Authority Tier: `machine`
- Audience: `operators`
- Owner: `bijux-atlas-operations`
- Purpose: `control emergency overrides with auditability, expiry, and rollback requirements`
- Consumers: `checks_ops_human_workflow_maturity`

## Workflow
- Declare emergency condition and impacted systems.
- Apply time-bounded override with owner approval.
- Capture evidence and rollback conditions.
- Remove override before expiry or renew with justification.
- Run post-incident review and document follow-up controls.

## Override Requirements
- Owner Approval
- Expiry Time
- Rollback Plan
- Audit Evidence

## Enforcement Links
- `checks_ops_human_workflow_maturity`
