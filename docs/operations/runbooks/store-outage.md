# Store outage

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: restore service when the serving store is unavailable or timing out broadly.

## Symptoms

- Key user-visible and operational signals indicating this condition.

## Metrics

- Primary SLO and saturation metrics to validate detection and recovery.

## Commands

```bash
make ops-readiness-scorecard
make ops-observability-verify
make ops-release-rollback
```

## Expected outputs

- Health signals identify the failing component and blast radius.
- Metrics confirm whether mitigation improves service behavior.

## Mitigations

1. Remove pressure from the failing store path by reducing rollout or traffic blast radius.
2. Roll back the latest risky release if the outage started after deploy.

## Verify success

Readiness returns, store-facing alerts stop paging, and core read traffic succeeds again.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Postmortem checklist

- Capture timeline, impact, contributing factors, and permanent corrective actions.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## Next

- [Store backend error spike](slo-store-backend-error-spike.md)
- [Rollback playbook](rollback-playbook.md)
