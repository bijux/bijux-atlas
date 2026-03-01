# Versioning

- `v1` is additive-only for fields and endpoints.
- Existing response fields are stable and must not change semantics.
- Error `code` values are stable machine contracts.
- Breaking changes require introducing `v2` path namespace.
