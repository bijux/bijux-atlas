---
title: Run failure tests locally
audience: operators
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags: [resilience, testing]
---

# Run failure tests locally

1. List scenarios: `bijux dev atlas ops scenario list --format json`.
2. Plan a failure scenario: `bijux dev atlas ops scenario run --scenario ingest-crash-partial-state --plan --format json`.
3. Run with evidence: `bijux dev atlas ops scenario run --scenario ingest-crash-partial-state --evidence --allow-write --format json`.
4. Build diagnose bundle: `bijux dev atlas ops diagnose bundle --allow-write --format json`.
5. Explain bundle: `bijux dev atlas ops diagnose explain artifacts/ops/diagnose/<run-id>/bundle.json --format json`.
