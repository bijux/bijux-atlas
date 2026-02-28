# Kubernetes E2E Guarantees

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: describe what Kubernetes E2E coverage guarantees before release.

## What this suite guarantees

- Chart installs with required defaults.
- Policy controls remain enforced.
- Rollout and rollback paths complete without orphaned resources.
- Readiness and health probes converge.

## What this suite does not guarantee

- Long-duration capacity planning.
- Production-specific cloud policy behavior outside Atlas chart controls.

## Commands

```bash
make k8s-validate
make ops-k8s-tests
```

## Verify success

Expected result: both commands pass and produce evidence in `artifacts/evidence/k8s/`.

## Next

- [End-to-end Tests](index.md)
- [Incident Response](../incident-response.md)
