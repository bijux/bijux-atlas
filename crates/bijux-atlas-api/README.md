# bijux-atlas-api

Stable Atlas API contracts, OpenAPI metadata, and request or response normalization helpers.

This crate owns:

- request parameter parsing and validation
- response DTO and error envelope contracts
- OpenAPI v1 document generation
- compatibility redirects and stable API error-code aliases

It depends on `bijux-atlas-core` for canonical hashing and shared error codes, and on
`bijux-atlas-model` for stable dataset and query value types.
