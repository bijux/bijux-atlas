# Store backend error spike

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Last changed: `2026-03-03`
- Reason to exist: triage elevated store-backed error ratios before they become a full outage.

## Prereqs

- Access to current alerts, store dashboards, and recent rollout history.

## Install

- Start the store error triage flow and determine whether the issue is localized or broad.

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

1. Reduce load or rollback the latest risky release if the spike started after change.
2. Escalate to [Store outage](store-outage.md) if errors turn into broad unavailability.

## Verify

Error-rate burn returns to baseline and the alert clears without rolling into a harder outage page.

## Verify success

Error-rate burn returns to baseline and the alert clears without rolling into a harder outage page.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Postmortem checklist

- Capture timeline, impact, contributing factors, and permanent corrective actions.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## Next

- [Store outage](store-outage.md)
- [Traffic spike](traffic-spike.md)
