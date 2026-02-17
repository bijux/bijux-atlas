# Bijux Config Schema Versioning

- Each subsystem must expose `config_schema_version` as a stable string.
- Version increments follow:
  - patch: typo/comment-only docs fixes
  - minor: backward-compatible new keys
  - major: breaking key semantics/removal
- Runtime version endpoint must include schema version for operator visibility.
- Config migrations must be documented before version bump.

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
