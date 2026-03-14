---
title: Release Process
audience: operator
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - operations
  - release
related:
  - docs/operations/release/index.md
  - docs/operations/release-operations.md
---

# Release Process

This is the canonical release-process narrative.

## Core flow

1. Validate environment and prerequisites.
2. Build and verify release evidence.
3. Promote release with policy checks.
4. Run post-release verification.

Tutorial evidence is part of release evidence. Run `bijux-dev-atlas tutorials run workflow` before final promotion checks.

Use [Release index](ops/release/index.md) for full procedures and runbooks.
