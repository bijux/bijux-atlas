# Ingestion Purity Rule

Ingestion is a pure file transformation.

Allowed:

- Read local input files (GFF3, FASTA, FAI).
- Write local output files (SQLite, manifest, copied inputs under artifact root).

Forbidden:

- Network calls.
- External DB access.
- External process spawning.

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
