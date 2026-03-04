# Performance Regression Policy

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@98e1c68fc`

## Purpose

Define the canonical policy for detecting, classifying, and gating performance regressions.

## Governing Source

- `configs/perf/regression-policy.json`

## Required Controls

- explicit thresholds for latency, throughput, error rate, and memory growth
- CI blocking rules for critical regressions
- bounded triage budget for follow-up actions
- explicit alert routing for critical and warning classes

## Expected Outcome

Performance regressions are detected automatically, classified consistently, and surfaced with actionable evidence.
