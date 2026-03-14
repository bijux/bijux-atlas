# External Reviewer Guidance

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`

## Purpose

This guide explains how to read the governed release evidence without relying on internal build
context.

Related ops contracts: `OPS-ROOT-023`, `REL-PACK-001`.

## Review Order

1. Read `ops/release/evidence/identity.json` for release identity and governance version.
2. Read `ops/release/evidence/manifest.json` for the required artifact inventory.
3. Read `ops/release/signing/release-verify.json` to confirm the release integrity checks passed.
4. Inspect the institutional packet inventory to locate the minimum evidence subset.
5. Use the SBOMs and provenance file to trace toolchain and image inputs.

## What To Look For

- The manifest schema version must match the governed schema.
- The checksum ledger must cover every signed artifact listed in policy.
- The verification report must show all `REL-SIGN-*`, `REL-TAR-001`, `REL-MAN-001`, and
  `REL-PROV-001` contracts passing.
- The packet inventory must show `REL-PACK-001=true`.

## If Verification Fails

Use the release verification failures runbook first. Do not accept a bundle with a failed contract
unless there is a separately approved exception and replacement evidence.
