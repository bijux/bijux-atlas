# Cross-Project Compatibility Policy

- Cross-project compatibility is contract-driven, not branch-coupled.
- Contracts include:
  - plugin metadata handshake
  - artifact schema version
  - API error schema and cursor format
- Compatibility matrix docs must be maintained per producer/consumer pair.
- No project may import internal crates from another project without explicit contract adoption.

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
