---
title: Reproducibility
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Reproducibility

Operational reproducibility is modeled explicitly through scenarios,
fixtures, and release-shaped rebuild checks.

## Purpose

Use this page to understand which release-critical artifacts Atlas expects to
rebuild deterministically and what counts as reproducible enough for release
confidence.

## Source of Truth

- `ops/reproducibility/scenarios.json`
- `ops/reproducibility/spec.json`
- `ops/reproducibility/fixtures/`
- `ops/reproducibility/report.schema.json`
- `ops/reproducibility/ci-scenario.json`

## Reproducibility Program

Atlas currently defines rebuild scenarios for:

- crates
- docker images
- Helm chart packages
- docs site output
- the release bundle itself

The reproducibility objective is to prove release-critical artifact identity
through signals such as source snapshot hashes, scenario results, and artifact
hashes.

## What Counts as Reproducible Enough

The release is reproducible enough when the governed scenarios complete with
stable output identity and the report schema records an `ok` status for the
artifacts under review.

## Related Contracts and Assets

- `ops/reproducibility/scenarios.json`
- `ops/reproducibility/fixtures/`
- `ops/reproducibility/spec.json`
- `ops/reproducibility/report.schema.json`
