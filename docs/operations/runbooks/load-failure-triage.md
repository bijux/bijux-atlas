# Load failure triage

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: explain how to diagnose a failed load suite and decide whether it blocks release.

## Symptoms

- Key user-visible and operational signals indicating this condition.

## Metrics

- Primary SLO and saturation metrics to validate detection and recovery.

## Commands

```bash
make ops-load-smoke
make ops-load-nightly
make ops-observability-verify
```

## Expected outputs

- Health signals identify the failing component and blast radius.
- Metrics confirm whether mitigation improves service behavior.

## Mitigations

1. Separate threshold regression from environment noise before blocking promotion.
2. Escalate to [Traffic spike](traffic-spike.md) if the same pattern appears in live traffic.

## Verify success

The failing suite is either reproduced with clear evidence or downgraded to an explained non-blocker with no hidden ambiguity.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Postmortem checklist

- Capture timeline, impact, contributing factors, and permanent corrective actions.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## Next

- [Load testing](../load/index.md)
- [Traffic spike](traffic-spike.md)
