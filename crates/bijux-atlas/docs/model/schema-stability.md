# Schema Stability (v1)

Rules for v1 public types:
- Additive-only evolution for serde structs/enums.
- Existing required fields must not be removed or renamed.
- Behavioral defaults must remain backward-compatible.

Strictness:
- `ArtifactManifest` and `Catalog` use `deny_unknown_fields` for strict contract validation.
- Breaking schema moves require version bump + compatibility notes update.
