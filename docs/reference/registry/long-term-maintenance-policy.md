# Bijux Long-Term Maintenance Policy

- Each project must define supported release lines.
- Maintenance expectations:
  - security patches for supported lines
  - compatibility contract tests stay green
  - policy gates remain mandatory for merges
- EOL policy:
  - announced at least one minor release ahead
  - migration path documented before EOL date

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
