---
title: CLI UX Benchmark Interpretation
audience: developer
type: guide
stability: stable
owner: bijux-dev-atlas
last_reviewed: 2026-03-05
tags:
  - perf
  - cli
  - benchmark
---

# CLI UX benchmark interpretation

Key fields:

- `latency_ms.p50`: median user experience.
- `latency_ms.p95`: tail latency target for routine usage.
- `latency_ms.p99`: worst-case tail latency budget.
- `thresholds_ms`: regression thresholds from benchmark spec.
- `threshold_flags`: per-metric regression indicators in diff output.

Status rules:

- `ok`: no sample failures and threshold checks passed.
- `failed`: sample failures or threshold regressions present.

Comparison flow:

```bash
cargo run -p bijux-dev-atlas -- perf cli-ux diff <baseline> <current> --format json
```
