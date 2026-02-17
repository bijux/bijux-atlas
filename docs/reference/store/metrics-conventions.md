# Bijux Metrics Naming Conventions

## Metric Prefix

- All Prometheus metrics must use `bijux_` prefix.

## Label Contract

Every runtime metric must include:

- `subsystem` (example: `atlas`)
- `version` (service semantic version)
- `dataset` (`all` or explicit dataset id when cardinality-safe)

## Naming Pattern

- Counters: `bijux_<domain>_<event>_total`
- Gauges: `bijux_<domain>_<value>`
- Latency summaries/p95 gauges: `bijux_<domain>_<operation>_p95_seconds`

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
