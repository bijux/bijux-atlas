---
title: Forbidden Repository Patterns
audience: contributor
type: policy
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - automation
  - policy
---

# Forbidden repository patterns

- Root `tools/` directory for automation scripts
- Root `scripts/` directory for automation scripts
- Root-level `*.sh` automation helpers
- Root-level `*.py` automation helpers
- Workflow steps that execute repository bash scripts directly
- Make wrappers containing business logic instead of delegation
