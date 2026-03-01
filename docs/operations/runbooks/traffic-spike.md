# Traffic spike

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: stabilize the service during sustained overload or sudden traffic growth.

## Symptoms

- Key user-visible and operational signals indicating this condition.

## Metrics

- Primary SLO and saturation metrics to validate detection and recovery.

## Commands

```bash
make ops-readiness-scorecard
make ops-observability-verify
make ops-load-smoke
```

## Expected outputs

- Health signals identify the failing component and blast radius.
- Metrics confirm whether mitigation improves service behavior.

## Mitigations

1. Reduce pressure on the dominant expensive path.
2. Roll back the latest capacity-sensitive deploy if the spike started after rollout.

## Verify success

Latency and timeout alerts return to target range and the service stays ready under representative smoke load.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Postmortem checklist

- Capture timeline, impact, contributing factors, and permanent corrective actions.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## Next

- [Load failure triage](load-failure-triage.md)
- [Store backend error spike](slo-store-backend-error-spike.md)
