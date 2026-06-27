# bijux-atlas-api

`bijux-atlas-api` owns the stable Atlas API boundary: request parameter
parsing, response DTOs, compatibility aliases, and OpenAPI generation that
runtime adapters expose but do not redefine.

Use this crate when you need:

- request parameter parsing and validation
- response DTO and error envelope contracts
- OpenAPI v1 document generation
- compatibility redirects and stable API error-code aliases
- crate-owned API contract, HTTP surface, and observability test coverage
- OpenAPI benchmark coverage and compatibility guards

Public references:

- Project docs: <https://bijux.github.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-api/latest/bijux_atlas_api/>
- Source repository: <https://github.com/bijux/bijux-atlas>
