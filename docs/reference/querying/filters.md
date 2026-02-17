# Filters

- Owner: `bijux-atlas-query`

Supported filter families: exact identifiers, prefix search, biotype/type, and region constraints.

- Unknown fields are rejected.
- Prefix and region bounds are policy-limited.
- Filter normalization is canonicalized before planning.

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
