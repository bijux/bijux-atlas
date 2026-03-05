---
title: CLI UX Benchmark
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

# CLI UX benchmark

CLI UX benchmark ownership is **dev-only** and is implemented in `bijux-dev-atlas`.

## Spec

- `configs/perf/cli-ux-benchmark-spec.json`

## Run benchmark

```bash
cargo run -p bijux-dev-atlas -- perf cli-ux bench --format json
```

## Compare runs

```bash
cargo run -p bijux-dev-atlas -- perf cli-ux diff \
  artifacts/perf/cli-ux/latest-report.json \
  artifacts/perf/cli-ux/latest-report.json \
  --format json
```

## Artifacts

- `artifacts/perf/cli-ux/latest-report.json`
- `artifacts/perf/cli-ux/latest-summary.md`
- `artifacts/perf/cli-ux/latest-diff.json`
- `artifacts/perf/cli-ux/raw/*.log`
