# Errors

Owner: `api-contracts`  
Type: `guide`  
Reason to exist: define the canonical API error envelope and usage model.

## Envelope

All errors return an `error` object with:

- `code`
- `message`
- `details` (optional structured fields)
- `request_id`

## HTTP Mapping Principles

- Validation and contract violations: `4xx`.
- Availability and upstream failures: `5xx`.
- Response and retry expectations must stay stable for existing codes.

## Related Pages

- [Error Codes Reference](../reference/errors.md)
- [Compatibility Policy](compatibility.md)
