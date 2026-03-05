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
cargo run -p bijux-dev-atlas -- perf cli-ux bench --mode cold-start --format json
cargo run -p bijux-dev-atlas -- perf cli-ux bench --mode warm-start --format json
cargo run -p bijux-dev-atlas -- perf cli-ux bench --mode completion --format json
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

## Related references

- [CLI UX Benchmark Mechanics](./cli-ux-how-it-works.md)
- [CLI UX Benchmark Interpretation](./cli-ux-interpretation.md)
- [CLI UX Baseline Update Procedure](./cli-ux-baseline-update.md)
