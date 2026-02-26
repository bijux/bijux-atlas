# Baseline Update Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines explicit approval process for updating performance baselines.

## Why

Prevents accidental acceptance of regressions as new normal.

## Contracts

- Baselines live in `ops/load/baselines/`.
- Baseline changes require explicit approval via `ATLAS_BASELINE_APPROVED=1`.
- Enforcement target: `ops-baseline-policy-check`.

## How to verify

```bash
$ ATLAS_BASELINE_APPROVED=1 make ops-baseline-policy-check
```

Expected output: baseline policy check passes when approved.

## See also

- [Load Reproducibility](reproducibility.md)
- [Load Suites](suites.md)
- `ops-load-full`

- Reference scenario: `mixed.json`
