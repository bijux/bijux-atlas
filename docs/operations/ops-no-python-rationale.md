---
title: Why Ops Has No Python
audience: contributor
type: explanation
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - operations
  - governance
---

# Why Ops Has No Python

Automation logic is centralized in `bijux-dev-atlas` so repository behavior stays deterministic and reviewable in one runtime.

Keeping `ops/` free of Python and shell scripts prevents hidden execution paths and duplicate tooling stacks.

When new operational behavior is needed:

1. Implement it in `crates/bijux-dev-atlas`.
2. Expose it through a stable command surface.
3. Store only artifacts and fixtures in `ops/`.
