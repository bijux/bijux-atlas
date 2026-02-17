# JSON Compatibility Rules

Stable response contract for v1:
- Field names are stable.
- Field ordering is stable for deterministic responses.
- `null` vs omitted behavior is explicit and documented per field.

Null/Omission policy:
- Use `null` when the field is part of the selected projection but value is absent.
- Omit fields only when the client explicitly excluded them via `fields=` projection.
- Error payload always includes `code`, `message`, `details`.
