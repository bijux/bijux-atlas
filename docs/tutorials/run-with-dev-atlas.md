---
title: Run Tutorials With Dev Atlas
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - automation
---

# Run tutorials with dev-atlas

Use `bijux-dev-atlas` for tutorial operations. Do not run local tutorial scripts.

## Core workflow

1. Inspect tutorial inventory:

```bash
bijux-dev-atlas tutorials list
```

2. Run the tutorial workflow:

```bash
bijux-dev-atlas tutorials run workflow
```

3. Verify tutorial contracts and checks:

```bash
bijux-dev-atlas tutorials verify
```

