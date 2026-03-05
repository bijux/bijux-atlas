---
title: Reproduce All Real Data Runs
audience: reviewer
type: how-to
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
---

# Reproduce All Real Data Runs

Run the full sequence with resume support:

```bash
bijux-dev-atlas tutorials real-data run-all --profile local --format json
```

Offline deterministic mode:

```bash
bijux-dev-atlas tutorials real-data run-all --profile local --no-fetch --format json
```

Outputs are written under `artifacts/tutorials/runs/<run_id>/`.
