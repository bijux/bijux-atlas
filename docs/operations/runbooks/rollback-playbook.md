# Rollback playbook

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: define the general rollback path used by deploy, release, and incident workflows.

## Symptoms

- Key user-visible and operational signals indicating this condition.

## Metrics

- Primary SLO and saturation metrics to validate detection and recovery.

## Commands

```bash
make ops-release-rollback
make ops-readiness-scorecard
make ops-observability-verify
```

## Expected outputs

- Health signals identify the failing component and blast radius.
- Metrics confirm whether mitigation improves service behavior.

## Mitigations

1. Revert to the last known good release pointer or deployment state.
2. Preserve failure evidence before attempting a second rollout.

## Verify success

Serving returns to the last known good behavior, alert pressure drops, and post-rollback checks pass.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Postmortem checklist

- Capture timeline, impact, contributing factors, and permanent corrective actions.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## Next

- [Release rollback procedure](../release/rollback-procedure.md)
- [Incident response](../incident-response.md)
