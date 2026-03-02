---
title: Reports schema
audience: contributor
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-01
source:
  - crates/bijux-dev-atlas/src/core/reporter
  - crates/bijux-dev-atlas/src/contracts
tags:
  - control-plane
  - reporting
  - schema
related:
  - docs/control-plane/reports-contract.md
  - docs/control-plane/ci-report-consumption.md
---

# Reports schema

## Purpose

This page defines the stable report payload shape emitted by the control-plane and the policy for schema evolution.

## Canonical fields

Governed machine-readable report payloads use this header:

- `report_id`: stable report identifier
- `version`: schema version for that report family
- `inputs`: declared inputs used to produce the report
- `summary`: stable aggregate counters or top-level state
- `evidence`: supporting metadata needed to interpret the report deterministically

Execution reports may still include runtime-specific fields such as:

- `schema_version`: integer schema marker for the broader execution envelope
- `kind`: report kind identifier
- `status`: pass or fail summary
- `violations`: array of violation entries
- deterministic `run_id` metadata when the report belongs to a specific run

Violation entries must include:

- `contract_id`
- `test_id`
- `file` (when path-scoped)
- `line` (when available)
- `message`
- `evidence` (optional)

## Versioning policy

- Backward-compatible additions increment fields within the same `schema_version`.
- Breaking changes require a `schema_version` increment.
- Breaking changes also require an explicit changelog entry in the contracts governance surface.
- CI and local consumers must treat unknown fields as additive and safe.

## Compatibility expectations

- Required CI gates consume structured fields, not terminal formatting.
- Historical artifacts remain readable by version-aware tooling.
- Schema changes must include fixture updates and contract coverage for deterministic output.
- The schema registry and ownership registries under `configs/reports/` are the SSOT for governed report families.

## See also

- [Reports contract](reports-contract.md)
- [CI report consumption](ci-report-consumption.md)
- [Add a check in 30 minutes](add-a-check-in-30-minutes.md)
