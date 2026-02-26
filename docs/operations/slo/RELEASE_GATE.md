# SLO Release Gate

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Purpose

Define when a release is blocked by SLO non-compliance.

## Block conditions

A release is blocked when any of these is true:

- `violated_slos > 0` in the generated SLO report.
- `compliance_ratio < 0.95` in the generated SLO report.
- Any burn-rate alert in `page` severity is firing for cheap/standard classes.
- Required SLO evidence artifacts are missing (`slo-report.json`, metrics snapshot).

## Required evidence

- `make ops-slo-report`
- `make ops-slo-alert-proof`
- `make ops-report`

## Override path

- Emergency override requires explicit incident reference and follow-up action items in `docs/operations/slo/CHANGELOG.md`.
- Override does not change target values in `configs/ops/slo/slo.v1.json`.
