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

- `bijux dev atlas contract list`
- `bijux dev atlas contract describe DOC-001`
- `bijux dev atlas contract run --mode static`
- `bijux dev atlas contract run --mode effect --effects-policy allow`

## Make Wrappers

- `make contract` delegates to `bijux dev atlas contract run --mode static`
- `make contract-effect` delegates to `bijux dev atlas contract run --mode effect --effects-policy allow`
- `make contract-all` delegates to `bijux dev atlas contract run --mode all --effects-policy allow`
- `make contract-list` delegates to `bijux dev atlas contract list`
- `make contracts`, `make contracts-effect`, and `make contracts-all` remain temporary deprecated wrappers

## Compatibility

- `contracts` prints a deprecation warning unless `--no-deprecation-warn` is set.
- New docs and generated command inventories intentionally show `contract` only.
