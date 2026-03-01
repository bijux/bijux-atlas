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

All check reports use this shared envelope:

- `schema_version`: integer schema marker
- `kind`: report kind identifier
- `status`: pass or fail summary
- `summary`: aggregate counters
- `violations`: array of violation entries
- `generated_at` or deterministic run metadata when required by the lane

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

## See also

- [Reports contract](reports-contract.md)
- [CI report consumption](ci-report-consumption.md)
- [Add a check in 30 minutes](add-a-check-in-30-minutes.md)
