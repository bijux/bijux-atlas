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
$ ls docs/contracts/examples/*.example.json
```

Expected output: one example JSON file per registry contract.

## How to verify

```bash
$ ./scripts/contracts/format_contracts.py
$ ./scripts/contracts/check_all.sh
```

Expected output: format and contract checks pass.

## See also

- [Contracts Index](../_index.md)
- [SSOT Workflow](../SSOT_WORKFLOW.md)
- [Terms Glossary](../../_style/TERMS_GLOSSARY.md)
