# Load testing

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define when to run load tests and how to interpret outcomes.

## When to run

- Before release promotion.
- After query-path or store changes.
- During incident reproduction for performance regressions.

## Core guides

- [k6 execution](k6.md)
- [Suite catalog](suites.md)
- [Load dashboards](dashboards.md)
- [Load testing strategy](strategy.md)
- [Load testing philosophy](load-testing-philosophy.md)
- [Load scenario specification](load-scenario-specification.md)
- [Load test metrics](metrics.md)
- [Performance baseline metrics](performance-baseline-metrics.md)
- [Capacity planning methodology](capacity-planning-methodology.md)
- [Performance regression thresholds](performance-regression-thresholds.md)
- [Load harness and generators](harness-and-generators.md)
- [Load testing documentation](documentation.md)
- [Load testing quickstart](quickstart.md)
- [Load testing troubleshooting](troubleshooting.md)
- [Load architecture diagram](architecture-diagram.md)
- [Load example configs](example-configs.md)
- [Load comparison tools](comparison-tools.md)
- [Load summary report](summary-report.md)
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
