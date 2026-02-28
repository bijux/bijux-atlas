# Errors Reference

Owner: `bijux-atlas-operations`  
Type: `reference`  
Reason to exist: canonical stable error code registry.

## Source Of Truth

- `docs/reference/contracts/errors.md`
- runtime error contract outputs

## Semantics

- Error codes are stable machine identifiers for client handling.
- Error `message` is human-readable and not a stable parsing surface.
- `details` contains optional structured fields for diagnostics.
- HTTP status mappings are stable for existing error codes in v1.

## Table Scope

Error code mappings are maintained only in this section.
