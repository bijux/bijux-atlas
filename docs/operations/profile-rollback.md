# Profile Rollback

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the rollback contract for profile-level changes.

## Prereqs

- Keep the last known-good overlay for the affected profile.
- Keep the matching metadata entry in `ops/k8s/values/profiles.json`.

## Install

Reapply the last known-good overlay file content for the affected profile.

## Verify

Confirm the overlay still satisfies the matching `profiles.json` requirements and any
`rollout-safety-contract.json` entry for that profile.

## Rollback

If the rollback candidate itself drifts from the profile registry, restore both the overlay and the
registry entry from the same known-good revision.
