# JSON Compatibility Rules

Stable response contract for v1:
- Field names are stable.
- Field ordering is stable for deterministic responses.
- `null` vs omitted behavior is explicit and documented per field.

Null/Omission policy:
- Use `null` when the field is part of the selected projection but value is absent.
- Omit fields only when the client explicitly excluded them via `fields=` projection.
- Error payload always includes `code`, `message`, `details`.

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
