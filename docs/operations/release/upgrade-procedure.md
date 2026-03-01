# Upgrade Procedure

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define the chart and runtime upgrade path.

## Why you are reading this

Use this procedure when moving to a newer Atlas chart or runtime version.

## Procedure

1. Confirm current environment health.

```bash
make ops-readiness-scorecard
```

2. Apply upgrade through release update.

```bash
make ops-release-update
```

3. Re-run k8s and observability checks.

```bash
make ops-k8s-tests
make ops-observability-verify
make ops-e2e-smoke
```

## Verify success

Expected result: upgraded release passes Kubernetes, readiness, and observability checks.

## Rollback

If upgrade checks fail, run `make ops-release-rollback`.

## Next

- [Rollback Procedure](rollback-procedure.md)
- [Capacity Planning](capacity-planning.md)
