# Sharding Behavior

- Owner: `bijux-atlas-query`

In sharded datasets, region queries fan out only to relevant shards with bounded concurrency.

- Non-region lookups use global/indexed shard paths.
- Response semantics are identical between monolithic and sharded modes.

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
