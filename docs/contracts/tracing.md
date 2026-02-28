# Trace Spans Contract

- Owner: `docs-governance`

## What

Defines the `Trace Spans Contract` registry contract.

## Why

Prevents drift between SSOT JSON, generated code, and operational consumers.

## Scope

Applies to producers and consumers of this registry.

## Non-goals

Does not define implementation internals outside this contract surface.

## Contracts

- `admission_control` required attributes: route
- `cache_lookup` required attributes: dataset
- `dataset_resolve` required attributes: route
- `serialize_response` required attributes: route, status
- `sqlite_query` required attributes: class
- `store_fetch` required attributes: backend, dataset

## Failure modes

Invalid or drifted registry content is rejected by contract checks and CI gates.

## Examples

```json
{
  "required_attributes": [
    "route"
  ],
  "span": "admission_control"
}
```

## How to verify

```bash
$ make ssot-check
$ make docs-freeze
```

Expected output: both commands exit status 0 and print contract generation/check success.

## See also

- [Contracts Index](contracts-index.md)
- [SSOT Workflow](ssot-workflow.md)
- [Terms Glossary](../_style/terms-glossary.md)
