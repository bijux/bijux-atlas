# Performance FAQ

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@5aac97716`

## Why are baselines committed?

Committed baselines make regressions reviewable and reproducible across environments.

## When can thresholds change?

Only after fresh benchmark evidence and documented approval.

## What blocks release?

Critical regressions in p99 latency or error rate block release.

## How are noisy runs handled?

Use history/trend evidence and anomaly classification before applying exceptions.
