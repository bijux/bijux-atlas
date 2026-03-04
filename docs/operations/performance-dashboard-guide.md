# Performance Dashboard Guide

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@2228f79ef`

## Purpose

Describe how to read and validate performance dashboard assets.

## Sources

- `ops/report/generated/performance-dashboard.json`
- `ops/report/generated/performance-trend.json`
- `ops/report/generated/performance-trend-graph.json`

## Validation

- Ensure each required panel has a source series.
- Ensure trend graph sources resolve to generated trend data.
