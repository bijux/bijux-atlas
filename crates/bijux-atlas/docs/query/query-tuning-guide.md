# Query Tuning Guide

- Owner: `bijux-atlas-query`
- Stability: `stable`

## Purpose

Document stable tuning levers for reducing query latency without weakening determinism guarantees.

## Tuning Order

1. Validate filter selectivity and avoid low-selectivity wide region requests.
2. Prefer targeted projections over full row projection.
3. Keep pagination limits bounded and use cursor-based iteration.
4. Verify index usage with `explain_query_plan`.
5. Compare cache hit and miss behavior before changing limits.
6. Re-run benchmark suites and diff against the committed baseline.

## Guardrails

- Do not bypass validation or work-unit limits for throughput gains.
- Do not tune by changing benchmark fixtures without updating baseline evidence.
