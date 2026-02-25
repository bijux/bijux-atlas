# API Versioning

Atlas API versioning is path-based (`/v1/...`).

Rules:
- v1 is additive-only for fields and endpoints.
- Existing fields are never removed or renamed in v1.
- Removed behavior requires:
  - prior deprecation annotation in OpenAPI
  - compatibility note in docs/reference/evolution/
  - major version bump (`/v2`).
- Cursor decoding is backward-compatible within v1.
- JSON response field order is stable and deterministic for the same request.

Deprecation:
- Deprecated endpoints/params use OpenAPI `deprecated: true`.
- Deprecations remain available for at least one minor release cycle.
- Compat mapping:
  - `/v1/releases/{release}/species/{species}/assemblies/{assembly}` -> `/v1/datasets/{release}/{species}/{assembly}`.

## What

Reference definition for this topic.

## Why

Defines stable semantics and operational expectations.

## Scope

Applies to the documented subsystem behavior only.

## Non-goals

Does not define unrelated implementation details.

## Contracts

Normative behavior and limits are listed here.

## Failure modes

Known failure classes and rejection behavior.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Terms Glossary](../../_style/terms-glossary.md)
