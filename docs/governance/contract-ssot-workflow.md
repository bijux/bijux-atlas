# SSOT Workflow

- Owner: `docs-governance`

## What

This is the only workflow document for contract evolution.

## Why

A single workflow definition removes ambiguity in contract update process.

## Scope

Applies to all JSON registries under `docs/reference/contracts/schemas/`.

## Non-goals

Does not define product release choreography outside contract updates.

## Contracts

1. Update SSOT registry JSON.
2. Run generator.
3. Run drift checks.
4. Review breaking-change detection output.
5. Commit SSOT and generated outputs together.

## Failure modes

- Drifted generated output: `check_contract_drift.py` fails.
- Breaking change: `check_breaking_contract_change.py` fails.
- Unformatted contracts: `bijux dev atlas check run --suite ci_fast` check fails.

## Examples

```bash
$ bijux dev atlas contracts check --checks drift
$ bijux dev atlas contracts generate --generators artifacts
$ make contracts
```

Expected output: all checks pass with no drift.

## How to verify

```bash
$ make contracts
```

Expected output: contract pipeline exits 0.

## See also

- [Contracts Index](../reference/contracts/index.md)
- [Contract Change Checklist](contract-change-checklist.md)
- [Glossary](../glossary.md)
