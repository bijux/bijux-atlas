---
title: Performance and Load
audience: operator
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Performance and Load

Atlas performance should be evaluated in terms of query shape, artifact layout, cache behavior, and runtime limits, not only raw request-per-second numbers.

## Performance Model

```mermaid
flowchart LR
    QueryShape[Query shape] --> Cost[Work cost]
    Cost --> Limits[Runtime limits]
    Limits --> Latency[Latency and throughput]
    Cache[Cache behavior] --> Latency
```

This performance model shows why Atlas performance cannot be summarized by one throughput number. The
cost of work depends on query shape, limits, and cache behavior together.

## Load Model

```mermaid
flowchart TD
    Traffic[Traffic] --> Classes[Cheap, medium, heavy classes]
    Classes --> Concurrency[Concurrency controls]
    Concurrency --> Overload[Overload behavior]
```

This load model explains why Atlas talks about traffic classes instead of treating all requests as
equal. Different classes stress the runtime differently and can trigger different guardrails.

## What Usually Drives Performance

- whether queries are explicit and selective
- whether caches are warm
- whether store access is healthy
- whether runtime concurrency limits match actual traffic shape

## Operator Advice

- measure realistic request mixes, not only synthetic happy-path queries
- observe overload and readiness under stress, not only average latency
- correlate load results with request class and policy behavior

## What Good Performance Means

Good performance is not just “fast.” It is:

- predictable under expected traffic
- explicit about overload behavior
- observable during degradation
- recoverable after stress

## A Better Performance Question

Instead of asking only “how fast is Atlas,” ask “how predictable is Atlas under the traffic mix we
actually expect to send?”

## Purpose

This page explains the Atlas material for performance and load and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
