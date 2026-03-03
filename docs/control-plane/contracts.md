# Contracts runtime behavior

- Owner: `platform`
- Type: `reference`
- Audience: `contributors`
- Stability: `stable`
- Last reviewed: `2026-03-01`
- Reason to exist: define stable exit semantics for contract execution.

## Exit codes

- Canonical CLI surface: `bijux dev atlas contract ...`
- Deprecated alias: `bijux dev atlas contracts ...` (scheduled removal `2026-09-01`)

- `0`: all selected contracts passed.
- `1`: one or more non-required contracts failed.
- `2`: usage error, including invalid wildcard filters or missing required flags.
- `3`: internal runner error.
- `4`: one or more required contracts failed.

## Required contracts lane map

- `local`: contributor verification lane.
- `pr`: pull-request required suite.
- `merge`: protected-branch merge gate.
- `release`: release promotion gate.
