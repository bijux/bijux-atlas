# API Errors

`ApiError` schema is fixed:

- `code`: stable machine code from SSOT error registry.
- `message`: human-readable summary.
- `details`: stable object payload for diagnostics.

Unknown fields are rejected for serialized/deserialized API errors.

Error code source of truth:
- `docs/reference/contracts/schemas/ERROR_CODES.json`
- generated runtime mapping: `src/generated/error_codes.rs`
