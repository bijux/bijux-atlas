# Performance Optimization Guidelines

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`

## Purpose

Define safe optimization rules that preserve correctness and evidence quality.

## Guidelines

1. Optimize the highest-severity regression first.
2. Prefer data-model and query-path improvements over threshold relaxation.
3. Keep optimization changes and benchmark evidence in the same review cycle.
4. Re-run regression detector and update audit assets after every optimization change.
