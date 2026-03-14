# Benchmark Troubleshooting

## Schema validation fails

- Confirm `configs/contracts/perf/benchmark-result.schema.json` exists and is valid JSON.
- Confirm generated result has `schema_version`, `scenario`, `latency_ms.p99`, and `throughput_rps`.

## Reproducibility drifts

- Ensure fixed seed from benchmark configuration inputs is unchanged.
- Ensure CPU and memory isolation config are unchanged.
- Compare `artifacts/benchmarks/*-history.json` between runs.

## Regression false positives

- Re-run the same scenario twice locally.
- Validate machine load is stable.
- Use `bijux-dev-atlas perf diff <report-a> <report-b> --format json` to inspect metric deltas.
