# Baseline Update Policy

- Owner: `bijux-atlas-operations`

## What

Defines explicit approval process for updating performance baselines.

## Why

Prevents accidental acceptance of regressions as new normal.

## Contracts

- Baselines live in `ops/load/baselines/`.
- Baseline changes require explicit approval via `ATLAS_BASELINE_APPROVED=1`.
- Enforcement script: `scripts/perf/check_baseline_update_policy.sh`.

## How to verify

```bash
$ ATLAS_BASELINE_APPROVED=1 scripts/perf/check_baseline_update_policy.sh
```

Expected output: baseline policy check passes when approved.

## See also

- [Load Reproducibility](reproducibility.md)
- [Load Suites](suites.md)
- `ops-load-full`
