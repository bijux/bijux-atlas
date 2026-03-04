# Performance Operations

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@db6478db1e24e685969191669cb17ca4d2fe06fb`

## Purpose

This runbook defines the governed performance evidence surface for Atlas: SLO targets, local load
checks, cold-start timing, regression comparison, and the kind-backed perf validation gate.

Related ops contracts: `OPS-ROOT-023`, `PERF-KIND-001`.

## Benchmark Philosophy

- Benchmark evidence is first-class release evidence, not ad hoc local output.
- Every benchmark run must use committed dataset metadata and committed configuration.
- Results are comparable only when isolation and reproducibility settings are identical.

## Target Definitions

- Latency metrics use `milliseconds` and are reported as `p50`, `p95`, and `p99`.
- Throughput metrics use `operations_per_second`.
- Dataset scale tiers are explicit: `small`, `medium`, `large`, `x_large`.

## Governing Sources

- `configs/perf/slo.yaml`
- `configs/perf/budgets.yaml`
- `configs/perf/benches.json`
- `configs/perf/exceptions.json`
- `configs/perf/benchmark-config.json`
- `configs/perf/benchmark-datasets.json`
- `configs/contracts/perf/benchmark-config.schema.json`
- `configs/contracts/perf/benchmark-datasets.schema.json`
- `configs/contracts/perf/benchmark-result.schema.json`
- `ops/report/gene-lookup-baseline.json`
- `the built-in gene-lookup scenario embedded in bijux-dev-atlas`

## Reproducibility Rules

- Inputs are fixed and committed.
- Randomized inputs use a fixed seed.
- The local harness uses localhost only and does not require external network.
- Comparison uses committed baseline artifacts, not ad hoc terminal output.

## Commands

1. `bijux-dev-atlas perf validate --format json`
2. `bijux-dev-atlas perf run --scenario gene-lookup --format json`
3. `bijux-dev-atlas perf diff ops/report/gene-lookup-baseline.json artifacts/perf/gene-lookup-load.json --format json`
4. `bijux-dev-atlas perf cold-start --format json`
5. `bijux-dev-atlas perf kind --profile perf --format json`
6. `bijux-dev-atlas perf benches list --format json`

## Triage

If a perf contract fails:

1. Check whether an exception exists in `configs/perf/exceptions.json` and whether it is still
   within its expiry date.
2. Compare the new report to the committed baseline.
3. Check latency, throughput, memory, and CPU deltas before changing any thresholds.
4. Update the baseline only when the new result is the accepted steady-state reference.

## Acceptance

- Baseline profile uses the committed localhost baseline as the reference floor.
- Perf profile must pass the kind-backed gate before it is treated as the stronger operator target.

## Evidence

Performance artifacts are part of the governed release evidence bundle through the manifest's
`perf_assets` entry.
