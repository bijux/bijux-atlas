# Institutional Readiness Checklist

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the minimum evidence package before external review or controlled rollout.

## Checklist

- Verify release evidence with `ops evidence verify`.
- Run at least one governed drill and confirm `ops-drills-summary.json` is current.
- Confirm the latest simulation summary and lifecycle summary are present.
- Confirm observability verification is passing.
- Confirm rollback documentation is current for the selected profile.
