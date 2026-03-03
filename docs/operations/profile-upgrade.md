# Profile Upgrade

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: describe how to change a Kubernetes profile without silently changing its intent.

## Prereqs

- Compare the target overlay with `ops/k8s/values/profile-baseline.yaml`.
- Confirm the target profile metadata in `ops/k8s/values/profiles.json`.
- If the profile is in `ops/k8s/rollout-safety-contract.json`, update that contract in the same
  change.

## Install

1. Edit only the toggles that change the declared intent.
2. Keep forbidden toggles forbidden for the profile.
3. Add or update docs if the operator-facing behavior changes.

## Verify

Run `helm lint` against the changed profile and confirm the resulting toggles still match
`ops/k8s/values/profiles.json`.

## Rollback

Restore the previous overlay content and re-run the same validation before reattempting the change.
