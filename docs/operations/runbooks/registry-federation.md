# Registry federation

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Last changed: `2026-03-03`
- Reason to exist: recover when release alias or registry federation fails to converge.

## Prereqs

- Access to registry state, alias controls, and the last converged release pointer.

## Install

- Freeze promotion and inspect the federated registry state before changing aliases.

## Symptoms

- Key user-visible and operational signals indicating this condition.

## Metrics

- Primary SLO and saturation metrics to validate detection and recovery.

## Commands

```bash
make ops-prereqs
make ops-release-rollback
make ops-readiness-scorecard
```

## Expected outputs

- Health signals identify the failing component and blast radius.
- Metrics confirm whether mitigation improves service behavior.

## Mitigations

1. Freeze promotion while the registry state is inconsistent.
2. Roll back the alias or release pointer to the last converged state.

## Verify

Registry errors stop, aliases resolve deterministically again, and readiness checks pass.

## Verify success

Registry errors stop, aliases resolve deterministically again, and readiness checks pass.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Postmortem checklist

- Capture timeline, impact, contributing factors, and permanent corrective actions.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## Next

- [Release workflow](../release-workflow.md)
- [Rollback playbook](rollback-playbook.md)
