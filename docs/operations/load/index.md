# Load testing

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define when to run load tests and how to interpret outcomes.

## When to run

- Before release promotion.
- After query-path or store changes.
- During incident reproduction for performance regressions.

## Core guides

- [k6 execution](k6.md)
- [Suite catalog](suites.md)
- [Load Failure Triage](../runbooks/load-failure-triage.md)

## Threshold interpretation

- smoke failures block promotion until explained
- nightly failures require triage even when they are not immediate release blockers
- repeated latency regression without traffic growth is treated as a product or capacity regression, not test noise

## Verify success

```bash
make ops-load-smoke
make ops-load-nightly
```

Expected result: smoke suite completes with no threshold failures.

## Rollback

If a rollout caused the regression, use [Rollback playbook](../runbooks/rollback-playbook.md) after capturing the failing evidence.

## Next

- [Observability](../observability/index.md)
- [Release Workflow](../release-workflow.md)
