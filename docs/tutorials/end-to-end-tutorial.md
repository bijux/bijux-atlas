---
title: End-to-End Tutorial
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - tutorial
  - end-to-end
related:
  - docs/tutorials/index.md
  - docs/reference/examples/index.md
  - docs/operations/workflows.md
---

# End-to-End Tutorial

## Goal

Run a complete Atlas flow from dataset ingest to validation, promotion, and query verification.

## Steps

1. Ingest dataset:

```bash
make ops-release-update
```

2. Run validation suites:

```bash
make check-all
make contract-all
cargo nextest run
```

3. Verify docs integrity:

```bash
cargo run -q -p bijux-dev-atlas -- docs links --strict --format json
```

4. Execute query example and confirm response structure.

## Expected result

All validation gates pass and example query returns contract-compliant output.
