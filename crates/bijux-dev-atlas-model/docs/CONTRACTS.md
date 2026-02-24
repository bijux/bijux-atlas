# Dev Atlas Model Contracts

## Stable Contracts
- `RunReport`, `CheckResult`, `Violation`, and `EvidenceRef` are persisted report artifacts.
- `schema_version` is carried on each persisted artifact.
- `CheckId`, `ViolationId`, and `ArtifactPath` are typed identifiers used in serialized output.
- Violation fingerprints are deterministic for identical fields.

## Internal Details
- Validation helper implementations and formatting helpers are internal implementation details.
- Additional non-persisted helper functions may change without compatibility guarantees.

## Compatibility Promise
- Existing stable fields are append-only unless a schema version increment is introduced.
- `schema_version` changes require fixture updates and consumer compatibility review.
