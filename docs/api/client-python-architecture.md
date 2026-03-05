---
title: Python Client Architecture
audience: user
type: reference
stability: experimental
owner: api-contracts
last_reviewed: 2026-03-05
tags:
  - api
  - python
  - sdk
---

# Python Client Architecture

Source of truth: `packages/bijux-atlas-python/docs/architecture.md`.

The Python SDK uses a minimal transport core with explicit configuration validation,
retry behavior, and optional logging/tracing hooks.
