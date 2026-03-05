---
title: Real Data Runs Report
audience: contributor
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
---

# Real Data Runs Report

This page is the reader entrypoint for real-data run reports and generated summaries.

## Human-readable generated reports

- [Real data runs overview](/_internal/generated/real-data-runs-overview/)
- [Real data runs table](/_generated/real-data-runs-table/)
- [Docs artifact link inventory](/_internal/generated/docs-artifact-link-inventory/)

## Source of truth

- Catalog: `configs/tutorials/real-data-runs.json`
- Schema: `configs/contracts/real-data-runs.schema.json`
- Governance contract: `docs/_internal/governance/real-data-runs-contract.md`

## Runtime evidence layout

- `artifacts/tutorials/runs/<run_id>/ingest-report.json`
- `artifacts/tutorials/runs/<run_id>/dataset-summary.json`
- `artifacts/tutorials/runs/<run_id>/query-results-summary.json`
- `artifacts/tutorials/runs/<run_id>/evidence-bundle.json`
- `artifacts/tutorials/runs/<run_id>/manifest.json`
- `artifacts/tutorials/runs/<run_id>/bundle.sha256`
