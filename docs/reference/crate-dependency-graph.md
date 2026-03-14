---
title: Crate dependency graph
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - crates
related:
  - docs/reference/crates.md
---

# Crate dependency graph

```mermaid
graph TD
  atlas[bijux-atlas]
  python[bijux-atlas-python]
  dev[bijux-dev-atlas]

  dev --> atlas
```

This graph reflects workspace crate-level dependencies, not third-party crates.
