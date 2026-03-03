# Contract Runner

- Owner: `platform`
- Type: `reference`
- Audience: `contributors`
- Stability: `stable`

## Canonical Surface

- Preferred command: `bijux dev atlas contract ...`
- Deprecated alias: `bijux dev atlas contracts ...`
- Alias removal target: `2026-09-01`

## Purpose

The `contract` surface is the stable entrypoint for governed contract execution, listing, and
inspection. The legacy `contracts` spelling remains available for one deprecation window so
automation can migrate without a hard cutover.

## Examples

- `bijux dev atlas contract all`
- `bijux dev atlas contract doctor --format json`
- `bijux dev atlas contract docs --mode static`

## Compatibility

- `contracts` prints a deprecation warning unless `--no-deprecation-warn` is set.
- New docs and generated command inventories intentionally show `contract` only.
