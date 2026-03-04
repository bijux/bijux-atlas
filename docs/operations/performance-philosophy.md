# Performance Philosophy

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`

## Purpose

Define the principles Atlas uses for performance engineering decisions and tradeoffs.

## Principles

- Correctness and determinism are non-negotiable.
- Reproducible evidence is required before and after performance changes.
- Tail latency and error rate regressions are treated as release blockers.
- Threshold updates require explicit documented evidence.
