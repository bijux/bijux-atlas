---
title: CLI UX Benchmark Mechanics
audience: developer
type: reference
stability: stable
owner: bijux-dev-atlas
last_reviewed: 2026-03-05
tags:
  - perf
  - cli
  - benchmark
---

# CLI UX benchmark mechanics

`bijux-dev-atlas perf cli-ux bench` executes a deterministic command sequence from
`configs/perf/cli-ux-benchmark-spec.json`.

Execution model:

1. Resolve benchmark mode (`cold-start`, `warm-start`, `completion`).
2. Execute warmup runs (excluded from percentile calculation).
3. Execute measured runs and capture per-run duration.
4. Persist raw stdout/stderr logs per sample.
5. Compute p50/p95/p99 and compare against configured thresholds.
6. Emit machine report and markdown summary.

Artifacts:

- `artifacts/perf/cli-ux/latest-report.json`
- `artifacts/perf/cli-ux/latest-summary.md`
- `artifacts/perf/cli-ux/raw/sample-*.stdout.log`
- `artifacts/perf/cli-ux/raw/sample-*.stderr.log`
