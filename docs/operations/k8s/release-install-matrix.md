# Release Install Matrix

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Generated matrix of k8s install/test profiles from CI summaries.

## Why

Provides a stable compatibility view across supported chart profiles.

## Contracts

Generated at: `unknown`

Profiles:

Verified test groups:

## Failure modes

Missing profile/test entries indicate CI generation drift or skipped suites.

## How to verify

```bash
$ bijux dev atlas ops k8s --report text contracts
$ make docs
```

Expected output: matrix doc updated and docs checks pass.

## See also

- [K8s Test Contract](k8s-test-contract.md)
- [Helm Chart Contract](chart.md)
- `ops-k8s-tests`
