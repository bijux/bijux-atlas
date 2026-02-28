# Compatibility Contract

Concept IDs: concept.compatibility-matrix

- Owner: `bijux-atlas-api`

This page is the single compatibility entrypoint.

## Scope

- API compatibility guarantees: [`docs/product/compatibility-promise.md`](../product/compatibility-promise.md)
- Plugin compatibility contract: [`docs/reference/contracts/plugin/umbrella-plugin-contract-v1.md`](plugin/umbrella-plugin-contract-v1.md)
- Artifact compatibility and producer alignment: generated matrix and registry references.

## Generated Matrix

- Compatibility matrix references are consolidated under [`docs/reference/index.md`](../index.md).

Compatibility source-of-truth inputs are contract files under `docs/reference/contracts/*.json` and generated outputs under `docs/_generated/contracts/`.

## What

Defines a stable contract surface for this topic.

## Why

Prevents ambiguity and drift across CLI, API, and operations.

## Non-goals

Does not define internal implementation details beyond the contract surface.

## Contracts

Use the rules in this page as the normative contract.

## Failure modes

Invalid contract input is rejected with stable machine-readable errors.

## Examples

```bash
$ make ssot-check
```

Expected output: a zero exit code and "contract artifacts generated" for successful checks.

## How to verify

Run `make docs docs-freeze ssot-check` and confirm all commands exit with status 0.

## See also

- [Contracts Overview](INDEX.md)
- [SSOT Workflow](ssot-workflow.md)
- [Terms Glossary](../../glossary.md)
