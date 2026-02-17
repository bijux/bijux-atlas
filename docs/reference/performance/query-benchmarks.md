# Query Benchmarks

Criterion benchmarks live in `crates/bijux-atlas-query/benches/query_patterns.rs`.

Policy:

- Benchmarks are non-CI by default.
- Run manually when query behavior/indexes change.
- Command: `cargo bench -p bijux-atlas-query --bench query_patterns`

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
