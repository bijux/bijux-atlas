# Errors reference

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: provide canonical error taxonomy and HTTP mapping references.

## Error taxonomy

- Validation errors: request shape or parameter violations.
- Compatibility errors: unsupported version/contract expectations.
- Runtime availability errors: downstream or service state failures.

## Canonical sources

- Error codes: [Contracts Errors](contracts/errors.md)
- Envelope schema: `docs/reference/contracts/schemas/ERROR_SCHEMA.json`
- Status mapping: `docs/reference/contracts/schemas/ERROR_STATUS_MAP.json`

## Usage boundaries

- API docs explain how clients should interpret and handle these codes: [API errors](../api/errors.md).
- Operations docs explain what to do when a service incident is ongoing: [Operations incident response](../operations/incident-response.md).
- This page stays descriptive: taxonomy, envelope, and status mapping only.

## Next steps

- [API errors](../api/errors.md)
- [Schemas reference](schemas.md)
