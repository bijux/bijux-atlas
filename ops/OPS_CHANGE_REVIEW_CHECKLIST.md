# Ops Change Review Checklist

- Owner: `bijux-atlas-operations`
- Purpose: `standardize human review expectations for ops contract and runtime governance changes`
- Consumers: `checks_ops_human_workflow_maturity`
- Authority Tier: `tier2`
- Audience: `reviewers`

## Checklist

- [ ] Authority updated: `ops/inventory/contracts-map.json` / `ops/inventory/authority-index.json` updated when paths changed
- [ ] Schema coverage updated or intentionally allowlisted
- [ ] Evidence impact reviewed (`ops/report/generated/*` and curated examples)
- [ ] Compatibility impact reviewed (schema lock, pins, or workflow routing)
- [ ] Deletion safety reviewed (`ops/MINIMAL_RELEASE_SURFACE.md`, `ops/DIRECTORY_NECESSITY.md`)
- [ ] Generated lifecycle metadata updated if generated files changed
- [ ] Checks added/updated for new policy or contract behavior
- [ ] Commit boundaries are logical and messages describe durable intent

## Escalation Conditions

- Cross-domain contract changes require review from each affected domain owner.
- Release-blocking evidence changes require release readiness sign-off before merge.

## Enforcement Links

- `checks_ops_human_workflow_maturity`
