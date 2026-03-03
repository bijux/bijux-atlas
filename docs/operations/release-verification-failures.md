# Release Verification Failures

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@0f088c31314aa61bc0ec69f1f5e683625b0d6bd5`
- Last changed: `2026-03-03`
- Reason to exist: provide triage steps when release integrity checks fail.

## Prereqs

- The failed `release verify` JSON output
- The matching `ops evidence verify` JSON output

## Install

- Check whether the failure is in `REL-SIGN-*`, `REL-PROV-001`, `REL-TAR-001`, or `REL-MAN-001`.
- If the failure is checksum-related, rerun `ops evidence collect` and `release sign` in that order.
- If the failure is schema-related, inspect `release/evidence/manifest.schema.json` and the generated manifest.

## Verify

- Re-run `ops evidence verify`.
- Re-run `release verify`.
- Confirm both commands return `status: ok`.

## Rollback

- Restore the last known good release evidence and signing files.
- Do not promote the candidate until the regenerated verification reports pass.
