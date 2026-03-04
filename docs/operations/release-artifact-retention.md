# Release Artifact Retention

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: align artifact retention with the evidence retention policy.

## Retention Rules

- Keep `release/evidence/` and `release/signing/` together for each promoted release.
- Keep `release/provenance.json` alongside the signed bundle for the full supported release lifetime.
- Retain the chart package, bundle tarball, checksum list, and sign/verify reports for every promoted release.

## Alignment

- This policy extends [Evidence Retention Policy](evidence-retention-policy.md).
- Evidence without matching signing artifacts is not considered a complete retained release set.
