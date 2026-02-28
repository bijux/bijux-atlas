# Contract JSON Examples

- Owner: `docs-governance`

## What

Canonical JSON examples for each registry.

## Why

Provides deterministic sample payloads for testing and documentation.

## Scope

Covers example JSON files in this directory only.

## Non-goals

Does not replace the authoritative registry JSON in `docs/contracts/`.

## Contracts

Each `*.example.json` mirrors a registry schema and remains sorted/canonical.

## Failure modes

Outdated examples can mislead tests and reviewers.

## Examples

```bash
$ rg --files docs/contracts/examples | rg '\.example\.json$'
```

Expected output: one example JSON file per registry contract.

## How to verify

```bash
$ bijux dev atlas contracts check --checks drift
$ make ssot-check
```

Expected output: format and contract checks pass.

## See also

- [Contracts Index](../contracts-index.md)
- [SSOT Workflow](../ssot-workflow.md)
- [Terms Glossary](../../_style/terms-glossary.md)
