# Release Promotion Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@0f088c31314aa61bc0ec69f1f5e683625b0d6bd5`
- Last changed: `2026-03-03`
- Reason to exist: define how a release candidate moves to general availability.

## Promotion Rules

- A release candidate is not promotable until `ops evidence verify` and `release verify` both pass.
- Promotion must use the same evidence bundle and signing set that were reviewed.
- If any artifact changes after review, regenerate the bundle, re-sign it, and restart verification.

## Re-signing Rule

- Promotion from release candidate to general availability requires a fresh verification pass on the final retained bundle.
- If the signing policy changes between release candidate and general availability, the bundle must be re-signed and re-reviewed.
