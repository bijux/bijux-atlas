---
title: Error taxonomy
audience: operators
type: reference
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags: [resilience, errors]
---

# Error taxonomy

| Code | Class | Operator action |
|---|---|---|
| `OPS_FAILURE_INGEST_CRASH` | availability | Restart ingest from last checkpoint and verify dataset state. |
| `OPS_FAILURE_QUERY_CRASH` | availability | Restart query runtime and validate cache manifest. |
| `OPS_FAILURE_INVALID_CONFIG` | configuration | Fix config, rerun validation, then restart runtime. |
| `OPS_FAILURE_MISSING_ARTIFACT` | supply | Restore artifact and rerun integrity check before boot. |
| `OPS_FAILURE_CORRUPTED_SHARD` | integrity | Quarantine shard and recover from healthy replica. |
| `OPS_FAILURE_DISK_FULL` | capacity | Free space, confirm filesystem health, retry ingest. |
| `OPS_FAILURE_OOM` | capacity | Increase memory budget and apply low-resource tuning. |
| `OPS_FAILURE_SLOW_QUERY` | performance | Inspect query plan and index/cache configuration. |
