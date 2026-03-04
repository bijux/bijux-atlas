---
title: How This Repo Enforces Itself
audience: contributor
type: concept
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - governance
  - enforcement
related:
  - docs/contract.md
  - docs/control-plane/index.md
---

# How This Repo Enforces Itself

Atlas uses executable contracts and checks as enforcement, not advisory text.

## Enforcement loop

1. Policy and schema inputs live in governed SSOT files.
2. Control-plane commands generate and verify evidence artifacts.
3. Contracts/checks fail CI when drift or policy violations appear.
4. Documentation, generated outputs, and behavior stay synchronized.
