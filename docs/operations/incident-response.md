# Incident response

- Owner: `bijux-atlas-operations`
- Audience: `operator`
- Type: `runbook`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: provide the first-15-minute response playbook for Atlas incidents.

## First 15 minutes

1. **Acknowledge**: page owner acknowledges and opens incident channel.
2. **Classify impact**: identify user impact and affected surfaces.
3. **Stabilize**: apply safest immediate mitigation from mapped runbook.
4. **Diagnose**: use observability triage to find the likely failing component.
5. **Escalate**: page secondary owner if recovery is not progressing in 10 minutes.
6. **Communicate**: publish status update with impact, mitigation, and next update time.

## Runbook selection

- Unknown cause: [Incident Playbook](runbooks/incident-playbook.md)
- Store degradation: [Store Backend Error Spike](runbooks/slo-store-backend-error-spike.md)
- Full store failure: [Store Outage](runbooks/store-outage.md)
- Dataset integrity issue: [Dataset Corruption](runbooks/dataset-corruption.md)
- Federation failure: [Registry Federation](runbooks/registry-federation.md)
- Capacity overload: [Traffic Spike](runbooks/traffic-spike.md)
- Failed mitigation: [Rollback Playbook](runbooks/rollback-playbook.md)

## Verify success

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

Expected result: service returns to SLO range and alert noise declines to baseline.

## Rollback

If mitigation fails, execute [Rollback Playbook](runbooks/rollback-playbook.md).

## Next

- [Observability](observability/index.md)
- [Runbooks](runbooks/index.md)
