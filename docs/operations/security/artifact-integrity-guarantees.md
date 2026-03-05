---
title: Artifact Integrity Guarantees
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Artifact Integrity Guarantees

Atlas guarantees artifact integrity before use through deterministic checks.

## Guarantees

- artifact checksum verification
- manifest integrity verification
- signature verification where signing key metadata is configured
- tamper detection signal emission on checksum mismatch

## Implementation references

- `crates/bijux-atlas-core/src/domain/security_data_protection.rs`
- `crates/bijux-atlas-server/src/runtime/dataset_cache_manager_storage_runtime/storage_methods.rs`
