# Effects

- Allowed: deterministic parsing/validation, serialization schema declaration.
- Forbidden: direct store access, sqlite access, tokio runtime concerns, network calls.
- OpenAPI generation is pure in-library; filesystem writes are done by `src/bin/openapi.rs`.
