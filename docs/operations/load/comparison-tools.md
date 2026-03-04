# Load Comparison Tools

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`

## Purpose

Document the supported comparison tool for baseline versus candidate load reports.

## Tool

- `ops/load/tools/compare-load-report.sh`

## Usage

```bash
ops/load/tools/compare-load-report.sh \
  ops/load/baselines/system-load-baseline.json \
  ops/load/baselines/system-load-baseline.json
```

Expected output is JSON with per-suite latency and error-rate deltas.
