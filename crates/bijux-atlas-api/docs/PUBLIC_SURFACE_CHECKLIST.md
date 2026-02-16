# Public Surface Checklist

Before merge, verify:

- Public functions are listed in `PUBLIC_API.md`.
- New query params are reflected in OpenAPI and parser tests.
- Error codes remain stable; added codes are documented.
- Deterministic OpenAPI output test passes.
- Forbidden deps (`tokio`, `reqwest`, `rusqlite`) are not introduced.
