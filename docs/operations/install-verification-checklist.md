# Install verification checklist

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: define the canonical post-install verification pass for pods, probes, metrics, and logs.

## Checklist

- [ ] pods are running and not crash-looping
- [ ] readiness and liveness probes are healthy
- [ ] service endpoints respond to smoke checks
- [ ] metrics endpoint is scraped
- [ ] logs show startup completion without repeated fatal errors

## Verify success

```bash
make ops-readiness-scorecard
make ops-observability-verify
make ops-e2e-smoke
```

Expected outputs:

- pod status returns healthy workloads
- observability verification succeeds
- smoke or e2e verification reports success

## Rollback

Use [Rollback procedure](release/rollback-procedure.md) when install verification fails after a change rollout.

## Next steps

- [Observability setup](observability-setup.md)
- [Incident response](incident-response.md)
- [Runbooks](runbooks/index.md)
