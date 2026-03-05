---
title: Tutorial Generated Content Workflow
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - docs
  - generated-content
related:
  - docs/tutorials/reference.md
  - docs/tutorials/generated-content-update-guide.md
---

# Tutorial Generated Content Workflow

Tutorial references under `docs/_generated/` are produced by `bijux-dev-atlas`.

Generation commands:

```bash
bijux-dev-atlas docs generate examples --allow-write --allow-subprocess
bijux-dev-atlas docs verify-generated
```

`docs generate examples` updates all tutorial generated snippets:

- `docs/_generated/command-lists.md`
- `docs/_generated/schema-snippets.md`
- `docs/_generated/openapi-snippets.md`
- `docs/_generated/ops-snippets.md`
- `docs/_generated/examples.md`

`docs verify-generated` validates presence, generator header, and exact content parity.
