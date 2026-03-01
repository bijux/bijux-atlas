# Dataset corruption

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: recover safely when a promoted dataset or artifact fails integrity checks.

## Symptoms

- Key user-visible and operational signals indicating this condition.

## Metrics

- Primary SLO and saturation metrics to validate detection and recovery.

## Commands

```bash
make ops-readiness-scorecard
make ops-release-rollback
```

## Expected outputs

- Health signals identify the failing component and blast radius.
- Metrics confirm whether mitigation improves service behavior.

## Mitigations

1. Stop promotion of the corrupt dataset and move traffic back to the last known good release.
2. Preserve evidence before any cleanup or republish action.

## Verify success

Integrity errors stop recurring, the previous good dataset serves successfully, and rollback evidence is captured.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Postmortem checklist

- Capture timeline, impact, contributing factors, and permanent corrective actions.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## Next

- [Backup and restore](../release/backup-and-restore.md)
- [Rollback playbook](rollback-playbook.md)
