---
title: Tutorial Generated Content Update Guide
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
  - docs/tutorials/generated-content-workflow.md
  - docs/tutorials/generated-content-rules.md
---

# Tutorial Generated Content Update Guide

Use this update flow when command surfaces, schemas, or OpenAPI outputs change.

1. Regenerate snippets.

```bash
bijux-dev-atlas docs generate examples --allow-write --allow-subprocess --format json
```

2. Verify generated artifacts are current.

```bash
bijux-dev-atlas docs verify-generated --format json
```

3. Run docs build checks.

```bash
bijux-dev-atlas docs build --allow-subprocess --allow-write --format json
```

If `docs build` fails with stale generated content, regenerate and commit the updated files under `docs/_generated/`.
