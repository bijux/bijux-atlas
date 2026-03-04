# Performance Philosophy

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@2228f79ef`

## Purpose

Define the principles Atlas uses for performance engineering decisions and tradeoffs.

## Principles

- Correctness and determinism are non-negotiable.
- Reproducible evidence is required before and after performance changes.
- Tail latency and error rate regressions are treated as release blockers.
- Threshold updates require explicit documented evidence.
