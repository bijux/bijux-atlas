---
title: Encrypted Artifact Storage Option
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Encrypted Artifact Storage Option

Atlas supports encrypted artifact storage as a governed deployment option.

## Configuration

- `configs/security/data-protection.yaml`
- field: `encryption.dataset_encryption_optional`

This option should be enabled for deployments with strict storage-at-rest requirements.
