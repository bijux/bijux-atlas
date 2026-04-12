---
title: Signing and Provenance
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Signing and Provenance

Release integrity depends on declared signing and provenance inputs rather than
after-the-fact manual notes.

## Purpose

Use this page to verify the release trust chain before any distribution or
installation claim is made.

## Source of Truth

- `ops/release/signing/checksums.json`
- `ops/release/signing/release-sign.json`
- `ops/release/signing/release-verify.json`
- `ops/release/provenance.json`
- `ops/release/signing/policy.yaml`

## Trust Chain

The release trust chain currently ties together:

- the checksum inventory for governed release artifacts
- the signing output generated for the release
- the verification output, which records contract checks and overall status
- provenance that binds the release to Git identity, policy path, and toolchain

## Operator Verification Path

Before distribution, confirm:

- `checksums.json` covers the required release artifacts
- `release-verify.json` reports `status: ok`
- provenance points to the expected release identity and signing policy
- the evidence manifest and checksums still agree on the artifact set

## Related Contracts and Assets

- `ops/release/signing/`
- `ops/release/evidence/`
- `ops/release/provenance.json`
