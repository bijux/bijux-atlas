---
title: Developer CLI Output Style
audience: contributor
type: policy
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - cli
  - output
---

# Developer CLI output style

- JSON mode must emit deterministic canonical JSON.
- Human mode must be concise and predictable for log scanning.
- Command summaries should include a stable `kind` field in JSON payloads.
- Error payloads must include stable error codes.
- New commands must document JSON contract fields before release.
