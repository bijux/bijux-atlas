# Observability Docs Checklist

- Owner: `docs-governance`

## Scope

Checklist for `docs/operations/observability/` canonical pages.

## Required pages

- [x] `INDEX.md`
- [x] `acceptance-gates.md`
- [x] `alerts.md`
- [x] `dashboard.md`
- [x] `profiles.md`
- [x] `slo.md`
- [x] `tracing.md`
- [x] `compatibility.md`

## Required headings per page

- [x] `## What`
- [x] `## Why`
- [x] `## Contracts`
- [x] `## Failure modes`
- [x] `## How to verify`

## Terminology

- [x] Use `OpenTelemetry` (not `otel`) in prose.
- [x] Use `Prometheus` capitalization in prose.
- [x] Use `Grafana` capitalization in prose.

## Verification

```bash
make docs
```
