# Error Contract

Shared machine error schema:
- `code`: stable machine error code string.
- `message`: human-readable summary.
- `details`: deterministic key-value map (`BTreeMap`).

Exit-code mapping:
- `0` success
- `2` usage
- `3` validation
- `4` dependency failure
- `10` internal

Rules:
- Error `code` values are stable API once released.
- `Display` output must remain concise and deterministic.
- Unknown fields are rejected during deserialization (`deny_unknown_fields`).
