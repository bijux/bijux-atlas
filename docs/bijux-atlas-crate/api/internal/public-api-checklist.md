# Public Surface Checklist

- Owner: `bijux-atlas::api`

Before merge, verify:

- Public functions are listed in public-api.md`.
- New query params are reflected in OpenAPI and parser tests.
- Error codes remain stable; added codes are documented.
- Deterministic OpenAPI output test passes.
- Forbidden deps (`tokio`, `reqwest`, `rusqlite`) are not introduced.
