---
title: Encryption Configuration Guide
audience: user
type: guide
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Encryption Configuration Guide

Configuration file:

- `configs/security/data-protection.yaml`

Recommended settings:

- `encryption.transport_tls_required: true`
- `encryption.min_tls_version: "1.2"` or higher
- `encryption.dataset_encryption_optional: true` when at-rest encryption is required by policy

Verification:

```bash
cargo test -p bijux-atlas-core --test security_data_protection_contracts
```
