# Evolution Roadmap

Near-term:
- Continue additive schema and API expansions within v1.
- Keep artifact and API compatibility tests as hard CI gates.

Planned major evolution topics:
- v2 canonical transcript policy.
- v2 potential sequence model extensions.
- optional storage layout advances with unchanged external contract.

Non-negotiables:
- No column removal in v1 SQLite artifacts.
- No breaking error-contract changes in v1.
- Backward-compatible cursor decoding in v1.

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
