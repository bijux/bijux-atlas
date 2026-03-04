---
title: Shard Debugging Guide
audience: operator
type: runbook
stability: evolving
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - debugging
  - sharding
---

# Shard Debugging Guide

1. Run `system cluster shard-routing` for route verification.
2. Run `system cluster shard-diagnostics` for runtime stats.
3. Compare cache hit/miss and latency metrics.
4. Use `shard-rebalance` when skew is persistent.
