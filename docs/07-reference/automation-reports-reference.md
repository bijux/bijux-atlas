---
title: Automation Reports Reference
audience: maintainer
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Automation Reports Reference

This page describes the report artifacts exposed by the Atlas development control plane.

## Report Commands

Use the `reports` command family for catalog and validation tasks:

```bash
cargo run -q -p bijux-dev-atlas -- reports list --format json
cargo run -q -p bijux-dev-atlas -- reports index --format json
cargo run -q -p bijux-dev-atlas -- reports progress --format json
cargo run -q -p bijux-dev-atlas -- reports validate --dir artifacts
```

## Shared Report Header

Governed report schemas under `configs/schemas/contracts/reports/` consistently require these fields:

- `report_id`: stable report family identifier
- `version`: schema version for the report family
- `inputs`: declared inputs used to produce the report
- `summary`: top-level counters or state
- `evidence`: supporting metadata needed to interpret the result

Report-specific payload fields appear after that shared header. For example, `docs-site-output` adds fields such as `docs_dir`, `site_dir`, `checks`, `counts`, `assets_directory`, and `status`.

## Current Governed Report Families

The current `reports list --format json` catalog exposes at least these report ids:

- `closure-index`
- `docs-build-closure-summary`
- `docs-site-output`
- `helm-env`
- `ops-profiles`

Each catalog entry points to both a schema in `configs/schemas/contracts/reports/` and an example artifact path.

## Artifact Path Pattern

Most generated reports live under workspace-controlled artifact roots such as:

- `artifacts/run/<run_id>/...` for run-scoped execution outputs
- `artifacts/contracts/ops/...` for contract-oriented artifacts
- `artifacts/governance/...` for governance indexes such as the ADR catalog

Treat those paths as report storage locations, not as new sources of truth. The contract lives in the schema and in the command that emits the report.

## Validation Rules

- schema changes must stay backward compatible unless the report version changes
- consumers should key off structured fields, not terminal formatting
- report validation should happen against a directory root, not through manual spot checks
- unknown additive fields should not break tolerant consumers

## Related Pages

- [Automation Command Surface](automation-command-surface.md)
- [Automation Contracts](../08-contracts/automation-contracts.md)
