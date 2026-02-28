# Release Lane Guarantees

- Owner: `bijux-atlas-operations`
- Status: `stable`
- Audience: `operators`

## Lane map

- `local`: ad hoc developer runs with no merge-blocking selection.
- `pr`: all required contracts plus static coverage.
- `merge`: required contracts plus effect coverage needed for merge readiness.
- `release`: full matrix coverage across required, effect, and slow checks.

## Commands

- `make contracts-pr`
- `make contracts-merge`
- `make contracts-release`

## Required contracts

- Source-of-truth: `ops/policy/required-contracts.json`
- Committed artifact: `artifacts/contracts/required.json`
- Approval metadata: `ops/policy/required-contracts-change.json`
