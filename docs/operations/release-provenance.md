---
title: Release Provenance
audience: operator
type: concept
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Release Provenance

- Owner: `bijux-atlas-operations`
- Type: `concept`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: explain what the release evidence proves and what it does not prove.

## Provenance Claims

- `release/evidence/identity.json` binds the bundle to one release ID, one git SHA, and one governance version.
- `release/evidence/manifest.json` declares the exact evidence members and their checksums.
- `release/evidence/bundle.tar` proves stable packaging order and normalized timestamps for the files included in the bundle.
- `release/evidence/sboms/*.spdx.json` document the digest-pinned image inputs used by governed profiles.

## Limits

- The bundle proves what was collected locally; it does not attest to third-party registry behavior after collection.
- Optional scan reports are included as evidence only. They do not change bundle validity unless your release gate requires them.
- Redacted logs are review aids, not a substitute for access-controlled raw runtime logs.

## Review Use

Use this page when an auditor asks what the evidence package demonstrates about reproducibility, traceability, and supply-chain closure.
