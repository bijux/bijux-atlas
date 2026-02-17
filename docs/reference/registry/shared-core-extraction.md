# Shared Core Extraction

Reusable canonical utilities are extracted to dedicated repository:

- repo: `/Users/bijan/bijux/bijux-core`
- crate: `bijux-core`
- modules:
  - `canonical` (stable JSON hashing/sorting)
  - `cursor` (signed opaque cursor encoding/decoding)

`bijux-atlas` keeps local compatibility surfaces during migration, while cross-project reuse should target `bijux-core` for new integrations.

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
