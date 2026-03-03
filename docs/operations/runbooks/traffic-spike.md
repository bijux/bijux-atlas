# Traffic spike

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Last changed: `2026-03-03`
- Reason to exist: stabilize the service during sustained overload or sudden traffic growth.

## Prereqs

- Access to latency, traffic, and queue depth signals for the live system.

## Install

- Start the overload mitigation flow and reduce pressure on the dominant path.

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

## Verify

Latency and timeout alerts return to target range and the service stays ready under representative smoke load.

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
