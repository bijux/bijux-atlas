# Errors

Owner: `api-contracts`  
Type: `guide`  
Surface version: `v1`  
Reason to exist: define the canonical API error envelope and usage model.

## Envelope

All errors return an `error` object with:

- `code`
- `message`
- `details` (optional structured fields)
- `request_id`

Codes and terms align with the repository glossary and reference error catalog.

## HTTP Mapping Principles

- Validation and contract violations: `4xx`.
- Availability and upstream failures: `5xx`.
- Response and retry expectations stay stable for existing codes.

## Example

```bash
curl -i -fsS 'http://127.0.0.1:8080/v1/genes?limit=0'
```

## Related References

- [Errors Reference](../reference/errors.md)
- [Schemas Reference](../reference/schemas.md)
