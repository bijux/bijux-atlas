# Performance Harness

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@2aa6d806f61cbfa5a9275332f8b6e80fdfd0553e`

## Purpose

This directory defines the deterministic load harness inputs consumed by
`bijux-dev-atlas perf run`. The command starts a localhost HTTP fixture server in-process and uses
these scenario files as the source of truth for concurrency, duration, warmup, and request shape.

## Files

- `gene-lookup.json`: canonical representative query scenario for the governed p99 check.

## Update rule

Only change these inputs alongside fresh perf evidence and a justified SLO review.
