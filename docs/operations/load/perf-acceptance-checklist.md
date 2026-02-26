# Perf Acceptance Checklist

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Checklist mapping performance acceptance gates to explicit make targets.

## Checklist

- Required scenario set: `cold-start.json`, `warm-steady-state-p99.json`, `spike-overload-proof.json`, `store-outage.json`, `pod-churn.json`
- Suite manifest/schema gate: `make ops-load-manifest-validate`
- Smoke load + score gate: `make ops-load-smoke`
- Full load + score gate: `make ops-load-full`
- Soak memory gate: `make ops-load-soak`
- Baseline regression gate: `make ops-perf-nightly`
- Baseline update policy gate: `make ops-baseline-policy-check`
- Performance report generation: `make ops-perf-report`

## Failure Triage

- Run runbook: `docs/operations/runbooks/load-failure-triage.md`
- Reproduce with: `make ops-load-nightly`
