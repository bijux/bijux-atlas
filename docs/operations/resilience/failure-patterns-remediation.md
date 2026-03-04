---
title: Failure patterns and remediation
audience: operators
type: guide
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags: [resilience, remediation]
---

# Failure patterns and remediation

- Crash during ingest: restart from checkpoint and verify dataset registry.
- Crash during query: restart runtime and validate cache integrity.
- Config parse failure: fix config contract violations before boot.
- Missing artifact: restore release artifacts and rerun integrity checks.
- Disk pressure: enforce write throttling and reclaim storage.
- Slow query: inspect query plan and tune index/cache settings.
