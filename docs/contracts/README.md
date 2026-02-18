# Contracts SSOT and Generation

- Owner: `docs-governance`

## What

Defines how contracts are sourced from SSOT registries and generated into docs/code artifacts.

## Why

Prevents drift between machine contracts, generated references, and runtime behavior.

## Scope

Applies to contract registries under `docs/contracts/*.json` and generated docs under `docs/_generated/`.

## Non-Goals

Does not document runtime implementation internals outside contract surfaces.

## Contracts

- SSOT registries live in `docs/contracts/*.json`.
- Generated contract docs live in `docs/_generated/contracts/`.
- OpenAPI generated docs live in `docs/_generated/openapi/`.

## Failure Modes

- Registry updates without regeneration cause contract drift failures.
- Generated docs modified manually drift from SSOT and fail checks.

## Examples

```bash
$ make ssot-check
$ make docs-freeze
```

Expected output: both commands exit 0 with no contract drift.

## How To Verify

```bash
$ make ssot-check
$ make docs
```

Expected output: contract checks and docs checks pass.

## See Also

- [Contracts SSOT](INDEX.md)
- [Contracts Index](contracts-index.md)
- [SSOT Workflow](ssot-workflow.md)
- [Terms Glossary](../_style/terms-glossary.md)
