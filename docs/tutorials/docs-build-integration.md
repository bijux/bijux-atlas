---
title: Tutorial Documentation Build Integration
audience: user
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - docs
  - build
---

# Tutorial Documentation Build Integration

Tutorial pages are built as part of MkDocs site generation.

Validation command:

```bash
mkdocs build --strict
```

Recommended pre-check script:

```bash
tutorials/scripts/build_tutorial_docs.sh
```
