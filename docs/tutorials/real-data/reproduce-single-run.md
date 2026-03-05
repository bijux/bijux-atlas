---
title: Reproduce Single Real Data Run
audience: reviewer
type: how-to
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
---

# Reproduce Single Real Data Run

Example for `rdr-001-genes-baseline`:

```bash
bijux-dev-atlas tutorials real-data fetch --run-id rdr-001-genes-baseline --format json
bijux-dev-atlas tutorials real-data ingest --run-id rdr-001-genes-baseline --profile local --format json
bijux-dev-atlas tutorials real-data query-pack --run-id rdr-001-genes-baseline --profile local --format json
bijux-dev-atlas tutorials real-data export-evidence --run-id rdr-001-genes-baseline --profile local --format json
```

Validation:

```bash
bijux-dev-atlas tutorials real-data compare-regression --run-id rdr-001-genes-baseline --format json
bijux-dev-atlas tutorials real-data verify-idempotency --run-id rdr-001-genes-baseline --format json
```
