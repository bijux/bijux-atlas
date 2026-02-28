# Errors Reference

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

## Troubleshooting links

- API usage and handling: [API Errors](../api/errors.md)
- Service incidents: [Operations Incident Response](../operations/incident-response.md)

## Next

- [API Errors](../api/errors.md)
- [Schemas Reference](schemas.md)
