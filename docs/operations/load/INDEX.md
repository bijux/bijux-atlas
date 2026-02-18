# Operations Load Index

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

Required k6 scenarios:

- `mixed.json`
- `spike.json`
- `cold-start.json`
- `stampede.json`
- `store-outage-under-spike.json`
- `noisy-neighbor-cpu-throttle.json`
- `pod-churn.json`
- `response-size-abuse.json`
- `multi-release.json`
- `sharded-fanout.json`
- `diff-heavy.json`
- `mixed-gene-sequence.json`
- `soak-30m.json`

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
