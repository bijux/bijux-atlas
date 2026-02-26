# Operations Load Index

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `docs-governance`

## What

Index page for `operations/load` documentation.

## Why

Provides a stable section entrypoint.

## Scope

Covers markdown pages directly under this directory.

## Non-goals

Does not duplicate page-level details.

## Contracts

List and maintain links to section pages in this directory.

Section pages:
- `suites.md`
- `k6.md`
- `ci-policy.md`
- `result-contract.md`
- `reproducibility.md`
- `baseline-update-policy.md`
- `runtime-policy.md`
- `perf-acceptance-checklist.md`

Required load suite IDs:

- `mixed`
- `cheap-only-survival`
- `warm-steady-state-p99`
- `cold-start-p99`
- `spike-overload-proof`
- `store-outage-under-spike`
- `pod-churn`
- `response-size-abuse`
- `multi-release`
- `sharded-fanout`
- `diff-heavy`
- `mixed-gene-sequence`
- `soak-30m`

Scenario file paths are internal implementation details under `ops/load/scenarios/`.
Use only suite IDs from `ops/load/suites/suites.json` in docs and operator workflows.

## Failure modes

Missing index links create orphan docs.

## How to verify

```bash
$ make docs
```

Expected output: docs build and docs-structure checks pass.

## See also

- [Docs Home](../../index.md)
- [Naming Standard](../../_style/naming-standard.md)
- [Terms Glossary](../../_style/terms-glossary.md)
- `ops-ci`
