---
title: Logging Contracts
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Logging Contracts

Atlas logs are intended to be structured, correlatable, classifiable, and free
of sensitive fields. The repository currently carries two overlapping format
contracts. Operators must understand their disagreement before treating a log
stream as contract-valid.

## Contract Layers

| Contract | Required shape | Additional control |
| --- | --- | --- |
| Log-field schema | `level`, `msg`, `request_id`; registry also names `event_name` | Uppercase levels, registered events, prohibited PII fields |
| Format-validator contract | `timestamp`, `level`, `target`, `message`, `request_id`, `event_name` | Uppercase or lowercase levels; blocks password, secret, and token content |
| Classification contract | Registered event prefix | Unknown classes are violations |

The field schema uses `msg`; the validator expects `message`. The validator also
requires `timestamp` and `target`, which are not required by the field schema.
The checked-in sample uses `msg`, omits `target`, and uses `ts` rather than
`timestamp` in its first record. It therefore illustrates events but does not
prove one stream satisfies both format contracts.

Until the contracts converge, preserve the validator and schema results
separately. Do not claim complete logging conformance from a pass against only
one of them.

Consumers must not silently translate `msg` to `message` or `ts` to
`timestamp` and then report the source stream as conformant. A normalized view
can support investigation, but retain the source record and identify the
normalization rule. Contract repair belongs at the producer and governance
boundary.

## Event Semantics

The field contract registers six events: request start, request end, policy
rejection, cache lookup, store fetch, and SQLite query. It defines additional
required fields for three:

| Event | Required context |
| --- | --- |
| `request_start` | request ID, path, method |
| `request_end` | request ID, status, latency in milliseconds |
| `policy_rejection` | request ID, policy, mode, reason, limit |

The classification contract separately recognizes runtime, query, ingest,
artifact, configuration, startup, shutdown, and security prefixes. The six
registered event names do not use those prefixes. Registration and
classification are therefore distinct checks rather than one taxonomy.

## Correlation and Data Safety

```mermaid
flowchart LR
    Event["Runtime event"] --> Fields["Stable event fields"]
    Fields --> Request["request_id"]
    Fields --> Dataset["dataset identity"]
    Fields --> Trace["optional trace_id"]
    Request --> Investigation["Cross-signal investigation"]
    Trace --> Investigation
    Dataset --> Investigation
```

The field schema prohibits email, phone, IP address, social-security number,
and personal name fields. The validator additionally blocks password, secret,
and token content. These sets are cumulative. Sensitive values must be excluded
before emission, not scrubbed only when evidence is packaged.

## From Emission to Evidence

```mermaid
flowchart LR
    Code[Instrumented code path] --> Record[Structured source record]
    Record --> Validate[Format, event, and safety validation]
    Validate --> Collect[Collector or log transport]
    Collect --> Store[Access-controlled retention]
    Store --> Query[Incident query]
    Query --> Snapshot[Run-bound evidence snapshot]
```

Every arrow can lose or alter information. Producer success does not prove
collector intake. Collector intake does not prove retention. A query result
does not prove completeness unless its time range, filters, tenant, index, and
ingestion delay are known.

For an incident snapshot, retain:

- runtime, deployment, dataset, and configuration identities;
- event-time and ingestion-time bounds;
- query text, filters, tenant, and result count;
- collector and backend health for the same interval;
- original structured records or a content-addressed export; and
- any normalization or redaction applied after emission.

Use `request_id` or `trace_id` for a single request. Use bounded fields such as
route, status, event class, release, and time window to discover a population.
Do not promote high-cardinality biological identifiers into metric-like log
aggregation without an explicit data and retention review.

## Interpret Missing Logs

```mermaid
flowchart TD
    Missing[Expected event absent] --> Executed{Code path executed?}
    Executed -- no --> Behavior[Investigate request or control flow]
    Executed -- yes --> Emitted{Producer emitted?}
    Emitted -- no --> Instrumentation[Instrumentation or filtering defect]
    Emitted -- yes --> Collected{Collector accepted?}
    Collected -- no --> Transport[Transport or collector failure]
    Collected -- yes --> Retained{Backend retained and query can see it?}
    Retained -- no --> Backend[Retention, indexing, tenancy, or query failure]
    Retained -- yes --> Search[Correct correlation and time bounds]
```

An absent event is not proof that the event did not occur. Classify the missing
boundary before using log absence in a release or incident decision.

## Acceptance

Validate syntax, both required-field contracts, event-specific context,
classification, redaction, and correlation. Include successful and failing
samples for request, policy, store, and query paths. Exercise collector intake
and retrieval after the required retention interval. Reject unknown events,
missing identifiers, ambiguous field aliases, or secret-bearing records.

See [Logging, Metrics, and Tracing](logging-metrics-and-tracing.md) for
cross-signal diagnosis and [Operational Evidence Reports](operational-evidence-reports.md)
for retained snapshots.
