# Benchmark Policy

Benchmarks track performance-sensitive control-plane paths.
Criterion groups map clearly to benchmark source files.

## Rules

- Benchmark groups and output names remain unique.
- Benchmark input fixtures remain stable across runs.
- Regression checks compare like-for-like scenarios only.
- Bench code must avoid network and environment-dependent variability.
- Results are interpreted with context, not single-run noise.

## Coverage

Current suites focus on inventory scans, report encoding, and policy evaluation.
