# External Reviewer Guidance

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@61030ac8825c2ea5c35489baffaa663d7ab77045`

## Purpose

This guide explains how to read the governed release evidence without relying on internal build
context.

Related ops contracts: `OPS-ROOT-023`, `REL-PACK-001`.

## Review Order

1. Read `release/evidence/identity.json` for release identity and governance version.
2. Read `release/evidence/manifest.json` for the required artifact inventory.
3. Read `release/signing/release-verify.json` to confirm the release integrity checks passed.
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
