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
bijux-dev-atlas datasets validate --format json
```

2. Run tutorial ingestion preview:

```bash
bijux-dev-atlas ingest dry-run --format json
```

3. Follow ingest and query tutorials.
