# Exception Review Checklist

- Owner: `bijux-atlas-governance`
- Type: `checklist`
- Audience: `reviewers`
- Stability: `stable`
- Last reviewed: `2026-03-03`
- Reason to exist: define the minimum review questions for governed exception approval and renewal.

## Checklist

- Confirm the scope points to a real contract or check id.
- Confirm the exception does not target a listed no-exception zone.
- Confirm `tracking_link` points to the mitigation issue used to remove the exception.
- Confirm `risk_accepted_by` names the approver who accepts the residual risk.
- Confirm `verification_plan` explains how the team will prove the exception can be closed.
- Confirm the expiry fits the severity SLA, or that governance approval is explicitly recorded for a high-severity overrun.
- Confirm the exception still narrows risk instead of silently redefining the default posture.
