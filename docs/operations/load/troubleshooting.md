# Load Testing Troubleshooting

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Purpose

Provide deterministic troubleshooting flow for failing load suites.

## Checklist

1. Validate JSON syntax for suite and threshold files.
2. Confirm scenario file referenced by suite exists under `ops/load/scenarios/`.
3. Confirm k6 script referenced by scenario exists under `ops/load/k6/suites/`.
4. Run suite plan command and inspect emitted `errors`.
5. Regenerate report and inspect `violations` fields.
6. Compare report against baseline references in `ops/load/baselines/`.

## Common Failures

- `unknown load suite`: suite name mismatch.
- `dataset path missing`: query pack or dataset file path mismatch.
- `threshold breach p95/p99/error_rate`: performance regression or environment pressure.
