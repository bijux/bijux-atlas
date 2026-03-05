---
title: Data Protection Documentation
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Data Protection Documentation

Primary data protection sources:

- `configs/security/data-protection.yaml`
- `configs/security/data-classification.yaml`
- `docs/operations/security/data-classification-policy.md`
- `docs/operations/security/encryption-strategy.md`
- `docs/operations/security/artifact-integrity-guarantees.md`

Validation surfaces:

- `cargo test -p bijux-atlas-core --test security_data_protection_contracts`
- `bijux-dev-atlas security validate --format json`
