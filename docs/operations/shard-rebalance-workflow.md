---
title: Shard Rebalance Workflow
audience: operator
type: runbook
stability: experimental
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - sharding
---

# Shard Rebalance Workflow

1. Capture baseline shard distribution.
2. Execute `system cluster shard-rebalance`.
3. Validate shard ownership changes.
4. Verify shard metrics remain healthy.
