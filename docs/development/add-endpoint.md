# Add a New Endpoint

- Owner: `docs-governance`
- Stability: `stable`

## What

SSOT-first workflow for adding an HTTP endpoint.

## Why

Prevents API drift between implementation, OpenAPI, and contract docs.

## Scope

Endpoint registry, OpenAPI generation, server implementation, and tests.

## Non-goals

Does not describe endpoint business logic design.

## Contracts

- Endpoint must be declared in [Endpoints Contract](../contracts/endpoints.md).
- OpenAPI output must stay under `docs/_generated/openapi/`.
- Generated endpoint contract doc must remain in sync.

## Steps

1. Update the [Endpoints Contract](../contracts/endpoints.md) SSOT source.
2. Implement endpoint in server crate.
3. Add/adjust tests.
4. Regenerate contract artifacts and OpenAPI docs.

## Failure modes

- Endpoint exists in code but not in SSOT.
- OpenAPI/contract docs drift from implementation.

## How to verify

```bash
$ make contracts
$ make dev-test
$ make docs
```

Expected output: no contract drift and all endpoint tests pass.

## See also

- [Endpoints Contract](../contracts/endpoints.md)
- [SSOT Workflow](../contracts/ssot-workflow.md)
- [Terms Glossary](../_style/terms-glossary.md)
