---
title: Tutorial Quickstart
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - quickstart
related:
  - docs/tutorials/example-datasets.md
  - docs/tutorials/ingest-dataset.md
---

# Tutorial Quickstart

1. Validate tutorial dataset:

```bash
tutorials/scripts/validate_example_dataset.py configs/examples/datasets/atlas-example-minimal
```

2. Run tutorial CLI flow:

```bash
tutorials/scripts/tutorial_cli_workflow.sh
```

3. Follow ingest and query tutorials.
