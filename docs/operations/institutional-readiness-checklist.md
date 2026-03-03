# Institutional Readiness Checklist

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@93ad533e5a4c4704f3a344db96b083570bb4d4b0`
- Reason to exist: define the minimum evidence package before external review or controlled rollout.

## Checklist

- Verify release evidence with `ops evidence verify`.
- Run at least one governed drill and confirm `ops-drills-summary.json` is current.
- Confirm the latest simulation summary and lifecycle summary are present.
- Confirm observability verification is passing.
- Confirm rollback documentation is current for the selected profile.
