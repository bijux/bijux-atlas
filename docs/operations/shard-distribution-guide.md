---
title: Shard Distribution Guide
audience: operator
type: runbook
stability: evolving
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - sharding
---

# Shard Distribution Guide

## Commands

- `bijux-dev-atlas system cluster shard-list`
- `bijux-dev-atlas system cluster shard-distribution`
- `bijux-dev-atlas system cluster shard-rebalance`

## Workflow

1. Inspect current distribution.
2. Trigger rebalance when skew exceeds policy.
3. Confirm post-rebalance distribution is stable.
