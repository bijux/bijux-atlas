# Incident playbook

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: provide the fallback runbook when the failing subsystem is not yet known.

## Symptoms

- Key user-visible and operational signals indicating this condition.

## Metrics

- Primary SLO and saturation metrics to validate detection and recovery.

## Commands

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

## Expected outputs

- Health signals identify the failing component and blast radius.
- Metrics confirm whether mitigation improves service behavior.

## Mitigations

1. Apply the safest stabilization action for the identified failure mode.
2. Reduce blast radius while preserving critical read paths.

## Verify success

Alerts quiet down, readiness recovers, and you can hand off to a more specific runbook if the subsystem is identified.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Postmortem checklist

- Capture timeline, impact, contributing factors, and permanent corrective actions.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## Next

- [Incident response](../incident-response.md)
- [Runbooks to alerts mapping](../runbooks-to-alerts-mapping.md)
