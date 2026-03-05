---
title: Artifact Signature Verification
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Artifact Signature Verification

Signature validation is implemented by `verify_artifact_signature`.

Inputs:

- artifact checksum
- provided signature
- signing key identifier

Verification failure blocks artifact acceptance.
