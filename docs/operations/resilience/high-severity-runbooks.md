---
title: High severity runbooks
audience: operators
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags: [resilience, runbooks]
---

# High severity runbooks

- `OPS_FAILURE_CORRUPTED_SHARD`: isolate shard, fail closed, recover from verified source.
- `OPS_FAILURE_DISK_FULL`: stop writes, preserve diagnostics, reclaim capacity, replay safely.
- `OPS_FAILURE_INVALID_CONFIG`: block boot, emit parse diagnostics, require explicit fix.
- `OPS_FAILURE_MISSING_ARTIFACT`: fail fast, restore artifact chain, rerun integrity checks.
- `OPS_FAILURE_OOM`: capture crash report, scale memory, retest under low-resource profile.
