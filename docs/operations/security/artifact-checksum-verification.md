---
title: Artifact Checksum Verification
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Artifact Checksum Verification

Checksum verification is implemented by `verify_artifact_checksum`.

- expected SHA-256 is compared against computed digest
- mismatch is treated as integrity failure and triggers tamper handling
