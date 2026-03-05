---
title: Encryption Strategy
audience: user
type: strategy
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Encryption Strategy

Atlas encryption strategy covers transport encryption, storage encryption options, and integrity-linked verification.

## Source of truth

- `configs/security/data-protection.yaml`

## Controls

- transport TLS required for runtime communication
- optional dataset encryption at rest for managed artifacts
- cryptographic checksum and signature verification before serve
