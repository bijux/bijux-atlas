# Public API Contract

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`

## Contract Scope

Atlas public API contract is defined by the OpenAPI `v1` document and endpoint lifecycle metadata.

## Contract Sources

- `configs/openapi/v1/openapi.generated.json`
- `ops/api/surface-registry.json`
- `ops/api/contracts/openapi-schema-validation-contract.json`

## Verification Commands

- `bijux-dev-atlas api verify --format json`
- `bijux-dev-atlas api contract --format json`
