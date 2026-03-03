# Release Signature Rotation

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@0f088c31314aa61bc0ec69f1f5e683625b0d6bd5`
- Last changed: `2026-03-03`
- Reason to exist: describe how release trust metadata changes without breaking bundle consumers.

## Prereqs

- Access to the release workspace and current signing policy
- A new verification instruction agreed by the release owner

## Install

- Update `release/signing/policy.yaml` to describe the new verification command or custody model.
- Regenerate the evidence bundle and signing artifacts.
- Run `release verify --evidence release/evidence/bundle.tar --format json`.

## Verify

- Confirm `REL-SIGN-*`, `REL-PROV-001`, `REL-TAR-001`, and `REL-MAN-001` all pass.
- Confirm consumers can still validate the bundle offline using the updated verification command.

## Rollback

- Restore the last known good `release/signing/policy.yaml`.
- Regenerate the signing artifacts and verify again before promotion resumes.
