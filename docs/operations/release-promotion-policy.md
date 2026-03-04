# Release Promotion Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define how a release candidate moves to general availability.

Related ops contracts: `OPS-ROOT-023`, `REL-SIGN-005`.

## Promotion Rules

- A release candidate is not promotable until `ops evidence verify` and `release verify` both pass.
- Promotion must use the same evidence bundle and signing set that were reviewed.
- If any artifact changes after review, regenerate the bundle, re-sign it, and restart verification.

## Re-signing Rule

- Promotion from release candidate to general availability requires a fresh verification pass on the final retained bundle.
- If the signing policy changes between release candidate and general availability, the bundle must be re-signed and re-reviewed.
