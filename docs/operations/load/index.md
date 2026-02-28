# Load Testing

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

## Verify success

```bash
make ops-load-smoke
```

Expected result: smoke suite completes with no threshold failures.

## Next

- [Observability](../observability/index.md)
- [Release Workflow](../release-workflow.md)
