# Schema Evolution Notes

Model evolution rules for v1:
- Add fields only when they are optional or have backward-compatible defaults.
- Do not remove or rename existing serialized fields.
- Keep enum variants additive-only; public enums are `#[non_exhaustive]`.
- Preserve canonical ordering and formatting contracts across versions.

Compatibility process:
- Update `SCHEMA_STABILITY.md` and public API docs for any surface evolution.
- Add round-trip and strict-unknown-field tests for new/changed model types.
