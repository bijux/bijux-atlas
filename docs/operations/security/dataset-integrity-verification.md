---
title: Dataset Integrity Verification
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Dataset Integrity Verification

Dataset integrity is enforced by manifest and artifact checksum validation during ingest and cache lifecycle.

Core function:

- `verify_dataset_manifest_integrity`

Runtime enforcement:

- strict checksum re-verification in dataset cache manager
