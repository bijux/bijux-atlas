# Bijux Logging And Tracing Standard

## Logging Format

- Default log format is JSON for all runtime services.
- Human-text logs may be enabled only with explicit local override.
- Required structured fields:
  - `timestamp`
  - `level`
  - `subsystem`
  - `version`
  - `request_id`

## Trace ID Propagation

- Incoming `x-request-id` is accepted and propagated unchanged.
- If missing, `traceparent` is used as source for request correlation.
- If both are missing, service generates a deterministic request id.
- Response must include `x-request-id` header.

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
