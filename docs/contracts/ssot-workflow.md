# SSOT Workflow

- Owner: `docs-governance`

## What

This is the only workflow document for contract evolution.

## Why

A single workflow definition removes ambiguity in contract update process.

## Scope

Applies to all JSON registries under `docs/contracts/`.

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
- Unformatted contracts: `atlasctl contracts check --checks drift` check fails.

## Examples

```bash
$ ./bin/atlasctl contracts check --checks drift
$ ./bin/atlasctl contracts generate --generators artifacts
$ make ssot-check
```

Expected output: all checks pass with no drift.

## How to verify

```bash
$ make ssot-check
```

Expected output: contract pipeline exits 0.

## See also

- [Contracts Index](contracts-index.md)
- [Contract Change Checklist](contract-change-checklist.md)
- [Terms Glossary](../_style/terms-glossary.md)
