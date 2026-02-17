# Ordering

- Owner: `bijux-atlas-query`

Default ordering is deterministic and query-class specific.

- Gene list defaults to `(seqid, start, gene_id)`.
- Tie-break keys are always explicit and stable.
- Ordering never depends on insertion order.

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
