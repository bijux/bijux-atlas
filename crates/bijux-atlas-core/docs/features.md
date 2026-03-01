# Feature Flags Policy

`bijux-atlas-core` should keep feature usage near zero.

Rules:
- No optional behavior that changes deterministic output.
- Add feature flags only for compile-time integration boundaries, never semantics.
- Any new feature must be documented with deterministic impact analysis.

Current features:
- `serde` (default): enables `serde_json`-backed canonical JSON helpers.
