# Security Key Rotation

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@a98808392299dfcbf57f73e25722d2b7070f72e4`
- Reason to exist: define a controlled rotation path for CI publishing tokens and release trust metadata.

## Prereqs

- Access to the CI secret store that backs publishing
- Access to `release/signing/policy.yaml`
- A clean working tree for publishing metadata updates

## Install

- Rotate the CI publishing token in the secret store
- Update any repository metadata that documents the trust boundary or verification command
- Run `bijux-dev-atlas security validate --format json`
- Regenerate release evidence if the rotation changes verification metadata

## Verify

- Confirm `release/signing/policy.yaml` still references the current verification command
- Confirm no raw token material appears in `release/evidence/`
- Confirm downstream verification still succeeds using the rotated trust material

## Rollback

- Revoke the rotated token if validation fails
- Restore the last known good token only long enough to recover release continuity
- Repeat rotation with corrected metadata before resuming normal publishing
