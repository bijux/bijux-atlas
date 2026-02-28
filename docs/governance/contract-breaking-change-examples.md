# Breaking Change Examples

- Owner: `docs-governance`

## What

Examples of breaking vs additive changes per registry.

## Why

Provides concrete classification guidance for contract reviews.

## Scope

Covers `ERROR_CODES`, `METRICS`, `TRACE_SPANS`, `ENDPOINTS`, `CONFIG_KEYS`, `CHART_VALUES`.

## Non-goals

Does not cover internal refactors without contract impact.

## Contracts

- `ERROR_CODES`: removing/renaming a code is breaking; adding a code is additive.
- `METRICS`: removing metric/label is breaking; adding new metric is additive.
- `TRACE_SPANS`: removing required attribute is breaking.
- `ENDPOINTS`: removing path/method is breaking; adding endpoint is additive.
- `CONFIG_KEYS`: removing key is breaking.
- `CHART_VALUES`: removing key is breaking.

## Failure modes

Misclassified changes bypass compatibility checks and can break clients.

## Examples

```json
{
  "change": "remove Timeout from ERROR_CODES",
  "classification": "breaking"
}
```

Expected output: breaking-change detector fails unless explicitly coordinated.

## How to verify

```bash
$ bijux dev atlas contracts check --checks breakage
```

Expected output: breaking changes are flagged.

## See also

- [Contracts Index](../reference/contracts/contracts-index.md)
- [Contract Diff Review](../reference/contracts/contract-diff.md)
- [Glossary](../glossary.md)
